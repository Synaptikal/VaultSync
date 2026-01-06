import '../../api/generated/export.dart';
import '../../services/api_client.dart';
import '../../services/api_exceptions.dart';

/// Remote Data Source for Inventory Items (TASK-AUD-001e)
///
/// Handles all API communication for inventory items.
/// Uses the production ApiClient for network requests.
///
/// Responsibilities:
/// - Make HTTP requests to backend
/// - Parse responses into domain models
/// - Throw typed exceptions on failure
/// - Handle pagination

class InventoryRemoteDataSource {
  final ApiClient apiClient;

  InventoryRemoteDataSource({required this.apiClient});

  /// Fetch all inventory items from server
  Future<List<InventoryItem>> getAll({int? limit, int? offset}) async {
    try {
      final response = await apiClient.get<List>(
        '/api/inventory',
        queryParameters: {
          if (limit != null) 'limit': limit,
          if (offset != null) 'offset': offset,
        },
      );

      return response
          .map((json) => InventoryItem.fromJson(json as Map<String, dynamic>))
          .toList();
    } on ApiException {
      rethrow;
    } catch (e) {
      throw UnknownException('Failed to fetch inventory: ${e.toString()}');
    }
  }

  /// Fetch a single inventory item by ID
  Future<InventoryItem?> getById(String inventoryUuid) async {
    try {
      final response = await apiClient.get<Map<String, dynamic>>(
        '/api/inventory/$inventoryUuid',
      );

      return InventoryItem.fromJson(response);
    } on NotFoundException {
      return null;
    } on ApiException {
      rethrow;
    } catch (e) {
      throw UnknownException('Failed to fetch inventory item: ${e.toString()}');
    }
  }

  /// Create a new inventory item on server
  Future<InventoryItem> create(InventoryItem item) async {
    try {
      final response = await apiClient.post<Map<String, dynamic>>(
        '/api/inventory',
        data: item.toJson(),
      );

      return InventoryItem.fromJson(response);
    } on ApiException {
      rethrow;
    } catch (e) {
      throw UnknownException(
          'Failed to create inventory item: ${e.toString()}');
    }
  }

  /// Update existing inventory item on server
  Future<InventoryItem> update(InventoryItem item) async {
    try {
      final response = await apiClient.put<Map<String, dynamic>>(
        '/api/inventory/${item.inventoryUuid}',
        data: item.toJson(),
      );

      return InventoryItem.fromJson(response);
    } on ApiException {
      rethrow;
    } catch (e) {
      throw UnknownException(
          'Failed to update inventory item: ${e.toString()}');
    }
  }

  /// Delete inventory item on server
  Future<void> delete(String inventoryUuid) async {
    try {
      await apiClient.delete('/api/inventory/$inventoryUuid');
    } on ApiException {
      rethrow;
    } catch (e) {
      throw UnknownException(
          'Failed to delete inventory item: ${e.toString()}');
    }
  }

  /// Get inventory items by product
  Future<List<InventoryItem>> getByProductUuid(String productUuid) async {
    try {
      final response = await apiClient.get<List>(
        '/api/inventory',
        queryParameters: {'product_uuid': productUuid},
      );

      return response
          .map((json) => InventoryItem.fromJson(json as Map<String, dynamic>))
          .toList();
    } on ApiException {
      rethrow;
    } catch (e) {
      throw UnknownException(
          'Failed to fetch inventory by product: ${e.toString()}');
    }
  }

  /// Get low stock items
  Future<List<InventoryItem>> getLowStock({int threshold = 3}) async {
    try {
      final response = await apiClient.get<List>(
        '/api/inventory/low-stock',
        queryParameters: {'threshold': threshold},
      );

      return response
          .map((json) => InventoryItem.fromJson(json as Map<String, dynamic>))
          .toList();
    } on ApiException {
      rethrow;
    } catch (e) {
      throw UnknownException(
          'Failed to fetch low stock items: ${e.toString()}');
    }
  }

  /// Bulk update inventory
  Future<void> bulkUpdate(List<InventoryItem> items) async {
    try {
      await apiClient.post(
        '/api/inventory/bulk',
        data: items.map((i) => i.toJson()).toList(),
      );
    } on ApiException {
      rethrow;
    } catch (e) {
      throw UnknownException(
          'Failed to bulk update inventory: ${e.toString()}');
    }
  }
}
