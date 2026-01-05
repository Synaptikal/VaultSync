import 'package:dio/dio.dart';
import 'package:flutter/foundation.dart';
import 'package:pretty_dio_logger/pretty_dio_logger.dart';
import '../config/environment.dart';
import 'storage_service.dart';

/// Service to provide the configured Dio client
class DioService {
  final StorageService _storageService;
  late final Dio _dio;

  DioService({StorageService? storageService})
      : _storageService = storageService ?? SecureStorageService() {
    _dio = Dio(BaseOptions(
      baseUrl: Environment.apiBaseUrl,
      connectTimeout: const Duration(seconds: 10),
      receiveTimeout: const Duration(seconds: 10),
      sendTimeout: const Duration(seconds: 10),
      validateStatus: (status) {
        // Return true for all statutes to handle them manually in ApiService
        // or false to let Dio throw DioException.
        // For now, allow 2xx, 4xx, 5xx to pass through so we can handle them
        // with the specific logic in ApiService, OR we transition to DioExceptions.
        // Let's stick to standard Dio behavior (throw on error) which is safer,
        // but ApiService expects to check status codes manually in some places.
        // actually, let's strictly validate 200-299.
        return status != null && status >= 200 && status < 300;
      },
    ));

    // Auth Interceptor
    _dio.interceptors.add(InterceptorsWrapper(
      onRequest: (options, handler) async {
        final token = await _storageService.read(key: 'jwt_token');
        if (token != null) {
          options.headers['Authorization'] = 'Bearer $token';
        }
        options.headers['Content-Type'] = 'application/json';
        options.headers['Accept'] = 'application/json';
        return handler.next(options);
      },
      onError: (DioException e, handler) async {
        if (e.response?.statusCode == 401) {
          // TODO: Handle token refresh logic here
          // For now, just pass the error through
          if (kDebugMode) {
            print('[DioService] 401 Unauthorized - Token may be expired');
          }
        }
        return handler.next(e);
      },
    ));

    // Logging Interceptor
    if (Environment.enableLogging) {
      _dio.interceptors.add(PrettyDioLogger(
        requestHeader: true,
        requestBody: true,
        responseHeader: true,
        responseBody: true,
        error: true,
        compact: true,
        maxWidth: 90,
      ));
    }
  }

  Dio get client => _dio;
}
