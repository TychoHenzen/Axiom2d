/// Booster gallery: a grid of forged packs, each themed by its grain mix.
library;

import 'package:flutter/material.dart';

import '../domain/inventory.dart';
import 'app_state.dart';
import 'theme.dart';

class GalleryScreen extends StatelessWidget {
  const GalleryScreen({super.key, required this.state});

  final AppState state;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Booster Packs')),
      body: ListenableBuilder(
        listenable: state,
        builder: (context, _) {
          final packs = state.boosters;
          if (packs.isEmpty) {
            return const Center(
              child: Text('No packs yet.\nCollect 100 grains to forge one.',
                  textAlign: TextAlign.center),
            );
          }
          return GridView.builder(
            padding: const EdgeInsets.all(12),
            gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
              crossAxisCount: 2,
              childAspectRatio: 0.8,
              crossAxisSpacing: 12,
              mainAxisSpacing: 12,
            ),
            itemCount: packs.length,
            itemBuilder: (context, i) => _PackTile(
              booster: packs[i],
              index: i,
              onTap: () => _showDetails(context, packs[i], i),
            ),
          );
        },
      ),
    );
  }

  void _showDetails(BuildContext context, Booster b, int index) {
    final counts = <String, int>{};
    for (final g in b.grains) {
      counts[g.type.json] = (counts[g.type.json] ?? 0) + 1;
    }
    showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Pack #${index + 1}'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Forged: ${b.forgedAt.toLocal()}'),
            if (b.locationName != null) Text('Location: ${b.locationName}'),
            const SizedBox(height: 8),
            ...counts.entries.map((e) => Text('${e.key}: ${e.value}')),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }
}

class _PackTile extends StatelessWidget {
  const _PackTile({required this.booster, required this.index, required this.onTap});

  final Booster booster;
  final int index;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final theme = BoosterTheme.of(booster.grains);
    final color = grainColor(theme.dominantType);
    return InkWell(
      onTap: onTap,
      child: Container(
        decoration: BoxDecoration(
          gradient: LinearGradient(
            colors: theme.isMixed
                ? [Colors.purple, Colors.teal, Colors.amber]
                : [color, color.withValues(alpha: 0.5)],
            begin: Alignment.topLeft,
            end: Alignment.bottomRight,
          ),
          borderRadius: BorderRadius.circular(14),
          border: theme.hasGoldShimmer
              ? Border.all(color: Colors.amber, width: 3)
              : null,
        ),
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text('Pack #${index + 1}',
                style: const TextStyle(color: Colors.white, fontWeight: FontWeight.bold)),
            if (theme.isPure)
              const Chip(label: Text('PURE'), backgroundColor: Colors.white70),
            Text(
              theme.isMixed ? 'Mixed' : theme.dominantType.json,
              style: const TextStyle(color: Colors.white),
            ),
          ],
        ),
      ),
    );
  }
}
