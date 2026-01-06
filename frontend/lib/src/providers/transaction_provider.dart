import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart' show DateTimeRange;
import '../services/api_service.dart';

/// TransactionProvider (TASK-AUD-001k: Created for offline-first transactions)
///
/// Manages transaction history state with caching.
/// Provides filtered access to transaction data.

class TransactionProvider with ChangeNotifier {
  final ApiService _apiService;

  TransactionProvider(this._apiService);

  List<Map<String, dynamic>> _transactions = [];
  List<Map<String, dynamic>> get transactions => _transactions;

  bool _isLoading = false;
  bool get isLoading => _isLoading;

  bool _isOffline = false;
  bool get isOffline => _isOffline;

  String? _error;
  String? get error => _error;

  String? _currentTypeFilter;
  int? _currentLimit;

  Future<void> loadTransactions({
    String? transactionType,
    int limit = 100,
  }) async {
    _isLoading = true;
    _error = null;
    _currentTypeFilter = transactionType;
    _currentLimit = limit;
    notifyListeners();

    try {
      _transactions = await _apiService.getTransactions(
        transactionType: transactionType,
        limit: limit,
      );
      _isOffline = false;
    } catch (e) {
      if (kDebugMode) print('Failed to load transactions: $e');
      _error = e.toString();
      _isOffline = true;
      // Keep existing cached data if available
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  /// Get transactions filtered by date range
  List<Map<String, dynamic>> filterByDateRange(DateTimeRange? dateRange) {
    if (dateRange == null) return _transactions;

    return _transactions.where((t) {
      final date = DateTime.tryParse(t['created_at'] ?? '');
      if (date == null) return true;
      return date.isAfter(dateRange.start.subtract(const Duration(days: 1))) &&
          date.isBefore(dateRange.end.add(const Duration(days: 1)));
    }).toList();
  }

  /// Get today's transactions
  List<Map<String, dynamic>> get todaysTransactions {
    final today = DateTime.now();
    return _transactions.where((t) {
      final date = DateTime.tryParse(t['created_at'] ?? '');
      if (date == null) return false;
      return date.year == today.year &&
          date.month == today.month &&
          date.day == today.day;
    }).toList();
  }

  /// Get today's sales total
  double get todaysSalesTotal {
    return todaysTransactions
        .where((t) => t['transaction_type'] == 'Sale')
        .fold(0.0, (sum, t) => sum + (t['total_amount'] ?? 0.0));
  }

  /// Refresh transactions with current filters
  Future<void> refresh() async {
    await loadTransactions(
      transactionType: _currentTypeFilter,
      limit: _currentLimit ?? 100,
    );
  }

  /// Get a specific transaction by UUID
  Map<String, dynamic>? getById(String transactionUuid) {
    return _transactions
        .where((t) => t['transaction_uuid'] == transactionUuid)
        .firstOrNull;
  }
}
