import 'package:flutter/foundation.dart';
import '../services/api_service.dart';
import '../services/local_storage_service.dart';
import '../api/generated/models/customer.dart';

class CustomerProvider with ChangeNotifier {
  final ApiService _apiService;
  final LocalStorageService _localStorage = LocalStorageService();
  
  CustomerProvider(this._apiService);
  
  List<Customer> _customers = [];
  List<Customer> get customers => _customers;
  
  bool _isLoading = false;
  bool get isLoading => _isLoading;

  String? _error;
  String? get error => _error;

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
        for (var customer in _customers) {
          await _localStorage.saveCustomer(customer, isSynced: true);
        }
      } catch (e) {
        if (kDebugMode) print('Online fetch failed, loading from local DB: $e');
        _customers = await _localStorage.getCustomers();
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
          await _apiService.createCustomer(customer.toJson() as Map<String, dynamic>);
          await _localStorage.saveCustomer(customer, isSynced: true);
        } catch (e) {
           if (kDebugMode) print('Failed to sync customer ${customer.customerUuid}: $e');
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
}
