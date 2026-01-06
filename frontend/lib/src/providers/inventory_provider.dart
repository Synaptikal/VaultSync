import 'package:flutter/foundation.dart';
import '../api/generated/export.dart';
import '../repositories/inventory_repository.dart';

/// Inventory Provider (TASK-AUD-001b)
///
/// State management for inventory data using the Repository pattern.
/// This is the correct way to access inventory data from UI.
///
/// Usage:
/// ```dart
/// // In your widget
/// final items = context.watch<InventoryProvider>().items;
///
/// // Trigger actions
/// context.read<InventoryProvider>().loadInventory();
/// ```

class InventoryProvider with ChangeNotifier {
  final InventoryRepository _repository;

  InventoryProvider(this._repository);

  List<InventoryItem> _items = [];
  List<InventoryItem> get items => _items;

  bool _isLoading = false;
  bool get isLoading => _isLoading;

  String? _error;
  String? get error => _error;

  bool _isOffline = false;
  bool get isOffline => _isOffline;

  Map<String, int> _syncStats = {};
  Map<String, int> get syncStats => _syncStats;

  /// Load all inventory items
  Future<void> loadInventory({int? limit, int? offset}) async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      _items = await _repository.getAll(limit: limit, offset: offset);
      _syncStats = await _repository.getSyncStats();
      _isOffline = false;
    } catch (e) {
      _error = e.toString();
      debugPrint('[InventoryProvider] Load failed: $e');
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  /// Load inventory for a specific product
  Future<void> loadByProduct(String productUuid) async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      _items = await _repository.getByProductUuid(productUuid);
    } catch (e) {
      _error = e.toString();
      debugPrint('[InventoryProvider] Load by product failed: $e');
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  /// Load low stock items
  Future<void> loadLowStock({int threshold = 3}) async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      _items = await _repository.getLowStock(threshold: threshold);
    } catch (e) {
      _error = e.toString();
      debugPrint('[InventoryProvider] Load low stock failed: $e');
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  /// Add new inventory item
  Future<InventoryItem?> addItem(InventoryItem item) async {
    _isLoading = true;
    notifyListeners();

    try {
      final created = await _repository.create(item);
      _items = [..._items, created];
      _syncStats = await _repository.getSyncStats();
      notifyListeners();
      return created;
    } catch (e) {
      _error = e.toString();
      debugPrint('[InventoryProvider] Add item failed: $e');
      notifyListeners();
      return null;
    } finally {
      _isLoading = false;
    }
  }

  /// Update existing inventory item
  Future<InventoryItem?> updateItem(InventoryItem item) async {
    _isLoading = true;
    notifyListeners();

    try {
      final updated = await _repository.update(item);
      final index =
          _items.indexWhere((i) => i.inventoryUuid == item.inventoryUuid);
      if (index != -1) {
        _items = [..._items];
        _items[index] = updated;
      }
      _syncStats = await _repository.getSyncStats();
      notifyListeners();
      return updated;
    } catch (e) {
      _error = e.toString();
      debugPrint('[InventoryProvider] Update item failed: $e');
      notifyListeners();
      return null;
    } finally {
      _isLoading = false;
    }
  }

  /// Delete inventory item
  Future<bool> deleteItem(String inventoryUuid) async {
    _isLoading = true;
    notifyListeners();

    try {
      await _repository.delete(inventoryUuid);
      _items = _items.where((i) => i.inventoryUuid != inventoryUuid).toList();
      _syncStats = await _repository.getSyncStats();
      notifyListeners();
      return true;
    } catch (e) {
      _error = e.toString();
      debugPrint('[InventoryProvider] Delete item failed: $e');
      notifyListeners();
      return false;
    } finally {
      _isLoading = false;
    }
  }

  /// Force refresh from server
  Future<void> refresh() async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      _items = await _repository.refresh();
      _syncStats = await _repository.getSyncStats();
      _isOffline = false;
    } catch (e) {
      _error = e.toString();
      _isOffline = true;
      debugPrint('[InventoryProvider] Refresh failed (offline?): $e');
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  /// Sync pending changes to server
  Future<int> syncPending() async {
    try {
      final synced = await _repository.syncPendingChanges();
      _syncStats = await _repository.getSyncStats();
      notifyListeners();
      return synced;
    } catch (e) {
      debugPrint('[InventoryProvider] Sync pending failed: $e');
      return 0;
    }
  }

  /// Get a single item by ID
  Future<InventoryItem?> getById(String inventoryUuid) async {
    // Check local list first
    final local =
        _items.where((i) => i.inventoryUuid == inventoryUuid).firstOrNull;
    if (local != null) return local;

    // Fetch from repository
    return await _repository.getById(inventoryUuid);
  }

  /// Clear error state
  void clearError() {
    _error = null;
    notifyListeners();
  }
}
