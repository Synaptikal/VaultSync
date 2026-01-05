import 'dart:async';
import 'package:flutter/foundation.dart';
import 'package:workmanager/workmanager.dart';
import 'package:connectivity_plus/connectivity_plus.dart';
import 'sync_queue_service.dart';
import 'database_service.dart';
import 'api_client.dart';
import 'storage_service.dart';

/// Background Sync Service (PHASE 3 - Background Worker)
///
/// Uses workmanager to process sync queue in the background.
/// Runs periodically and on connectivity changes.
///
/// Features:
/// - Periodic sync (every 15 minutes)
/// - Immediate sync on connectivity restoration
/// - Battery-aware scheduling
/// - Runs even when app is closed
///
/// Setup in main.dart:
/// ```dart
/// void main() {
///   WidgetsFlutterBinding.ensureInitialized();
///   Workmanager().initialize(callbackDispatcher);
///   BackgroundSyncService.initialize();
///   runApp(MyApp());
/// }
/// ```

/// Global callback dispatcher (must be top-level function)
@pragma('vm:entry-point')
void callbackDispatcher() {
  Workmanager().executeTask((task, inputData) async {
    debugPrint('[BackgroundSync] Task started: $task');

    try {
      // Initialize services
      final dbService = DatabaseService();
      final storage = SecureStorageService();
      final apiClient = ApiClient(storage: storage);
      final syncQueue = SyncQueueService(
        dbService: dbService,
        apiClient: apiClient,
      );

      // Check connectivity
      final connectivity = Connectivity();
      final result = await connectivity.checkConnectivity();

      if (result == ConnectivityResult.none) {
        debugPrint('[BackgroundSync] No connectivity, skipping sync');
        return Future.value(true);
      }

      // Process queue
      final (successCount, failureCount) = await syncQueue.processQueue();

      debugPrint(
          '[BackgroundSync] Sync complete: $successCount/$successCount+$failureCount');

      return Future.value(true);
    } catch (e) {
      debugPrint('[BackgroundSync] Error: $e');
      return Future.value(false);
    }
  });
}

class BackgroundSyncService {
  static const String _periodicSyncTask = 'periodic_sync';
  static const String _immediateSyncTask = 'immediate_sync';

  /// Initialize background sync
  static Future<void> initialize() async {
    await Workmanager().initialize(
      callbackDispatcher,
      isInDebugMode: false, // Set to true for debugging
    );

    // Register periodic sync (every 15 minutes)
    await registerPeriodicSync();

    // Listen for connectivity changes
    _listenToConnectivityChanges();
  }

  /// Register periodic background sync task
  static Future<void> registerPeriodicSync({
    Duration frequency = const Duration(minutes: 15),
  }) async {
    await Workmanager().registerPeriodicTask(
      _periodicSyncTask,
      _periodicSyncTask,
      frequency: frequency,
      constraints: Constraints(
        networkType: NetworkType.connected,
        requiresBatteryNotLow: true, // Don't drain battery
        requiresDeviceIdle: false,
      ),
      backoffPolicy: BackoffPolicy.exponential,
      backoffPolicyDelay: const Duration(minutes: 1),
    );

    debugPrint(
        '[BackgroundSync] Periodic sync registered (every ${frequency.inMinutes} min)');
  }

  /// Trigger immediate sync
  static Future<void> triggerImmediateSync() async {
    await Workmanager().registerOneOffTask(
      _immediateSyncTask,
      _immediateSyncTask,
      constraints: Constraints(
        networkType: NetworkType.connected,
      ),
    );

    debugPrint('[BackgroundSync] Immediate sync triggered');
  }

  /// Listen to connectivity changes and sync when coming online
  static void _listenToConnectivityChanges() {
    Connectivity().onConnectivityChanged.listen((result) {
      if (result != ConnectivityResult.none) {
        debugPrint('[BackgroundSync] Connectivity restored, triggering sync');
        triggerImmediateSync();
      }
    });
  }

  /// Cancel all background tasks
  static Future<void> cancelAll() async {
    await Workmanager().cancelAll();
    debugPrint('[BackgroundSync] All tasks cancelled');
  }

  /// Cancel periodic sync only
  static Future<void> cancelPeriodicSync() async {
    await Workmanager().cancelByUniqueName(_periodicSyncTask);
    debugPrint('[BackgroundSync] Periodic sync cancelled');
  }
}

/// Sync Status Notifier
///
/// Provides a stream of sync events for UI updates
class SyncStatusNotifier {
  static final SyncStatusNotifier _instance = SyncStatusNotifier._internal();
  factory SyncStatusNotifier() => _instance;
  SyncStatusNotifier._internal();

  final _controller = StreamController<SyncEvent>.broadcast();

  Stream<SyncEvent> get events => _controller.stream;

  void notify(SyncEvent event) {
    _controller.add(event);
  }

  void dispose() {
    _controller.close();
  }
}

/// Sync Event types
sealed class SyncEvent {
  final DateTime timestamp;
  SyncEvent() : timestamp = DateTime.now();
}

class SyncStarted extends SyncEvent {}

class SyncProgress extends SyncEvent {
  final int current;
  final int total;
  SyncProgress(this.current, this.total);
}

class SyncCompleted extends SyncEvent {
  final int successCount;
  final int failureCount;
  SyncCompleted(this.successCount, this.failureCount);
}

class SyncFailed extends SyncEvent {
  final String error;
  SyncFailed(this.error);
}
