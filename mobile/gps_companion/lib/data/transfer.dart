/// LAN transfer to the paired desktop. The desktop shows a QR code holding
/// connection info; the phone scans it and pushes its booster packs as JSON
/// over a WebSocket. No internet, no backend — local network only.
library;

import 'dart:convert';

import 'package:web_socket_channel/web_socket_channel.dart';

import '../domain/inventory.dart';

/// Payload format version (must match the desktop importer).
const int kPayloadVersion = 1;

/// Connection info decoded from the desktop's QR code.
class PairingInfo {
  PairingInfo({required this.ip, required this.port, required this.token});

  final String ip;
  final int port;
  final String token;

  /// Parse the QR payload: `{"ip":"192.168.1.42","port":9876,"token":"a1b2c3"}`.
  static PairingInfo fromQr(String qr) {
    final j = jsonDecode(qr) as Map<String, dynamic>;
    return PairingInfo(
      ip: j['ip'] as String,
      port: (j['port'] as num).toInt(),
      token: j['token'] as String,
    );
  }

  Uri get wsUri => Uri.parse('ws://$ip:$port');
}

/// Build the transfer payload JSON (pure — testable without a socket).
String buildPayload({
  required List<Booster> boosters,
  required String deviceId,
  required String token,
}) {
  return jsonEncode({
    'version': kPayloadVersion,
    'device_id': deviceId,
    'token': token,
    'boosters': boosters.map((b) => b.toJson()).toList(),
  });
}

/// Send all boosters to the paired desktop, then close the connection.
Future<void> sendBoosters({
  required PairingInfo info,
  required List<Booster> boosters,
  required String deviceId,
}) async {
  final channel = WebSocketChannel.connect(info.wsUri);
  await channel.ready;
  channel.sink.add(
    buildPayload(boosters: boosters, deviceId: deviceId, token: info.token),
  );
  await channel.sink.close();
}
