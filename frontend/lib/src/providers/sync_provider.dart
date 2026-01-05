import 'package:flutter/foundation.dart';
import '../services/api_service.dart';
import '../services/local_storage_service.dart';
import '../models/sync_models.dart';

class SyncProvider with ChangeNotifier {
  final ApiService _apiService;
  final LocalStorageService _localStorageService;

  bool _isSyncing = false;
  String? _lastError;
  DateTime? _lastSyncTime;

  bool get isSyncing => _isSyncing;
  String? get lastError => _lastError;
  DateTime? get lastSyncTime => _lastSyncTime;

  SyncProvider({
    ApiService? apiService,
    LocalStorageService? localStorageService,
  })  : _apiService = apiService ?? ApiService(),
        _localStorageService = localStorageService ?? LocalStorageService();

  int _lastSyncVersion = 0; // Tracks the last vector clock we synced from

  Future<void> sync() async {
    if (_isSyncing) return;

    _isSyncing = true;
    _lastError = null;
    notifyListeners();

    try {
      // 1. Get pending local changes
      final pendingChangesMap =
          await _localStorageService.getPendingSyncChanges();

      if (pendingChangesMap.isNotEmpty) {
        // 2. Convert to ChangeRecord objects
        final changes = pendingChangesMap.map((map) {
          return ChangeRecord(
            recordId: map['record_id'],
            recordType: RecordType.values.firstWhere(
              (e) =>
                  e.name.toLowerCase() ==
                  (map['record_type'] as String).toLowerCase(),
              orElse: () => RecordType.product,
            ),
            operation: SyncOperation.values.firstWhere(
              (e) =>
                  e.name.toLowerCase() ==
                  (map['operation'] as String).toLowerCase(),
              orElse: () => SyncOperation.update,
            ),
            data: map['data'],
            vectorTimestamp: VectorTimestamp(
              entries: {'mobile_device': DateTime.now().millisecondsSinceEpoch},
            ),
            timestamp: DateTime.parse(map['timestamp']),
          );
        }).toList();

        // 3. Push to backend
        await _apiService.pushSyncChanges(changes);

        // 4. Clear local sync log for pushed records
        for (var change in changes) {
          await _localStorageService.clearSyncLog(change.recordId);
        }
      }

      // 5. Pull from backend
      final remoteChanges = await _apiService.pullSyncChanges(_lastSyncVersion);

      if (remoteChanges.isNotEmpty) {
        await _localStorageService.applyRemoteChanges(remoteChanges);

        // Update last sync version to the max sequence number received
        _lastSyncVersion = remoteChanges
            .map((c) => c.sequenceNumber ?? 0)
            .fold(_lastSyncVersion, (max, curr) => curr > max ? curr : max);
      }

      _lastSyncTime = DateTime.now();
    } catch (e) {
      _lastError = e.toString();
      debugPrint('Sync error: $e');
    } finally {
      _isSyncing = false;
      notifyListeners();
    }
  }
}
