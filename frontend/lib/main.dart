import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'src/app.dart';
import 'src/providers/product_provider.dart';
import 'src/providers/customer_provider.dart';
import 'src/providers/cart_provider.dart';
import 'src/providers/sync_provider.dart';
import 'src/providers/events_provider.dart';
import 'src/providers/wants_provider.dart';
import 'src/services/api_service.dart';
import 'src/services/offline_sync_service.dart';

void main() {
  final apiService = ApiService();

  runApp(
    MultiProvider(
      providers: [
        Provider<ApiService>.value(value: apiService),
        ChangeNotifierProvider(create: (_) => ProductProvider(apiService)),
        ChangeNotifierProvider(create: (_) => CustomerProvider(apiService)),
        ChangeNotifierProvider(create: (_) => CartProvider(apiService)),
        ChangeNotifierProvider(create: (_) => SyncProvider(apiService: apiService)),
        ChangeNotifierProvider(create: (_) => EventsProvider(apiService)),
        ChangeNotifierProvider(create: (_) => WantsProvider(apiService)),
        // Offline-first sync service
        ChangeNotifierProvider(create: (_) => OfflineSyncService(apiService: apiService)),
      ],
      child: const VaultSyncApp(),
    ),
  );
}
