import 'dart:convert';
import 'package:sqflite/sqflite.dart';
import 'database_service.dart';
import 'api_client.dart';
import 'api_exceptions.dart';

/// Sync Queue Service (PHASE 3 - Offline Queue)
///
/// Manages offline operations that need to be synced to the server.
/// Provides automatic retry with exponential backoff.
///
/// Flow:
/// 1. User performs action while offline
/// 2. Repository saves locally + adds to sync queue
/// 3. Background worker/connectivity change triggers processQueue()
/// 4. Queue items are synced with retry logic
/// 5. Successfully synced items are removed from queue
///
/// Queue Entry Schema:
/// ```sql
/// CREATE TABLE sync_queue (
///   id INTEGER PRIMARY KEY,
///   entity_type TEXT,      -- 'Product', 'Inventory', etc.
///   entity_uuid TEXT,
///   operation TEXT,        -- 'CREATE', 'UPDATE', 'DELETE'
///   payload TEXT,          -- JSON data
///   attempts INTEGER,
///   last_error TEXT,
///   created_at TEXT,
///   updated_at TEXT
/// )
/// ```

enum SyncOperation { create, update, delete }

class SyncQueueEntry {
  final int? id;
  final String entityType;
  final String entityUuid;
  final SyncOperation operation;
  final Map<String, dynamic> payload;
  final int attempts;
  final String? lastError;
  final DateTime createdAt;
  final DateTime updatedAt;

  SyncQueueEntry({
    this.id,
    required this.entityType,
    required this.entityUuid,
    required this.operation,
    required this.payload,
    this.attempts = 0,
    this.lastError,
    DateTime? createdAt,
    DateTime? updatedAt,
  })  : createdAt = createdAt ?? DateTime.now(),
        updatedAt = updatedAt ?? DateTime.now();

  Map<String, dynamic> toMap() {
    return {
      if (id != null) 'id': id,
      'entity_type': entityType,
      'entity_uuid': entityUuid,
      'operation': operation.name.toUpperCase(),
      'payload': jsonEncode(payload),
      'attempts': attempts,
      'last_error': lastError,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt.toIso8601String(),
    };
  }

  factory SyncQueueEntry.fromMap(Map<String, dynamic> map) {
    return SyncQueueEntry(
      id: map['id'] as int,
      entityType: map['entity_type'] as String,
      entityUuid: map['entity_uuid'] as String,
      operation: SyncOperation.values.firstWhere(
        (e) => e.name.toUpperCase() == map['operation'],
      ),
      payload: jsonDecode(map['payload'] as String) as Map<String, dynamic>,
      attempts: map['attempts'] as int,
      lastError: map['last_error'] as String?,
      createdAt: DateTime.parse(map['created_at'] as String),
      updatedAt: DateTime.parse(map['updated_at'] as String),
    );
  }

  /// Should we retry this entry?
  bool shouldRetry({int maxAttempts = 5}) {
    return attempts < maxAttempts;
  }

  /// Calculate backoff delay (exponential)
  Duration get backoffDelay {
    // 1s, 2s, 4s, 8s, 16s, 32s...
    final seconds = [1, 2, 4, 8, 16, 32];
    final index = attempts.clamp(0, seconds.length - 1);
    return Duration(seconds: seconds[index]);
  }
}

class SyncQueueService {
  final DatabaseService _dbService;
  final ApiClient _apiClient;

  SyncQueueService({
    required DatabaseService dbService,
    required ApiClient apiClient,
  })  : _dbService = dbService,
        _apiClient = apiClient;

  /// Add an operation to the sync queue
  Future<void> enqueue(SyncQueueEntry entry) async {
    final db = await _dbService.database;

    // Check if already queued (avoid duplicates)
    final existing = await db.query(
      'sync_queue',
      where: 'entity_type = ? AND entity_uuid = ? AND operation = ?',
      whereArgs: [
        entry.entityType,
        entry.entityUuid,
        entry.operation.name.toUpperCase()
      ],
    );

    if (existing.isEmpty) {
      await db.insert('sync_queue', entry.toMap());
      print(
          '[SyncQueue] Queued ${entry.operation} for ${entry.entityType}:${entry.entityUuid}');
    } else {
      // Update existing entry with new payload
      await db.update(
        'sync_queue',
        entry.toMap(),
        where: 'id = ?',
        whereArgs: [existing.first['id']],
      );
      print(
          '[SyncQueue] Updated queue entry for ${entry.entityType}:${entry.entityUuid}');
    }
  }

  /// Get all pending sync operations
  Future<List<SyncQueueEntry>> getPending() async {
    final db = await _dbService.database;
    final maps = await db.query(
      'sync_queue',
      orderBy: 'created_at ASC',
    );

    return maps.map((map) => SyncQueueEntry.fromMap(map)).toList();
  }

  /// Get queue count
  Future<int> getCount() async {
    final db = await _dbService.database;
    final result =
        await db.rawQuery('SELECT COUNT(*) as count FROM sync_queue');
    return Sqflite.firstIntValue(result) ?? 0;
  }

  /// Process the entire queue
  ///
  /// Returns: (successCount, failureCount)
  Future<(int, int)> processQueue() async {
    final pending = await getPending();
    int successCount = 0;
    int failureCount = 0;

    print('[SyncQueue] Processing ${pending.length} pending operations');

    for (final entry in pending) {
      // Check if should retry (backoff logic)
      if (!entry.shouldRetry()) {
        print(
            '[SyncQueue] Max retries exceeded for ${entry.entityType}:${entry.entityUuid}');
        // Move to failed items table or notify user
        await _markAsFailed(entry);
        failureCount++;
        continue;
      }

      // Attempt to sync
      final success = await _syncEntry(entry);
      if (success) {
        await _removeFromQueue(entry.id!);
        successCount++;
      } else {
        await _incrementAttempts(entry.id!);
        failureCount++;
      }

      // Small delay between operations to avoid overwhelming server
      await Future.delayed(const Duration(milliseconds: 100));
    }

    print(
        '[SyncQueue] Sync complete: $successCount succeeded, $failureCount failed');
    return (successCount, failureCount);
  }

  /// Sync a single queue entry
  Future<bool> _syncEntry(SyncQueueEntry entry) async {
    try {
      switch (entry.entityType) {
        case 'Product':
          return await _syncProduct(entry);
        case 'Inventory':
          return await _syncInventory(entry);
        case 'Transaction':
          return await _syncTransaction(entry);
        default:
          print('[SyncQueue] Unknown entity type: ${entry.entityType}');
          return false;
      }
    } on ApiException catch (e) {
      await _updateError(entry.id!, e.message);

      // Don't retry on validation errors (they'll always fail)
      if (e is ValidationException) {
        await _markAsFailed(entry);
      }

      return false;
    } catch (e) {
      await _updateError(entry.id!, e.toString());
      return false;
    }
  }

  /// Sync a product operation
  Future<bool> _syncProduct(SyncQueueEntry entry) async {
    switch (entry.operation) {
      case SyncOperation.create:
        await _apiClient.post('/api/products', data: entry.payload);
        return true;

      case SyncOperation.update:
        await _apiClient.put(
          '/api/products/${entry.entityUuid}',
          data: entry.payload,
        );
        return true;

      case SyncOperation.delete:
        await _apiClient.delete('/api/products/${entry.entityUuid}');
        return true;
    }
  }

  /// Sync an inventory operation
  Future<bool> _syncInventory(SyncQueueEntry entry) async {
    switch (entry.operation) {
      case SyncOperation.create:
        await _apiClient.post('/api/inventory', data: entry.payload);
        return true;

      case SyncOperation.update:
        await _apiClient.put(
          '/api/inventory/${entry.entityUuid}',
          data: entry.payload,
        );
        return true;

      case SyncOperation.delete:
        await _apiClient.delete('/api/inventory/${entry.entityUuid}');
        return true;
    }
  }

  /// Sync a transaction operation
  Future<bool> _syncTransaction(SyncQueueEntry entry) async {
    // Transactions are typically create-only
    if (entry.operation == SyncOperation.create) {
      await _apiClient.post('/api/transactions', data: entry.payload);
      return true;
    }
    return false;
  }

  /// Remove successfully synced entry
  Future<void> _removeFromQueue(int id) async {
    final db = await _dbService.database;
    await db.delete('sync_queue', where: 'id = ?', whereArgs: [id]);
  }

  /// Increment attempt counter
  Future<void> _incrementAttempts(int id) async {
    final db = await _dbService.database;
    await db.rawUpdate(
      'UPDATE sync_queue SET attempts = attempts + 1, updated_at = ? WHERE id = ?',
      [DateTime.now().toIso8601String(), id],
    );
  }

  /// Update error message
  Future<void> _updateError(int id, String error) async {
    final db = await _dbService.database;
    await db.update(
      'sync_queue',
      {
        'last_error': error,
        'updated_at': DateTime.now().toIso8601String(),
      },
      where: 'id = ?',
      whereArgs: [id],
    );
  }

  /// Mark entry as permanently failed
  Future<void> _markAsFailed(SyncQueueEntry entry) async {
    // TODO: Move to failed_sync_items table for user review
    // For now, just log
    print(
        '[SyncQueue] PERMANENT FAILURE: ${entry.entityType}:${entry.entityUuid} - ${entry.lastError}');
    await _removeFromQueue(entry.id!);
  }

  /// Clear all queue entries (use carefully)
  Future<void> clearQueue() async {
    final db = await _dbService.database;
    await db.delete('sync_queue');
  }

  /// Get failed items (exceeded retry limit)
  Future<List<SyncQueueEntry>> getFailedItems() async {
    final db = await _dbService.database;
    final maps = await db.query(
      'sync_queue',
      where: 'attempts >= ?',
      whereArgs: [5], // Max attempts
    );

    return maps.map((map) => SyncQueueEntry.fromMap(map)).toList();
  }

  /// Retry a specific item
  Future<bool> retryItem(int id) async {
    final db = await _dbService.database;
    final maps = await db.query('sync_queue', where: 'id = ?', whereArgs: [id]);

    if (maps.isEmpty) return false;

    final entry = SyncQueueEntry.fromMap(maps.first);
    final success = await _syncEntry(entry);

    if (success) {
      await _removeFromQueue(id);
    }

    return success;
  }
}
