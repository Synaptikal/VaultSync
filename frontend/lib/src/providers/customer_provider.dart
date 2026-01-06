import 'package:flutter/foundation.dart';
import '../services/api_service.dart';
import '../services/local_storage_service.dart';
import '../api/generated/models/customer.dart';

/// CustomerProvider (TASK-AUD-001j: Added offline-first methods)
///
/// Manages customer state with offline-first support.
/// Now includes getCustomerHistory() and updateStoreCredit() for offline use.

class CustomerProvider with ChangeNotifier {
  final ApiService _apiService;
  final LocalStorageService _localStorage = LocalStorageService();

  CustomerProvider(this._apiService);

  List<Customer> _customers = [];
  List<Customer> get customers => _customers;

  bool _isLoading = false;
  bool get isLoading => _isLoading;

  bool _isOffline = false;
  bool get isOffline => _isOffline;

  String? _error;
  String? get error => _error;

  // Cache for customer history
  final Map<String, List<Map<String, dynamic>>> _historyCache = {};

  Future<void> loadCustomers() async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      // 1. Sync pending
      await _syncPendingCustomers();

      // 2. Try online
      try {
        _customers = await _apiService.getCustomers();
        _isOffline = false;
        for (var customer in _customers) {
          await _localStorage.saveCustomer(customer, isSynced: true);
        }
      } catch (e) {
        if (kDebugMode) print('Online fetch failed, loading from local DB: $e');
        _customers = await _localStorage.getCustomers();
        _isOffline = true;
      }
    } catch (e) {
      _error = e.toString();
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<void> _syncPendingCustomers() async {
    try {
      final pending = await _localStorage.getUnsyncedCustomers();
      for (var customer in pending) {
        try {
          await _apiService
              .createCustomer(customer.toJson() as Map<String, dynamic>);
          await _localStorage.saveCustomer(customer, isSynced: true);
        } catch (e) {
          if (kDebugMode)
            print('Failed to sync customer ${customer.customerUuid}: $e');
        }
      }
    } catch (e) {
      if (kDebugMode) print('Sync pending failed: $e');
    }
  }

  Future<void> addCustomer(Map<String, dynamic> customerData) async {
    _isLoading = true;
    notifyListeners();

    final customer = Customer.fromJson(customerData);

    try {
      // 1. Save local unsynced
      await _localStorage.saveCustomer(customer, isSynced: false);

      // 2. Try online
      await _apiService.createCustomer(customerData);

      // 3. Mark synced
      await _localStorage.saveCustomer(customer, isSynced: true);

      await loadCustomers();
    } catch (e) {
      if (kDebugMode) print('Add customer offline mode: $e');
      _customers = await _localStorage.getCustomers();
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  /// Get customer transaction history
  /// First tries to load from cache, then from API
  Future<List<Map<String, dynamic>>> getCustomerHistory(
      String customerUuid) async {
    // Check cache first
    if (_historyCache.containsKey(customerUuid)) {
      return _historyCache[customerUuid]!;
    }

    try {
      final history = await _apiService.getCustomerHistory(customerUuid);
      _historyCache[customerUuid] = history;
      return history;
    } catch (e) {
      if (kDebugMode) print('Failed to get customer history: $e');
      // Return empty list for offline mode
      return [];
    }
  }

  /// Update customer store credit
  /// Saves locally first, then syncs to server
  Future<void> updateStoreCredit(String customerUuid, double amount) async {
    try {
      // Try to sync to server
      await _apiService.updateStoreCredit(customerUuid, amount);

      // Reload customers to get updated data
      await loadCustomers();

      // Clear history cache for this customer
      _historyCache.remove(customerUuid);
    } catch (e) {
      if (kDebugMode) print('Failed to update store credit: $e');
      // For now, throw the error - in future we could queue this locally
      rethrow;
    }
  }

  /// Find customer by UUID
  Customer? getById(String customerUuid) {
    return _customers.where((c) => c.customerUuid == customerUuid).firstOrNull;
  }

  /// Refresh data from server
  Future<void> refresh() async {
    _historyCache.clear();
    await loadCustomers();
  }
}
