import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'src/app.dart';
import 'src/providers/product_provider.dart';
import 'src/providers/customer_provider.dart';
import 'src/providers/cart_provider.dart';
import 'src/providers/sync_provider.dart';
import 'src/providers/events_provider.dart';
import 'src/providers/wants_provider.dart';
import 'src/providers/inventory_provider.dart';
import 'src/providers/transaction_provider.dart';
import 'src/providers/reports_provider.dart';
import 'src/providers/pricing_provider.dart';
import 'src/services/api_service.dart';
import 'src/services/api_client.dart';
import 'src/services/database_service.dart';
import 'src/services/offline_sync_service.dart';
import 'src/datasources/local/inventory_local_datasource.dart';
import 'src/datasources/remote/inventory_remote_datasource.dart';
import 'src/repositories/inventory_repository.dart';

void main() {
  // Services
  final apiService = ApiService();
  final apiClient = ApiClient();
  final dbService = DatabaseService();

  // Datasources
  final inventoryLocalDs = InventoryLocalDataSource(dbService: dbService);
  final inventoryRemoteDs = InventoryRemoteDataSource(apiClient: apiClient);

  // Repositories
  final inventoryRepo = InventoryRepository(
    local: inventoryLocalDs,
    remote: inventoryRemoteDs,
  );

  runApp(
    MultiProvider(
      providers: [
        // Services
        Provider<ApiService>.value(value: apiService),
        Provider<ApiClient>.value(value: apiClient),
        Provider<DatabaseService>.value(value: dbService),

        // Legacy Providers (TODO: Migrate to Repository pattern)
        ChangeNotifierProvider(create: (_) => ProductProvider(apiService)),
        ChangeNotifierProvider(create: (_) => CustomerProvider(apiService)),
        ChangeNotifierProvider(create: (_) => CartProvider(apiService)),
        ChangeNotifierProvider(
            create: (_) => SyncProvider(apiService: apiService)),
        ChangeNotifierProvider(create: (_) => EventsProvider(apiService)),
        ChangeNotifierProvider(create: (_) => WantsProvider(apiService)),

        // New Repository-Pattern Providers (TASK-AUD-001)
        ChangeNotifierProvider(create: (_) => InventoryProvider(inventoryRepo)),

        // Offline-first sync service
        ChangeNotifierProvider(
            create: (_) => OfflineSyncService(apiService: apiService)),

        // Transaction Provider (TASK-AUD-001k)
        ChangeNotifierProvider(create: (_) => TransactionProvider(apiService)),

        // Reports Provider (TASK-AUD-004)
        ChangeNotifierProvider(create: (_) => ReportsProvider(apiService)),

        // Pricing Provider (TASK-AUD-004)
        ChangeNotifierProvider(create: (_) => PricingProvider(apiService)),
      ],
      child: const VaultSyncApp(),
    ),
  );
}
