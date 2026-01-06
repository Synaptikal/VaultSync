import 'package:flutter/foundation.dart';
import '../services/api_service.dart';

/// PricingProvider (TASK-AUD-004: Anti-Pattern Remediation)
///
/// Manages pricing data state to avoid repeated API calls.
/// Supports offline-first by caching the last fetched dashboard data.

class PricingProvider with ChangeNotifier {
  final ApiService _apiService;

  PricingProvider(this._apiService);

  Map<String, dynamic>? _dashboardData;
  Map<String, dynamic>? get dashboardData => _dashboardData;

  bool _isLoading = false;
  bool get isLoading => _isLoading;

  bool _isOffline = false;
  bool get isOffline => _isOffline;

  String? _error;
  String? get error => _error;

  Future<void> loadPricingDashboard() async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      _dashboardData = await _apiService.getPricingDashboard();
      _isOffline = false;
    } catch (e) {
      _error = e.toString();
      _isOffline = true; // Assume offline on error for now, or check error type
      if (kDebugMode) print('Failed to load pricing dashboard: $e');
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<void> triggerPriceSync() async {
    _isLoading = true;
    notifyListeners();

    try {
      await _apiService.triggerPriceSync();
      // Reload dashboard to reflect any immediate changes
      await loadPricingDashboard();
    } catch (e) {
      _error = e.toString();
      if (kDebugMode) print('Failed to trigger price sync: $e');
      rethrow;
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }
}
