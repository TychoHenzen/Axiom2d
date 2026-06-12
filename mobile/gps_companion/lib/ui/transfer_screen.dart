/// Transfer screen: scan the desktop's pairing QR, then push booster packs
/// over the local-network WebSocket. No internet involved.
library;

import 'package:flutter/material.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

import '../data/transfer.dart';
import 'app_state.dart';

const String kDeviceId = 'gps-companion-android';

class TransferScreen extends StatefulWidget {
  const TransferScreen({super.key, required this.state});

  final AppState state;

  @override
  State<TransferScreen> createState() => _TransferScreenState();
}

class _TransferScreenState extends State<TransferScreen> {
  bool _scanning = false;
  bool _busy = false;
  String _message = '';

  Future<void> _onDetect(BarcodeCapture capture) async {
    if (_busy) return;
    final raw = capture.barcodes.isEmpty ? null : capture.barcodes.first.rawValue;
    if (raw == null) return;
    setState(() {
      _busy = true;
      _scanning = false;
    });
    await _send(raw);
  }

  Future<void> _send(String qr) async {
    try {
      final info = PairingInfo.fromQr(qr);
      final packs = widget.state.boosters;
      await sendBoosters(info: info, boosters: packs, deviceId: kDeviceId);
      setState(() => _message = 'Sent ${packs.length} pack(s) to ${info.ip}');
    } catch (e) {
      setState(() => _message = 'Transfer failed: $e');
    } finally {
      setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Transfer')),
      body: ListenableBuilder(
        listenable: widget.state,
        builder: (context, _) {
          if (_scanning) {
            return Stack(
              children: [
                MobileScanner(onDetect: _onDetect),
                Align(
                  alignment: Alignment.bottomCenter,
                  child: Padding(
                    padding: const EdgeInsets.all(24),
                    child: FilledButton(
                      onPressed: () => setState(() => _scanning = false),
                      child: const Text('Cancel'),
                    ),
                  ),
                ),
              ],
            );
          }
          return Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Text('${widget.state.packCount} pack(s) ready to send',
                    style: Theme.of(context).textTheme.titleLarge),
                const SizedBox(height: 16),
                if (_busy) const CircularProgressIndicator(),
                if (!_busy)
                  FilledButton.icon(
                    onPressed: widget.state.packCount == 0
                        ? null
                        : () => setState(() {
                            _message = '';
                            _scanning = true;
                          }),
                    icon: const Icon(Icons.qr_code_scanner),
                    label: const Text('Scan desktop QR'),
                  ),
                const SizedBox(height: 16),
                if (_message.isNotEmpty) Text(_message),
              ],
            ),
          );
        },
      ),
    );
  }
}
