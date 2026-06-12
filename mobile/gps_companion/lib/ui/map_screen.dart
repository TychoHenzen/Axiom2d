/// Primary screen: a live map centered on the player, the collection-radius
/// brush, and the ground already vacuumed. Moving through the world continuously
/// harvests grains from freshly covered cells. Uncovered ground is rendered as
/// biome-colored fog; walking clears the fog and spawns grain pixels into the
/// animated tube on the right.
library;

import 'dart:math' show Random;

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart' show Ticker;
import 'package:flutter_background_service/flutter_background_service.dart';
import 'package:flutter_map/flutter_map.dart';
import 'package:geolocator/geolocator.dart';
import 'package:latlong2/latlong.dart';
import 'package:wakelock_plus/wakelock_plus.dart';

import '../data/biome_service.dart';
import '../domain/biome.dart';
import '../domain/biome_def.dart';
import '../domain/coverage.dart';
import '../domain/grain.dart';
import '../domain/grain_pixel.dart';
import '../domain/inventory.dart';
import '../domain/leyline.dart';
import '../domain/projection.dart';
import '../domain/route_log.dart';
import '../domain/spawn.dart' show dayNumber;
import 'app_state.dart';
import 'ca_settling.dart';
import 'pixel_physics.dart';
import 'piston_animation.dart';
import 'tube_painter.dart';

/// Maps a BiomeSample (OSM tag-level) to the legacy Biome enum used by the
/// spawn / coverage system. Picks by the dominant index in dist:
///   [0]=nature, [1]=urban, [2]=water, [3]=earth, [4]=arcane
Biome _biomeSampleToEnum(BiomeSample sample) {
  final dist = sample.dist;
  var maxIdx = 0;
  for (var i = 1; i < dist.length; i++) {
    if (dist[i] > dist[maxIdx]) maxIdx = i;
  }
  return switch (maxIdx) {
    0 => Biome.forestPark,
    1 => Biome.urbanResidential,
    2 => Biome.waterCoast,
    3 => Biome.mountainDesert,
    4 => Biome.historicCultural,
    _ => Biome.noData,
  };
}

class MapScreen extends StatefulWidget {
  const MapScreen({super.key, required this.state});

  final AppState state;

  @override
  State<MapScreen> createState() => _MapScreenState();
}

class _MapScreenState extends State<MapScreen>
    with SingleTickerProviderStateMixin {
  final MapController _map = MapController();
  Position? _pos;
  String _status = 'Acquiring GPS…';
  bool _mapReady = false;
  bool _follow = true;
  bool _initialCentered = false;
  GpsPoint? _lastFix;
  bool _backgroundEnabled = false;

  // ── Grain tube simulation ─────────────────────────────────────────────────
  final _rng = Random();
  final List<TubePixel> _tubePixels = [];
  // Height is set from MediaQuery on first build and updated when it changes.
  double _tubeHeight = 200.0;
  late TubePhysics _tubePhysics;
  late CaSettleParams _caParams;
  late SandGrid _sandGrid;
  final _pistonState = PistonState();
  late final Ticker _ticker;
  int _lastTickUs = 0;
  double _prevPackProgress = 0.0;

  // ── Fog grid (uncovered viewport cells) ───────────────────────────────────
  // Fog covers ground the player has NOT walked through yet. Each coarse cell
  // is ~27.5 m wide (5× the fine paint cell). Covered fine cells clear their
  // parent coarse cell; the fog lifts as the player walks.
  static const double _fogCellDegrees = kPaintCellDegrees * 5; // ~27.5 m
  static const double _fogCellHalf = _fogCellDegrees / 2;
  static const int _maxFogTiles = 2500; // viewport-culled cap
  // Each entry carries its center point and the biome color for that cell.
  List<({LatLng center, Color color})> _fogGrid = const [];
  bool _fogDirty = false;
  late final BiomeService _biomeService;

  @override
  void initState() {
    super.initState();
    _tubePhysics = TubePhysics(tubeHeight: _tubeHeight);
    _caParams = CaSettleParams(gridHeight: _tubeHeight.round());
    _sandGrid = SandGrid(_caParams);
    _biomeService = BiomeService()
      ..onTileLoaded = () {
        if (mounted) setState(() => _fogDirty = true);
      };
    WakelockPlus.enable();
    _startLocation();
    _ticker = createTicker(_onTick)..start();
    final bgService = FlutterBackgroundService();
    bgService.isRunning().then((running) {
      if (running) {
        bgService.invoke('stopService');
        _replayBackgroundPoints();
      }
    });
  }

  @override
  void dispose() {
    _ticker.dispose();
    WakelockPlus.disable();
    if (_backgroundEnabled) {
      FlutterBackgroundService().invoke('stopService');
    }
    super.dispose();
  }

  // ── Simulation tick ───────────────────────────────────────────────────────

  void _onTick(Duration elapsed) {
    final us = elapsed.inMicroseconds;
    final dt = _lastTickUs == 0
        ? 0.0
        : ((us - _lastTickUs) / 1e6).clamp(0.0, 0.05);
    _lastTickUs = us;

    // Fog refresh is batched to ticker rate to avoid rebuilding on every
    // gesture-callback invocation.
    if (_fogDirty && _mapReady) {
      _fogDirty = false;
      _refreshFog();
      setState(() {}); // fog grid changed — rebuild immediately
    }

    if (dt <= 0) return;
    _stepSimulation(dt);
  }

  void _stepSimulation(double dt) {
    final settled = _tubePixels
        .where((p) => p.phase == PixelPhase.settled)
        .toList(growable: false);

    var changed = false;
    for (final px in _tubePixels) {
      if (px.phase != PixelPhase.falling) continue;
      stepFalling(px, _tubePhysics, dt);
      if (shouldBeginSettling(px, settled, _tubePhysics, 1.5)) {
        final cx = px.x.round().clamp(0, _caParams.gridWidth - 1);
        final cy = px.y.round().clamp(0, _caParams.gridHeight - 1);
        trySettle(cell: GridCell(cx, cy), grid: _sandGrid, params: _caParams);
        lockSettled(px);
        changed = true;
      }
    }

    // Piston fires when packProgress wraps: was near-full, now near-zero
    // (a new pack was just forged and progress reset).
    final progress = widget.state.packProgress;
    if (_prevPackProgress > 60 &&
        progress < 20 &&
        _pistonState.phase == PistonPhase.idle) {
      _pistonState.trigger();
    }
    _prevPackProgress = progress;

    final hadBlock = _pistonState.tick(dt);
    if (hadBlock) {
      // Piston cycle complete: clear tube and reset sand grid for next fill.
      _tubePixels.clear();
      _sandGrid = SandGrid(_caParams);
      changed = true;
    }

    final anyActive = _tubePixels.any((p) => p.phase == PixelPhase.falling) ||
        _pistonState.phase != PistonPhase.idle;

    if (changed || anyActive) setState(() {});
  }

  /// Spawn one colored pixel per harvested grain, dropping from the tube top.
  void _spawnTubePixels(List<Grain> grains) {
    for (final grain in grains) {
      _tubePixels.add(TubePixel(
        x: _tubePhysics.tubeWidth / 2 + _rng.nextDouble() * 6 - 3,
        y: 2.0,
        color: grainPixelColor(grain),
        vx: _rng.nextDouble() * 40 - 20,
        vy: 20.0,
      ));
    }
  }

  // ── Fog grid ──────────────────────────────────────────────────────────────

  /// Rebuild fog grid: coarse viewport cells that have no covered fine cells.
  /// Each cell is colored by its OSM biome (from BiomeService cache) if known,
  /// or a neutral dark-gray if the tile hasn't been fetched yet.
  void _refreshFog() {
    final bounds = _map.camera.visibleBounds;
    final cam = _map.camera.center;

    // Kick off background biome fetches for a 3×3 tile grid around camera.
    // kCacheTileDegrees ≈ 0.01° (~1.1 km); three steps covers ~3 km radius.
    for (var dy = -1; dy <= 1; dy++) {
      for (var dx = -1; dx <= 1; dx++) {
        _biomeService.prefetch(
          cam.latitude + dy * kCacheTileDegrees,
          cam.longitude + dx * kCacheTileDegrees,
        );
      }
    }

    final minFX = (bounds.west / _fogCellDegrees).floor();
    final maxFX = (bounds.east / _fogCellDegrees).floor();
    final minFY = (bounds.south / _fogCellDegrees).floor();
    final maxFY = (bounds.north / _fogCellDegrees).floor();

    final covered = widget.state.coverage.covered;
    final fog = <({LatLng center, Color color})>[];

    outer:
    for (var fy = minFY; fy <= maxFY; fy++) {
      for (var fx = minFX; fx <= maxFX; fx++) {
        // A coarse cell maps to a 5×5 block of fine paint cells.
        // If any fine cell is covered, the fog has been cleared there.
        var cleared = false;
        for (var dy = 0; dy < 5 && !cleared; dy++) {
          for (var dx = 0; dx < 5 && !cleared; dx++) {
            if (covered.contains('${fx * 5 + dx}:${fy * 5 + dy}')) {
              cleared = true;
            }
          }
        }
        if (!cleared) {
          final cellLat = (fy + 0.5) * _fogCellDegrees;
          final cellLon = (fx + 0.5) * _fogCellDegrees;
          final sample = _biomeService.sampleAt(cellLat, cellLon);
          // Use the biome's defined fog color when available;
          // fall back to a neutral dark veil for unmapped tiles.
          final color = sample != null
              ? Color(sample.colorArgb)
              : const Color(0x40202020);
          fog.add((center: LatLng(cellLat, cellLon), color: color));
          if (fog.length >= _maxFogTiles) break outer;
        }
      }
    }
    _fogGrid = fog;
  }

  List<LatLng> _fogCellSquare(LatLng c) => [
        LatLng(c.latitude - _fogCellHalf, c.longitude - _fogCellHalf),
        LatLng(c.latitude - _fogCellHalf, c.longitude + _fogCellHalf),
        LatLng(c.latitude + _fogCellHalf, c.longitude + _fogCellHalf),
        LatLng(c.latitude + _fogCellHalf, c.longitude - _fogCellHalf),
      ];

  // ── GPS / vacuum ──────────────────────────────────────────────────────────

  Future<void> _replayBackgroundPoints() async {
    final store = RouteLogStore();
    final log = await store.load();
    final currentWeek = weekNumber(DateTime.now().toUtc());
    log.unprocessed;
    if (log.unprocessed.isNotEmpty) {
      final day =
          DateTime.now().toUtc().difference(DateTime.utc(1970, 1, 1)).inDays;
      for (final p in log.unprocessed) {
        await widget.state.vacuum(
          lat: p.lat,
          lon: p.lon,
          day: day,
          week: currentWeek,
        );
        await widget.state.logPosition(
          lat: p.lat,
          lon: p.lon,
          speed: p.speed,
          timestamp: p.timestamp,
          isProjected: p.isProjected,
        );
      }
      widget.state.routeLog.markProcessed();
      await widget.state.routeLogStore.save(widget.state.routeLog);
    }
  }

  Future<void> _toggleBackgroundTracking(bool enabled) async {
    final service = FlutterBackgroundService();
    if (enabled) {
      final started = await service.startService();
      if (started) setState(() => _backgroundEnabled = true);
    } else {
      service.invoke('stopService');
      setState(() => _backgroundEnabled = false);
    }
  }

  Future<void> _startLocation() async {
    if (!await Geolocator.isLocationServiceEnabled()) {
      setState(() => _status = 'Location services are off');
      return;
    }
    var perm = await Geolocator.checkPermission();
    if (perm == LocationPermission.denied) {
      perm = await Geolocator.requestPermission();
    }
    if (perm == LocationPermission.denied ||
        perm == LocationPermission.deniedForever) {
      setState(() => _status = 'Location permission denied');
      return;
    }
    final last = await Geolocator.getLastKnownPosition();
    if (last != null && mounted && _pos == null) _onPosition(last);
    Geolocator.getPositionStream(
      locationSettings: AndroidSettings(
        accuracy: LocationAccuracy.high,
        distanceFilter: 5,
        intervalDuration: const Duration(seconds: 1),
      ),
    ).listen(_onPosition);
  }

  void _recenter({bool force = false}) {
    final p = _pos;
    if (p == null || !_mapReady) return;
    if (!force && !_follow) return;
    _map.move(LatLng(p.latitude, p.longitude), _map.camera.zoom);
  }

  Future<void> _onPosition(Position p) async {
    final now = DateTime.now().toUtc();
    final day = dayNumber(now);
    final week = weekNumber(now);

    // Resolve biome from the OSM service (synchronous cache hit, or noData
    // on first visit before the background prefetch returns).
    _biomeService.prefetch(p.latitude, p.longitude);
    final biomeSample = _biomeService.sampleAt(p.latitude, p.longitude);
    final biome =
        biomeSample != null ? _biomeSampleToEnum(biomeSample) : Biome.noData;

    final currentFix = GpsPoint(
      lat: p.latitude,
      lon: p.longitude,
      speed: p.speed,
      timestamp: now,
    );

    await widget.state.logPosition(
      lat: p.latitude,
      lon: p.longitude,
      speed: p.speed,
      timestamp: now,
    );

    if (_lastFix != null && currentFix.speed >= 1.0 && p.heading >= 0) {
      final projected = projectIntermediate(
        last: _lastFix!,
        current: currentFix,
        headingDegrees: p.heading,
      );
      for (final proj in projected) {
        final projDay = dayNumber(proj.timestamp);
        final projWeek = weekNumber(proj.timestamp);
        await widget.state.vacuum(
          lat: proj.lat,
          lon: proj.lon,
          day: projDay,
          week: projWeek,
          biome: biome,
        );
        await widget.state.logPosition(
          lat: proj.lat,
          lon: proj.lon,
          speed: proj.speed,
          timestamp: proj.timestamp,
          isProjected: true,
        );
      }
    }
    _lastFix = currentFix;

    setState(() {
      _pos = p;
      _status = '';
    });
    if (!_initialCentered) {
      _initialCentered = true;
      _follow = true;
      _recenter(force: true);
    } else {
      _recenter();
    }

    final res = await widget.state.vacuum(
      lat: p.latitude,
      lon: p.longitude,
      day: day,
      week: week,
      biome: biome,
    );
    if (!mounted) return;

    if (res.newCells.isNotEmpty) {
      _spawnTubePixels(res.grains);
      _fogDirty = true;
      setState(() {});
    }
    if (res.forged > 0) _toast('🎉 Forged ${res.forged} booster pack(s)!');
  }

  void _toast(String msg) {
    if (!mounted) return;
    ScaffoldMessenger.of(context)
      ..clearSnackBars()
      ..showSnackBar(
          SnackBar(content: Text(msg), duration: const Duration(seconds: 1)));
  }

  @override
  Widget build(BuildContext context) {
    final p = _pos;
    final center = p == null
        ? const LatLng(51.5074, -0.1278)
        : LatLng(p.latitude, p.longitude);
    final theme = weeklyTheme(weekNumber(DateTime.now().toUtc()));
    // Map packProgress (0..100) to tube fill level (0..0.9).
    final fillLevel =
        (widget.state.packProgress / 100.0 * 0.9).clamp(0.0, 0.9);

    // Derive tube height from screen. Update physics objects when it changes
    // (screen rotation, window resize). Mutation during build is intentional —
    // no setState needed; the ticker picks up the new values next frame.
    final mq = MediaQuery.of(context);
    final screenTubeHeight = (mq.size.height - mq.padding.top - 100.0)
        .clamp(100.0, double.infinity);
    if ((screenTubeHeight - _tubeHeight).abs() > 1.0) {
      _tubeHeight = screenTubeHeight;
      _tubePhysics = TubePhysics(tubeHeight: _tubeHeight);
      _caParams = CaSettleParams(gridHeight: _tubeHeight.round());
      _sandGrid = SandGrid(_caParams);
    }

    return Stack(
      children: [
        FlutterMap(
          mapController: _map,
          options: MapOptions(
            initialCenter: center,
            initialZoom: 17,
            onMapReady: () {
              _mapReady = true;
              _fogDirty = true;
              if (!_initialCentered && _pos != null) {
                _initialCentered = true;
                _recenter(force: true);
              }
            },
            onPositionChanged: (camera, hasGesture) {
              // Mark fog dirty on any camera movement; ticker batches the rebuild.
              _fogDirty = true;
              if (!hasGesture || !_follow) return;
              final p = _pos;
              if (p == null) return;
              final drift = Geolocator.distanceBetween(
                camera.center.latitude,
                camera.center.longitude,
                p.latitude,
                p.longitude,
              );
              if (drift > 60) setState(() => _follow = false);
            },
          ),
          children: [
            TileLayer(
              // CARTO voyager_nolabels: colored landuse, no POI clutter.
              urlTemplate:
                  'https://a.basemaps.cartocdn.com/rastertiles/voyager_nolabels/{z}/{x}/{y}.png',
              userAgentPackageName: 'com.tychohenzen.gps_companion',
            ),
            // Fog: covers UNCOVERED ground; lifts as the player walks through it.
            // Each cell is colored by its OSM biome (forest=green, water=blue, etc.)
            if (_fogGrid.isNotEmpty)
              PolygonLayer(
                polygons: [
                  for (final cell in _fogGrid)
                    Polygon(
                      points: _fogCellSquare(cell.center),
                      color: cell.color,
                      borderStrokeWidth: 0,
                    ),
                ],
              ),
            if (p != null)
              CircleLayer(
                circles: [
                  CircleMarker(
                    point: LatLng(p.latitude, p.longitude),
                    radius: kCollectRadiusMeters,
                    useRadiusInMeter: true,
                    color: Colors.blueAccent.withValues(alpha: 0.12),
                    borderColor: Colors.blueAccent.withValues(alpha: 0.6),
                    borderStrokeWidth: 2,
                  ),
                ],
              ),
            MarkerLayer(
              markers: [
                if (p != null)
                  Marker(
                    point: LatLng(p.latitude, p.longitude),
                    width: 22,
                    height: 22,
                    child: const _PlayerDot(),
                  ),
              ],
            ),
          ],
        ),
        // Tube widget: animated grain progress (RIGHT side).
        Positioned(
          right: 8,
          top: mq.padding.top + 8,
          child: TubeHud(
            height: _tubeHeight,
            pixels: _tubePixels,
            fillLevel: fillLevel,
            showPiston: _pistonState.phase != PistonPhase.idle,
            pistonProgress: _pistonState.progress,
          ),
        ),
        // Pack stack: one 5px-high colored bar per booster, bottom-aligned
        // with the tube and sitting just to its left (right: 8+20+4 = 32).
        Positioned(
          right: 32,
          bottom: mq.size.height - (mq.padding.top + 8 + _tubeHeight),
          child: PackStackHud(boosters: widget.state.boosters),
        ),
        SafeArea(
            child: _Hud(
          state: widget.state,
          themeLabel: 'This week: ${theme.label}',
          status: _status,
          backgroundEnabled: _backgroundEnabled,
          onToggleBackground: _toggleBackgroundTracking,
        )),
        const Positioned(
          left: 6,
          bottom: 4,
          child: Text(
            '© OpenStreetMap, © CARTO',
            style: TextStyle(fontSize: 9, color: Colors.black54),
          ),
        ),
        Positioned(
          left: 16,
          bottom: 16,
          child: FloatingActionButton.small(
            heroTag: 'recenter',
            onPressed: p == null
                ? null
                : () {
                    setState(() => _follow = true);
                    _recenter();
                  },
            child:
                Icon(_follow ? Icons.my_location : Icons.location_searching),
          ),
        ),
      ],
    );
  }
}

class _PlayerDot extends StatelessWidget {
  const _PlayerDot();

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: Colors.blueAccent,
        shape: BoxShape.circle,
        border: Border.all(color: Colors.white, width: 3),
      ),
    );
  }
}

class _Hud extends StatelessWidget {
  const _Hud({
    required this.state,
    required this.themeLabel,
    required this.status,
    required this.backgroundEnabled,
    required this.onToggleBackground,
  });

  final AppState state;
  final String themeLabel;
  final String status;
  final bool backgroundEnabled;
  final ValueChanged<bool> onToggleBackground;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: state,
      builder: (context, _) {
        return Padding(
          padding: const EdgeInsets.all(8),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _chip(themeLabel),
              if (status.isNotEmpty) ...[
                const SizedBox(height: 4),
                _chip(status),
              ],
              const SizedBox(height: 8),
              Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Icon(Icons.gps_fixed, color: Colors.white70, size: 16),
                  const SizedBox(width: 6),
                  const Text(
                    'Track in Background',
                    style: TextStyle(color: Colors.white70, fontSize: 12),
                  ),
                  const SizedBox(width: 4),
                  SizedBox(
                    height: 24,
                    child: Switch(
                      value: backgroundEnabled,
                      onChanged: onToggleBackground,
                      materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
                    ),
                  ),
                ],
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _chip(String text) => Container(
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
        decoration: BoxDecoration(
          color: Colors.black.withValues(alpha: 0.6),
          borderRadius: BorderRadius.circular(12),
        ),
        child: Text(text,
            style: const TextStyle(color: Colors.white, fontSize: 13)),
      );
}

/// Animated grain tube — RIGHT side of the map screen.
///
/// Displays falling grain pixels (colored by grain aspect), a settled sand
/// pile growing from the bottom, and the piston head when compressing.
/// Fill level drives from [AppState.packProgress] (0..100 → 0..0.9 tube fill).
class TubeHud extends StatelessWidget {
  const TubeHud({
    super.key,
    this.height = 200.0,
    this.fillLevel = 0.0,
    this.pixels = const [],
    this.showPiston = false,
    this.pistonProgress = 0.0,
  });

  final double height;
  final double fillLevel;
  final List<TubePixel> pixels;
  final bool showPiston;
  final double pistonProgress;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 20,
      height: height,
      child: CustomPaint(
        painter: TubePainter(
          pixels: pixels,
          fillLevel: fillLevel,
          showPiston: showPiston,
          pistonProgress: pistonProgress,
          tubeHeight: height,
        ),
      ),
    );
  }
}

/// Pack stack widget — sits to the LEFT of the tube, bottom-aligned.
///
/// Each sealed booster pack is a 5px-high × 16px-wide bar colored by the
/// pack's average grain aspect. New packs appear at the bottom; the stack
/// grows upward.
class PackStackHud extends StatelessWidget {
  const PackStackHud({super.key, this.boosters = const []});

  final List<Booster> boosters;

  /// Average the aspect colors of all grains in a booster.
  static Color _packColor(Booster b) {
    if (b.grains.isEmpty) return const Color(0x80FFFFFF);
    var r = 0.0, g = 0.0, bl = 0.0;
    for (final grain in b.grains) {
      final c = grainPixelColor(grain);
      r += c.r;
      g += c.g;
      bl += c.b;
    }
    final n = b.grains.length.toDouble();
    return Color.fromRGBO(
      (r / n * 255).round().clamp(0, 255),
      (g / n * 255).round().clamp(0, 255),
      (bl / n * 255).round().clamp(0, 255),
      0.85,
    );
  }

  @override
  Widget build(BuildContext context) {
    // Cap visible packs so the column doesn't extend past the tube top.
    final visible = boosters.length.clamp(0, 200);
    return Column(
      mainAxisSize: MainAxisSize.min,
      verticalDirection: VerticalDirection.up,
      children: [
        for (var i = 0; i < visible; i++)
          Container(
            width: 16,
            height: 5,
            margin: const EdgeInsets.only(top: 1),
            color: _packColor(boosters[i]),
          ),
      ],
    );
  }
}
