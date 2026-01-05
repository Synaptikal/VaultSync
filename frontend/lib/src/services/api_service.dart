import 'package:dio/dio.dart';
import '../api/generated/models/product.dart';
import '../api/generated/models/customer.dart';
import '../api/generated/models/inventory_item.dart';
import '../models/sync_models.dart';
import '../models/buylist_models.dart';
import '../api/generated/models/event.dart';
import '../api/generated/models/event_participant.dart';
import '../api/generated/models/wants_list.dart';
import 'storage_service.dart';
import 'dio_service.dart';
import 'api_exceptions.dart';

class ApiService {
  final StorageService _storageService;
  late final Dio _dio;

  ApiService({
    StorageService? storageService,
    Dio? dio,
  }) : _storageService = storageService ?? SecureStorageService() {
    _dio = dio ?? DioService(storageService: _storageService).client;
  }

  /// Centralized error handling
  Exception _handleError(Object error) {
    if (error is DioException) {
      if (error.type == DioExceptionType.connectionTimeout ||
          error.type == DioExceptionType.sendTimeout ||
          error.type == DioExceptionType.receiveTimeout ||
          error.type == DioExceptionType.connectionError) {
        return NetworkException('Connection failed. You may be offline.');
      }

      if (error.response != null) {
        final statusCode = error.response!.statusCode;
        final data = error.response!.data;
        final message = (data is Map && data['error'] != null)
            ? data['error']
            : error.message ?? 'Unknown error';

        switch (statusCode) {
          case 400:
            return ValidationException(message);
          case 401:
            return AuthenticationException(
                'Session expired. Please login again.');
          case 403:
            return AuthorizationException('Access denied.');
          case 404:
            return NotFoundException('Resource not found.');
          case 409:
            return ConflictException(message);
          case 500:
            return ServerException('Server error. Please try again later.');
        }
      }
    }
    return UnknownException(error.toString());
  }

  // ============================================
  // Authentication
  // ============================================

  Future<bool> login(String username, String password) async {
    try {
      final response = await _dio.post(
        '/auth/login',
        data: {'username': username, 'password': password},
      );
      final data = response.data;
      await _storageService.write(key: 'jwt_token', value: data['token']);
      return true;
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<bool> register(String username, String email, String password,
      {String role = 'admin'}) async {
    try {
      await _dio.post(
        '/auth/register',
        data: {
          'username': username,
          'password': password,
          'role': role,
        },
      );
      return true;
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<void> logout() async {
    await _storageService.delete(key: 'jwt_token');
  }

  Future<bool> healthCheck() async {
    try {
      await _dio.get('/health');
      return true;
    } catch (e) {
      return false;
    }
  }

  // ============================================
  // User
  // ============================================

  Future<Map<String, dynamic>> getCurrentUser() async {
    try {
      final response = await _dio.get('/api/user/me');
      return response.data as Map<String, dynamic>;
    } catch (e) {
      throw _handleError(e);
    }
  }

  // ============================================
  // Products
  // ============================================

  Future<List<Product>> getProducts({String query = ''}) async {
    if (query.isNotEmpty) {
      return searchProducts(query: query);
    }

    try {
      final response = await _dio.get('/api/products');
      final List<dynamic> data = response.data;
      return data
          .map((json) => Product.fromJson(json as Map<String, dynamic>))
          .toList();
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<Product> getProductById(String productUuid) async {
    try {
      final response = await _dio.get('/api/products/$productUuid');
      return Product.fromJson(response.data as Map<String, dynamic>);
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<List<Product>> searchProducts({
    String? query,
    String? category,
    int limit = 50,
    int offset = 0,
  }) async {
    try {
      final queryParams = <String, dynamic>{
        'limit': limit,
        'offset': offset,
      };
      if (query != null && query.isNotEmpty) queryParams['search'] = query;
      if (category != null) queryParams['category'] = category;

      final response = await _dio.get(
        '/api/products/search',
        queryParameters: queryParams,
      );

      final List<dynamic> data = response.data;
      return data
          .map((json) => Product.fromJson(json as Map<String, dynamic>))
          .toList();
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<Product> createProduct(Map<String, dynamic> productData) async {
    try {
      final response = await _dio.post(
        '/api/products',
        data: productData,
      );
      return Product.fromJson(response.data as Map<String, dynamic>);
    } catch (e) {
      throw _handleError(e);
    }
  }

  // ============================================
  // Inventory
  // ============================================

  Future<List<InventoryItem>> getInventory(
      {int limit = 50, int offset = 0}) async {
    try {
      final response = await _dio.get(
        '/api/inventory',
        queryParameters: {'limit': limit, 'offset': offset},
      );
      final List<dynamic> data = response.data;
      return data
          .map((json) => InventoryItem.fromJson(json as Map<String, dynamic>))
          .toList();
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<InventoryItem> getInventoryItem(String inventoryUuid) async {
    try {
      final response = await _dio.get('/api/inventory/$inventoryUuid');
      return InventoryItem.fromJson(response.data as Map<String, dynamic>);
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<void> deleteInventoryItem(String inventoryUuid) async {
    try {
      await _dio.delete('/api/inventory/$inventoryUuid');
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<List<InventoryItem>> getLowStockItems({int threshold = 3}) async {
    try {
      final response = await _dio.get(
        '/api/inventory/low-stock',
        queryParameters: {'threshold': threshold},
      );
      final List<dynamic> data = response.data;
      return data
          .map((json) => InventoryItem.fromJson(json as Map<String, dynamic>))
          .toList();
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<Map<String, int>> getInventoryMatrix(String productUuid) async {
    try {
      final response = await _dio.get(
        '/api/inventory/matrix',
        queryParameters: {'product_uuid': productUuid},
      );
      final Map<String, dynamic> data = response.data;
      return data.map((key, value) => MapEntry(key, value as int));
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<void> bulkInventoryUpdate(List<Map<String, dynamic>> items) async {
    try {
      await _dio.post('/api/inventory/bulk', data: items);
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<void> addInventory(Map<String, dynamic> inventoryData) async {
    try {
      await _dio.post('/api/inventory', data: inventoryData);
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<void> updateInventory(
      String inventoryUuid, Map<String, dynamic> inventoryData) async {
    try {
      await _dio.put('/api/inventory/$inventoryUuid', data: inventoryData);
    } catch (e) {
      throw _handleError(e);
    }
  }

  // ============================================
  // Customers
  // ============================================

  Future<List<Customer>> getCustomers() async {
    try {
      final response = await _dio.get('/api/customers');
      final List<dynamic> data = response.data;
      return data
          .map((json) => Customer.fromJson(json as Map<String, dynamic>))
          .toList();
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<Customer> getCustomerById(String customerUuid) async {
    try {
      final response = await _dio.get('/api/customers/$customerUuid');
      return Customer.fromJson(response.data as Map<String, dynamic>);
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<void> createCustomer(Map<String, dynamic> customerData) async {
    try {
      await _dio.post('/api/customers', data: customerData);
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<List<Map<String, dynamic>>> getCustomerHistory(
      String customerUuid) async {
    try {
      final response = await _dio.get(
        '/api/customers/history',
        queryParameters: {'customer_uuid': customerUuid},
      );
      final List<dynamic> data = response.data;
      return data.cast<Map<String, dynamic>>();
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<void> updateStoreCredit(String customerUuid, double amount) async {
    try {
      await _dio.post(
        '/api/customers/credit',
        data: {'customer_uuid': customerUuid, 'amount': amount},
      );
    } catch (e) {
      throw _handleError(e);
    }
  }

  // ============================================
  // Transactions
  // ============================================

  Future<void> createTransaction({
    required String? customerUuid,
    required List<Map<String, dynamic>> items,
    List<Map<String, dynamic>>? tradeInItems,
    String transactionType = 'Sale',
  }) async {
    try {
      await _dio.post(
        '/api/transactions',
        data: {
          'customer_uuid': customerUuid,
          'items': items,
          'trade_in_items': tradeInItems,
          'transaction_type': transactionType,
        },
      );
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<List<Map<String, dynamic>>> getTransactions({
    String? customerUuid,
    String? transactionType,
    int limit = 50,
    int offset = 0,
  }) async {
    try {
      final queryParams = <String, dynamic>{
        'limit': limit,
        'offset': offset,
      };
      if (customerUuid != null) queryParams['customer_uuid'] = customerUuid;
      if (transactionType != null) {
        queryParams['transaction_type'] = transactionType;
      }

      final response = await _dio.get(
        '/api/transactions',
        queryParameters: queryParams,
      );
      final List<dynamic> data = response.data;
      return data.cast<Map<String, dynamic>>();
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<Map<String, dynamic>> getTransactionById(
      String transactionUuid) async {
    try {
      final response = await _dio.get('/api/transactions/$transactionUuid');
      return response.data as Map<String, dynamic>;
    } catch (e) {
      throw _handleError(e);
    }
  }

  // ============================================
  // Pricing
  // ============================================

  Future<Map<String, dynamic>> getPricingDashboard() async {
    try {
      final response = await _dio.get('/api/pricing/dashboard');
      return response.data as Map<String, dynamic>;
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<Map<String, dynamic>> getPriceInfo(String productUuid) async {
    try {
      final response = await _dio.get('/api/pricing/$productUuid');
      return response.data as Map<String, dynamic>;
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<void> triggerPriceSync() async {
    try {
      await _dio.post('/api/pricing/sync');
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<void> logPriceOverride(String productUuid, double originalPrice,
      double newPrice, String reason) async {
    try {
      await _dio.post(
        '/api/pricing/override',
        data: {
          'product_uuid': productUuid,
          'original_price': originalPrice,
          'new_price': newPrice,
          'reason': reason,
          'timestamp': DateTime.now().toIso8601String(),
        },
      );
    } catch (e) {
      throw _handleError(e);
    }
  }

  // ============================================
  // Buylist
  // ============================================

  Future<QuoteResult> getBuylistQuote(BuylistItem item) async {
    try {
      final response = await _dio.post(
        '/api/buylist/quote',
        data: item.toJson(),
      );
      return QuoteResult.fromJson(response.data);
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<void> processBuylist({
    required List<BuylistItem> items,
    String? customerUuid,
    required PaymentMethod paymentMethod,
  }) async {
    try {
      final body = {
        'items': items.map((i) => i.toJson()).toList(),
        'customer_uuid': customerUuid,
        'payment_method':
            paymentMethod == PaymentMethod.cash ? 'Cash' : 'StoreCredit',
      };
      await _dio.post('/api/buylist/process', data: body);
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<void> processTradeIn({
    required List<BuylistItem> tradeInItems,
    required List<Map<String, dynamic>> purchaseItems,
    String? customerUuid,
  }) async {
    try {
      final body = {
        'trade_in_items': tradeInItems.map((i) => i.toJson()).toList(),
        'purchase_items': purchaseItems,
        'customer_uuid': customerUuid,
      };
      await _dio.post('/api/buylist/trade-in', data: body);
    } catch (e) {
      throw _handleError(e);
    }
  }

  // ============================================
  // Sync
  // ============================================

  Future<Map<String, dynamic>> getSyncStatus() async {
    try {
      final response = await _dio.get('/api/sync/status');
      return response.data as Map<String, dynamic>;
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<void> pushSyncChanges(List<ChangeRecord> changes) async {
    try {
      await _dio.post(
        '/api/sync/push',
        data: changes.map((c) => c.toJson()).toList(),
      );
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<List<ChangeRecord>> pullSyncChanges(int lastVersion) async {
    try {
      final response = await _dio.get(
        '/api/sync/pull',
        queryParameters: {'since': lastVersion},
      );
      final List<dynamic> data = response.data;
      return data.map((json) => ChangeRecord.fromJson(json)).toList();
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<void> triggerPeerSync() async {
    try {
      await _dio.post('/api/sync/trigger');
    } catch (e) {
      throw _handleError(e);
    }
  }

  // ============================================
  // Reports
  // ============================================

  Future<Map<String, dynamic>> getSalesReport({
    DateTime? startDate,
    DateTime? endDate,
  }) async {
    try {
      final queryParams = <String, dynamic>{};
      if (startDate != null)
        queryParams['start_date'] = startDate.toIso8601String();
      if (endDate != null) queryParams['end_date'] = endDate.toIso8601String();

      final response = await _dio.get(
        '/api/reports/sales',
        queryParameters: queryParams,
      );
      return response.data as Map<String, dynamic>;
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<Map<String, dynamic>> getInventoryValuation() async {
    try {
      final response = await _dio.get('/api/reports/inventory-valuation');
      return response.data as Map<String, dynamic>;
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<Map<String, dynamic>> getTopSellers({
    DateTime? startDate,
    DateTime? endDate,
    int limit = 10,
  }) async {
    try {
      final queryParams = <String, dynamic>{
        'limit': limit,
      };
      if (startDate != null)
        queryParams['start_date'] = startDate.toIso8601String();
      if (endDate != null) queryParams['end_date'] = endDate.toIso8601String();

      final response = await _dio.get(
        '/api/reports/top-sellers',
        queryParameters: queryParams,
      );
      return response.data as Map<String, dynamic>;
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<Map<String, dynamic>> getLowStockReport({int threshold = 5}) async {
    try {
      final response = await _dio.get(
        '/api/reports/low-stock',
        queryParameters: {'threshold': threshold},
      );
      return response.data as Map<String, dynamic>;
    } catch (e) {
      throw _handleError(e);
    }
  }

  // ============================================
  // Dashboard
  // ============================================

  Future<Map<String, dynamic>> getDashboardStats() async {
    try {
      final response = await _dio.get('/api/dashboard/stats');
      return response.data as Map<String, dynamic>;
    } catch (e) {
      throw _handleError(e);
    }
  }

  // ============================================
  // Events
  // ============================================

  Future<List<Event>> getEvents() async {
    try {
      final response = await _dio.get('/api/events');
      final List<dynamic> data = response.data;
      return data
          .map((json) => Event.fromJson(json as Map<String, dynamic>))
          .toList();
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<void> createEvent(Event event) async {
    try {
      await _dio.post('/api/events', data: event.toJson());
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<void> registerParticipant(
      String eventUuid, EventParticipant participant) async {
    try {
      await _dio.post(
        '/api/events/$eventUuid/register',
        data: participant.toJson(),
      );
    } catch (e) {
      throw _handleError(e);
    }
  }

  // ============================================
  // Wants List
  // ============================================

  Future<void> createWantsList(WantsList wantsList) async {
    try {
      await _dio.post('/api/wants', data: wantsList.toJson());
    } catch (e) {
      throw _handleError(e);
    }
  }

  Future<List<WantsList>> getWantsLists(String customerUuid) async {
    try {
      final response = await _dio.get('/api/customers/$customerUuid/wants');
      final List<dynamic> data = response.data;
      return data
          .map((json) => WantsList.fromJson(json as Map<String, dynamic>))
          .toList();
    } catch (e) {
      throw _handleError(e);
    }
  }
}
