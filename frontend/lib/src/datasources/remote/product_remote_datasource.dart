import '../../api/generated/export.dart';
import '../../services/api_client.dart';
import '../../services/api_exceptions.dart';

/// Remote Data Source for Products (PHASE 2)
///
/// Handles all API communication for products.
/// Uses the production ApiClient for network requests.
///
/// Responsibilities:
/// - Make HTTP requests to backend
/// - Parse responses into domain models
/// - Throw typed exceptions on failure
/// - Handle pagination
///
/// Does NOT:
/// - Cache data (that's the local datasource's job)
/// - Handle offline scenarios (that's the repository's job)
/// - Manage sync queue (that's the sync service's job)

class ProductRemoteDataSource {
  final ApiClient apiClient;

  ProductRemoteDataSource({required this.apiClient});

  /// Fetch all products from server
  Future<List<Product>> getAll({int? limit, int? offset}) async {
    try {
      final response = await apiClient.get<List>(
        '/api/products',
        queryParameters: {
          if (limit != null) 'limit': limit,
          if (offset != null) 'offset': offset,
        },
      );

      return response
          .map((json) => Product.fromJson(json as Map<String, dynamic>))
          .toList();
    } on ApiException {
      rethrow;
    } catch (e) {
      throw UnknownException('Failed to fetch products: ${e.toString()}');
    }
  }

  /// Fetch a single product by ID
  Future<Product?> getById(String productUuid) async {
    try {
      final response = await apiClient.get<Map<String, dynamic>>(
        '/api/products/$productUuid',
      );

      return Product.fromJson(response);
    } on NotFoundException {
      return null;
    } on ApiException {
      rethrow;
    } catch (e) {
      throw UnknownException('Failed to fetch product: ${e.toString()}');
    }
  }

  /// Create a new product on server
  Future<Product> create(Product product) async {
    try {
      final response = await apiClient.post<Map<String, dynamic>>(
        '/api/products',
        data: product.toJson(),
      );

      return Product.fromJson(response);
    } on ApiException {
      rethrow;
    } catch (e) {
      throw UnknownException('Failed to create product: ${e.toString()}');
    }
  }

  /// Update existing product on server
  Future<Product> update(Product product) async {
    try {
      final response = await apiClient.put<Map<String, dynamic>>(
        '/api/products/${product.productUuid}',
        data: product.toJson(),
      );

      return Product.fromJson(response);
    } on ApiException {
      rethrow;
    } catch (e) {
      throw UnknownException('Failed to update product: ${e.toString()}');
    }
  }

  /// Delete product on server
  Future<void> delete(String productUuid) async {
    try {
      await apiClient.delete('/api/products/$productUuid');
    } on ApiException {
      rethrow;
    } catch (e) {
      throw UnknownException('Failed to delete product: ${e.toString()}');
    }
  }

  /// Search products by name
  Future<List<Product>> search(String query) async {
    try {
      final response = await apiClient.get<List>(
        '/api/products/search',
        queryParameters: {'q': query},
      );

      return response
          .map((json) => Product.fromJson(json as Map<String, dynamic>))
          .toList();
    } on ApiException {
      rethrow;
    } catch (e) {
      throw UnknownException('Failed to search products: ${e.toString()}');
    }
  }

  /// Get products by category
  Future<List<Product>> getByCategory(String category) async {
    try {
      final response = await apiClient.get<List>(
        '/api/products',
        queryParameters: {'category': category},
      );

      return response
          .map((json) => Product.fromJson(json as Map<String, dynamic>))
          .toList();
    } on ApiException {
      rethrow;
    } catch (e) {
      throw UnknownException(
          'Failed to fetch products by category: ${e.toString()}');
    }
  }

  /// Lookup product by barcode
  Future<Product?> getByBarcode(String barcode) async {
    try {
      final response = await apiClient.get<Map<String, dynamic>>(
        '/api/products/barcode/$barcode',
      );

      return Product.fromJson(response);
    } on NotFoundException {
      return null;
    } on ApiException {
      rethrow;
    } catch (e) {
      throw UnknownException('Failed to lookup barcode: ${e.toString()}');
    }
  }
}
