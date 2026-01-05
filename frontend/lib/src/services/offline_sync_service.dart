import 'dart:async';
import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import '../services/api_service.dart';
import '../services/local_storage_service.dart';
import '../models/sync_models.dart';

/// Manages offline-first data synchronization
///
/// This service:
/// - Monitors network connectivity
/// - Queues changes when offline
/// - Syncs automatically when online
/// - Handles conflicts with last-write-wins or custom resolution
class OfflineSyncService with ChangeNotifier {
  final ApiService _apiService;
  final LocalStorageService _localStorage;

  // State
  bool _isOnline = true;
  bool _isSyncing = false;
  int _pendingChangesCount = 0;
  DateTime? _lastSuccessfulSync;
  String? _lastError;

  // Timers
  Timer? _connectivityCheckTimer;
  Timer? _autoSyncTimer;

  // Config
  static const Duration _connectivityCheckInterval = Duration(seconds: 30);
  static const Duration _autoSyncInterval = Duration(minutes: 5);
  static const String _lastSyncVersionKey = 'last_sync_version';
  static const String _lastSyncTimeKey = 'last_sync_time';

  // Getters
  bool get isOnline => _isOnline;
  bool get isSyncing => _isSyncing;
  int get pendingChangesCount => _pendingChangesCount;
  DateTime? get lastSuccessfulSync => _lastSuccessfulSync;
  String? get lastError => _lastError;
  bool get hasPendingChanges => _pendingChangesCount > 0;

  OfflineSyncService({
    ApiService? apiService,
    LocalStorageService? localStorage,
  })  : _apiService = apiService ?? ApiService(),
        _localStorage = localStorage ?? LocalStorageService() {
    _initialize();
  }

  Future<void> _initialize() async {
    // Load last sync info from preferences
    final prefs = await SharedPreferences.getInstance();
    final lastSyncTimeStr = prefs.getString(_lastSyncTimeKey);
    if (lastSyncTimeStr != null) {
      _lastSuccessfulSync = DateTime.tryParse(lastSyncTimeStr);
    }

    // Check initial connectivity
    await _checkConnectivity();

    // Update pending changes count
    await _updatePendingCount();

    // Start timers
    _startConnectivityMonitoring();
    _startAutoSync();

    notifyListeners();
  }

  void _startConnectivityMonitoring() {
    _connectivityCheckTimer?.cancel();
    _connectivityCheckTimer = Timer.periodic(_connectivityCheckInterval, (_) {
      _checkConnectivity();
    });
  }

  void _startAutoSync() {
    _autoSyncTimer?.cancel();
    _autoSyncTimer = Timer.periodic(_autoSyncInterval, (_) {
      if (_isOnline && _pendingChangesCount > 0) {
        syncNow();
      }
    });
  }

  Future<void> _checkConnectivity() async {
    final wasOnline = _isOnline;

    try {
      // Try to reach the API server
      final result = await InternetAddress.lookup('localhost')
          .timeout(const Duration(seconds: 5));
      _isOnline = result.isNotEmpty && result[0].rawAddress.isNotEmpty;

      // Also try to ping the API
      if (_isOnline) {
        try {
          await _apiService.healthCheck();
        } catch (e) {
          _isOnline = false;
        }
      }
    } catch (e) {
      _isOnline = false;
    }

    // If we just came online and have pending changes, trigger sync
    if (!wasOnline && _isOnline && _pendingChangesCount > 0) {
      debugPrint('📶 Back online with pending changes - triggering sync');
      syncNow();
    }

    if (wasOnline != _isOnline) {
      notifyListeners();
    }
  }

  Future<void> _updatePendingCount() async {
    final pending = await _localStorage.getPendingSyncChanges();
    _pendingChangesCount = pending.length;
    notifyListeners();
  }

  /// Queue a local change for later sync
  Future<void> queueChange({
    required String recordId,
    required String recordType,
    required String operation,
    required Map<String, dynamic> data,
  }) async {
    await _localStorage.logSyncChange(recordId, recordType, operation, data);
    await _updatePendingCount();

    // If online, try to sync immediately
    if (_isOnline) {
      syncNow();
    }
  }

  /// Force an immediate sync
  Future<SyncResult> syncNow() async {
    if (_isSyncing) {
      return SyncResult(success: false, message: 'Sync already in progress');
    }

    if (!_isOnline) {
      return SyncResult(success: false, message: 'Device is offline');
    }

    _isSyncing = true;
    _lastError = null;
    notifyListeners();

    int pushed = 0;
    int pulled = 0;

    try {
      final prefs = await SharedPreferences.getInstance();
      int lastSyncVersion = prefs.getInt(_lastSyncVersionKey) ?? 0;

      // --- PUSH local changes ---
      final pendingChanges = await _localStorage.getPendingSyncChanges();

      if (pendingChanges.isNotEmpty) {
        debugPrint('📤 Pushing ${pendingChanges.length} local changes...');

        final changes = pendingChanges
            .map((map) => ChangeRecord(
                  recordId: map['record_id'],
                  recordType: _parseRecordType(map['record_type']),
                  operation: _parseOperation(map['operation']),
                  data: map['data'],
                  vectorTimestamp: VectorTimestamp(
                    entries: {'mobile': DateTime.now().millisecondsSinceEpoch},
                  ),
                  timestamp: DateTime.parse(map['timestamp']),
                ))
            .toList();

        await _apiService.pushSyncChanges(changes);

        // Clear pushed changes from local queue
        for (var change in changes) {
          await _localStorage.clearSyncLog(change.recordId);
        }

        pushed = changes.length;
      }

      // --- PULL remote changes ---
      debugPrint('📥 Pulling changes since version $lastSyncVersion...');
      final remoteChanges = await _apiService.pullSyncChanges(lastSyncVersion);

      if (remoteChanges.isNotEmpty) {
        debugPrint('📥 Received ${remoteChanges.length} remote changes');

        // Apply changes with conflict detection
        await _applyRemoteChangesWithConflictResolution(remoteChanges);

        // Update sync version
        lastSyncVersion = remoteChanges
            .map((c) => c.sequenceNumber ?? 0)
            .fold(lastSyncVersion, (max, curr) => curr > max ? curr : max);

        await prefs.setInt(_lastSyncVersionKey, lastSyncVersion);
        pulled = remoteChanges.length;
      }

      // Update state
      _lastSuccessfulSync = DateTime.now();
      await prefs.setString(
          _lastSyncTimeKey, _lastSuccessfulSync!.toIso8601String());
      await _updatePendingCount();

      debugPrint('✅ Sync complete: $pushed pushed, $pulled pulled');

      return SyncResult(
        success: true,
        message: 'Sync complete',
        pushedCount: pushed,
        pulledCount: pulled,
      );
    } catch (e) {
      _lastError = e.toString();
      debugPrint('❌ Sync error: $e');

      // If we lost connectivity, update status
      if (e.toString().contains('SocketException') ||
          e.toString().contains('Connection refused')) {
        _isOnline = false;
      }

      return SyncResult(success: false, message: e.toString());
    } finally {
      _isSyncing = false;
      notifyListeners();
    }
  }

  Future<void> _applyRemoteChangesWithConflictResolution(
      List<ChangeRecord> changes) async {
    // Group by record type for batch processing
    final byType = <RecordType, List<ChangeRecord>>{};
    for (var change in changes) {
      byType.putIfAbsent(change.recordType, () => []).add(change);
    }

    // Check for conflicts (local changes to same records)
    final pendingLocal = await _localStorage.getPendingSyncChanges();
    final localRecordIds =
        pendingLocal.map((m) => m['record_id'] as String).toSet();

    for (var change in changes) {
      if (localRecordIds.contains(change.recordId)) {
        // Conflict! Use last-write-wins (remote wins since it was committed first)
        debugPrint(
            '⚠️ Conflict detected for ${change.recordId} - using server version');
        // Remove local pending change
        await _localStorage.clearSyncLog(change.recordId);
      }
    }

    // Apply remote changes
    await _localStorage.applyRemoteChanges(changes);
  }

  RecordType _parseRecordType(String type) {
    return RecordType.values.firstWhere(
      (e) => e.name.toLowerCase() == type.toLowerCase(),
      orElse: () => RecordType.product,
    );
  }

  SyncOperation _parseOperation(String op) {
    return SyncOperation.values.firstWhere(
      (e) => e.name.toLowerCase() == op.toLowerCase(),
      orElse: () => SyncOperation.update,
    );
  }

  /// Reset sync state (useful for logout or troubleshooting)
  Future<void> resetSyncState() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_lastSyncVersionKey);
    await prefs.remove(_lastSyncTimeKey);
    _lastSuccessfulSync = null;
    _pendingChangesCount = 0;
    notifyListeners();
  }

  @override
  void dispose() {
    _connectivityCheckTimer?.cancel();
    _autoSyncTimer?.cancel();
    super.dispose();
  }
}

/// Result of a sync operation
class SyncResult {
  final bool success;
  final String message;
  final int pushedCount;
  final int pulledCount;

  SyncResult({
    required this.success,
    required this.message,
    this.pushedCount = 0,
    this.pulledCount = 0,
  });
}
