/// Integration test: verifies that a cold Overpass tile fetch completes within
/// an acceptable time bound after the 8-second HTTP timeout is applied.
///
/// TDD RED (before fix): no timeout on http.get() → hangs for 15 s →
///   completer.future.timeout(10 s) throws → test FAILS.
/// TDD GREEN (after fix): .timeout(Duration(seconds: 8)) applied →
///   _doFetch catches the TimeoutException after 8 s → onFetchComplete fires →
///   stopwatch.elapsed < 10 s → test PASSES.
@Tags(['integration'])
library;

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:gps_companion/data/biome_service.dart';
import 'package:http/http.dart' as http;

/// HTTP client that hangs for 15 seconds — simulates a slow Overpass
/// connection with no timeout configured.
class _SlowHttpClient extends http.BaseClient {
  @override
  Future<http.StreamedResponse> send(http.BaseRequest request) async {
    await Future<void>.delayed(const Duration(seconds: 15));
    return http.StreamedResponse(const Stream.empty(), 200);
  }
}

void main() {
  test(
    'when_overpass_http_hangs_then_prefetch_completes_within_10s',
    () async {
      // Arrange: BiomeService with a slow HTTP client that takes 15 s.
      final service = BiomeService(httpClient: _SlowHttpClient());
      final completer = Completer<void>();
      service.onFetchComplete = completer.complete;

      final stopwatch = Stopwatch()..start();

      // Act: trigger the prefetch (fires debounce timer → _doFetch).
      unawaited(service.prefetch(51.51, -0.12));

      // Wait for debounce to fire, then for _doFetch to finish.
      // With the 8 s HTTP timeout: completer resolves after ~8.2 s.
      // Without the timeout: hangs for 15 s → timeout below throws.
      await completer.future.timeout(
        const Duration(seconds: 10),
        onTimeout: () => throw TimeoutException(
          'prefetch took longer than 10 s — HTTP timeout not applied',
        ),
      );
      stopwatch.stop();

      // Assert: fetch resolved well within the 10-second deadline.
      expect(stopwatch.elapsed, lessThan(const Duration(seconds: 10)));
    },
    timeout: const Timeout(Duration(seconds: 15)),
  );
}
