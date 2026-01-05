import 'package:flutter/foundation.dart' hide Category;
import '../services/api_service.dart';
import '../services/local_storage_service.dart';
import '../api/generated/models/product.dart';
import '../api/generated/models/category.dart';

class ProductProvider with ChangeNotifier {
  final ApiService _apiService;
  final LocalStorageService _localStorage = LocalStorageService();

  ProductProvider(this._apiService);

  List<Product> _products = [];
  List<Product> _allProducts = []; // Unfiltered cache
  List<Product> get products => _products;

  Category? _activeCategory;
  Category? get activeCategory => _activeCategory;

  bool _isLoading = false;
  bool get isLoading => _isLoading;

  String? _error;
  String? get error => _error;

  void filterByCategory(Category? category) {
    _activeCategory = category;
    if (category == null) {
      _products = List.from(_allProducts);
    } else {
      _products = _allProducts.where((p) => p.category == category).toList();
    }
    notifyListeners();
  }

  Future<void> loadProducts({String query = ''}) async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      // 1. Try to sync pending items first
      await _syncPendingProducts();

      // 2. Try online fetch
      try {
        final fetched = await _apiService.getProducts(query: query);

        // If this was a full load (no query), update our master cache
        if (query.isEmpty) {
          _allProducts = List.from(fetched);
          for (var product in fetched) {
            await _localStorage.saveProduct(product, isSynced: true);
          }
        }

        // If we have an active filter, apply it to the results
        if (_activeCategory != null) {
          _products =
              fetched.where((p) => p.category == _activeCategory).toList();
        } else {
          _products = fetched;
        }
      } catch (e) {
        // 3. Fallback to offline
        if (kDebugMode) print('Online fetch failed, loading from local DB: $e');

        var localProducts = await _localStorage.getProducts();

        // If this was a full load, update master cache
        if (query.isEmpty) {
          _allProducts = List.from(localProducts);
        }

        // Apply query filter if needed
        if (query.isNotEmpty) {
          localProducts = localProducts
              .where((p) =>
                  p.name.toLowerCase().contains(query.toLowerCase()) ||
                  (p.setCode?.toLowerCase().contains(query.toLowerCase()) ??
                      false))
              .toList();
        }

        // Apply category filter if needed
        if (_activeCategory != null) {
          _products = localProducts
              .where((p) => p.category == _activeCategory)
              .toList();
        } else {
          _products = localProducts;
        }
      }
    } catch (e) {
      _error = e.toString();
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<void> _syncPendingProducts() async {
    try {
      final pending = await _localStorage.getUnsyncedProducts();
      for (var product in pending) {
        try {
          await _apiService.createProduct(product.toJson());
          await _localStorage.saveProduct(product, isSynced: true);
        } catch (e) {
          if (kDebugMode)
            print('Failed to sync product ${product.productUuid}: $e');
          // Keep it unsynced
        }
      }
    } catch (e) {
      if (kDebugMode) print('Sync pending failed: $e');
    }
  }

  Future<void> addProduct(Map<String, dynamic> productData) async {
    _isLoading = true;
    notifyListeners();

    // Map string category to Enum
    // The generated Product.fromJson handles category string to enum conversion automatically
    if (productData['category'] is String) {
      // Category is handled by Product.fromJson
    }

    // Fix: Ensure category is handled correctly if it's passed as a string
    // The Product.fromJson generated code expects the JSON map to have the string value for the enum.
    final product = Product.fromJson(productData);

    try {
      // 1. Save locally as unsynced (Optimistic)
      await _localStorage.saveProduct(product, isSynced: false);

      // 2. Try to push to API
      // Note: We need to pass the JSON representation.
      // The generated toJson() returns Map<String, dynamic> (or Object?)
      // We might need to cast or ensure it matches what api_service expects.
      await _apiService.createProduct(product.toJson() as Map<String, dynamic>);

      // 3. If success, mark as synced
      await _localStorage.saveProduct(product, isSynced: true);

      // 4. Reload to ensure consistency
      await loadProducts();
    } catch (e) {
      if (kDebugMode) print('Add product offline mode: $e');
      // If API fails, we still have it locally.
      // We should probably just reload from local to show it in the list.
      _products = await _localStorage.getProducts();
      // We don't rethrow here, so the UI thinks it succeeded (Offline success).
      // But maybe we should warn the user?
      // For now, let's consider it a success "Saved to Device".
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }
}
