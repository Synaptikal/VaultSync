/// Base Repository Interface (PHASE 2 - Repository Pattern)
///
/// Defines the contract for all repositories in the application.
/// Repositories act as a single source of truth for data, abstracting
/// whether data comes from local storage or remote API.
///
/// Pattern:
/// ```
/// UI → Provider → Repository → [Remote + Local] Datasources
/// ```
///
/// Benefits:
/// - Single responsibility (data access)
/// - Testability (mock repositories)
/// - Offline-first by default
/// - Consistent error handling
///
/// Example Implementation:
/// ```dart
/// class ProductRepository implements Repository<Product> {
///   final ProductRemoteDataSource remote;
///   final ProductLocalDataSource local;
///
///   @override
///   Future<Product> create(Product product) async {
///     // 1. Save locally first (guaranteed)
///     await local.insert(product);
///
///     // 2. Try to sync to server
///     if (await connectivity.isOnline()) {
///       try {
///         final synced = await remote.create(product);
///         await local.markSynced(synced.id);
///         return synced;
///       } catch (e) {
///         // Already saved locally, queue for background sync
///         await syncQueue.add(product);
///       }
///     }
///
///     return product;
///   }
/// }
/// ```

/// Generic repository interface for CRUD operations
abstract class Repository<T> {
  /// Get all items (paginated if needed)
  Future<List<T>> getAll({int? limit, int? offset});

  /// Get a single item by ID
  Future<T?> getById(String id);

  /// Create a new item
  ///
  /// Saves locally first, then syncs to remote.
  /// Returns the item even if remote sync fails.
  Future<T> create(T item);

  /// Update an existing item
  ///
  /// Updates locally first, then syncs to remote.
  /// Returns the updated item even if remote sync fails.
  Future<T> update(T item);

  /// Delete an item
  ///
  /// Marks as deleted locally first, then syncs to remote.
  /// Uses soft delete to support offline sync.
  Future<void> delete(String id);

  /// Search items with a query
  Future<List<T>> search(String query);

  /// Refresh data from remote (pull)
  ///
  /// Forces a fetch from the server to get latest data.
  /// Updates local cache with fresh data.
  Future<List<T>> refresh();

  /// Sync pending local changes to remote (push)
  ///
  /// Processes the offline queue for this entity type.
  /// Returns number of items successfully synced.
  Future<int> syncPendingChanges();

  /// Clear local cache
  ///
  /// Use carefully - only for logout or data corruption scenarios.
  Future<void> clearCache();
}

/// Repository with filtering capabilities
abstract class FilterableRepository<T> extends Repository<T> {
  /// Get items filtered by category
  Future<List<T>> getByCategory(String category);

  /// Get items filtered by custom criteria
  Future<List<T>> getWhere(Map<String, dynamic> criteria);
}

/// Repository with sync status tracking
abstract class SyncableRepository<T> extends Repository<T> {
  /// Get items that haven't been synced to server yet
  Future<List<T>> getUnsynced();

  /// Get items that failed to sync
  Future<List<T>> getFailedSync();

  /// Retry syncing a specific item
  Future<void> retrySync(String id);
}

/// Repository result wrapper
///
/// Provides additional metadata about the operation
class RepositoryResult<T> {
  final T? data;
  final bool isFromCache;
  final DateTime? lastSyncTime;
  final String? errorMessage;
  final bool needsSync;

  const RepositoryResult({
    this.data,
    this.isFromCache = false,
    this.lastSyncTime,
    this.errorMessage,
    this.needsSync = false,
  });

  bool get isSuccess => data != null && errorMessage == null;
  bool get isError => errorMessage != null;
}

/// Pagination helper
class PaginationParams {
  final int limit;
  final int offset;

  const PaginationParams({
    this.limit = 50,
    this.offset = 0,
  });

  int get page => (offset / limit).floor();
}

/// Sort helper
enum SortOrder { asc, desc }

class SortParams {
  final String field;
  final SortOrder order;

  const SortParams({
    required this.field,
    this.order = SortOrder.asc,
  });
}
