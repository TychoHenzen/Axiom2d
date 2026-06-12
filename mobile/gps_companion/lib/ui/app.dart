/// App shell: MaterialApp + bottom navigation across Map / Gallery / Transfer.
library;

import 'package:flutter/material.dart';
import 'package:wakelock_plus/wakelock_plus.dart';

import 'app_state.dart';
import 'gallery_screen.dart';
import 'map_screen.dart';
import 'transfer_screen.dart';

class GpsCompanionApp extends StatelessWidget {
  const GpsCompanionApp({super.key, required this.state});

  final AppState state;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'GPS Companion',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.indigo),
        useMaterial3: true,
      ),
      home: HomeShell(state: state),
    );
  }
}

class HomeShell extends StatefulWidget {
  const HomeShell({super.key, required this.state});

  final AppState state;

  @override
  State<HomeShell> createState() => _HomeShellState();
}

class _HomeShellState extends State<HomeShell> {
  int _index = 0;

  @override
  Widget build(BuildContext context) {
    final screens = [
      MapScreen(state: widget.state),
      GalleryScreen(state: widget.state),
      TransferScreen(state: widget.state),
    ];
    return Scaffold(
      body: IndexedStack(index: _index, children: screens),
      bottomNavigationBar: NavigationBar(
        selectedIndex: _index,
        onDestinationSelected: (i) => setState(() {
          _index = i;
          if (i == 0) {
            WakelockPlus.enable();
          } else {
            WakelockPlus.disable();
          }
        }),
        destinations: const [
          NavigationDestination(icon: Icon(Icons.map), label: 'Map'),
          NavigationDestination(icon: Icon(Icons.inventory_2), label: 'Packs'),
          NavigationDestination(icon: Icon(Icons.qr_code_scanner), label: 'Transfer'),
        ],
      ),
    );
  }
}
