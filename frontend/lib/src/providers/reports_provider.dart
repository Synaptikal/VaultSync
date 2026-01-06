import 'package:flutter/foundation.dart';
import '../services/api_service.dart';

/// ReportsProvider (TASK-AUD-004: Remediate FutureBuilder Anti-Pattern)
///
/// Manages report data state to avoid repeated API calls on UI rebuilds.
/// Supports caching and offline-safe data access where applicable.

class ReportsProvider with ChangeNotifier {
  final ApiService _apiService;

  ReportsProvider(this._apiService);

  // Cache for report data
  Map<String, dynamic>? _salesReport;
  Map<String, dynamic>? _inventoryValuation;
  Map<String, dynamic>? _topSellers;
  Map<String, dynamic>? _lowStock;

  bool _isLoading = false;
  bool get isLoading => _isLoading;

  String? _error;
  String? get error => _error;

  // Getters
  Map<String, dynamic>? get salesReport => _salesReport;
  Map<String, dynamic>? get inventoryValuation => _inventoryValuation;
  Map<String, dynamic>? get topSellers => _topSellers;
  Map<String, dynamic>? get lowStock => _lowStock;

  Future<void> loadSalesReport({DateTime? startDate, DateTime? endDate}) async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      _salesReport = await _apiService.getSalesReport(
        startDate: startDate,
        endDate: endDate,
      );
    } catch (e) {
      _error = e.toString();
      if (kDebugMode) print('Failed to load sales report: $e');
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<void> loadInventoryValuation() async {
    // If we have cached data, don't reload immediately unless forced
    // For now, simple aggressive loading is fine, but we avoid reload-on-build
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      _inventoryValuation = await _apiService.getInventoryValuation();
    } catch (e) {
      _error = e.toString();
      if (kDebugMode) print('Failed to load inventory valuation: $e');
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<void> loadTopSellers({int limit = 20}) async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      _topSellers = await _apiService.getTopSellers(limit: limit);
    } catch (e) {
      _error = e.toString();
      if (kDebugMode) print('Failed to load top sellers: $e');
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<void> loadLowStock({int threshold = 5}) async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      _lowStock = await _apiService.getLowStockReport(threshold: threshold);
    } catch (e) {
      _error = e.toString();
      if (kDebugMode) print('Failed to load low stock: $e');
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  void clearCache() {
    _salesReport = null;
    _inventoryValuation = null;
    _topSellers = null;
    _lowStock = null;
    notifyListeners();
  }
}
