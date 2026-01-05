import 'package:flutter/foundation.dart';

/// Environment configuration for VaultSync Flutter app
///
/// Uses compile-time constants from --dart-define flags.
///
/// Example build:
/// ```bash
/// flutter build windows --dart-define=API_BASE_URL=https://api.yourshop.com --dart-define=ENVIRONMENT=production
/// ```
class Environment {
  /// API base URL for backend communication
  /// Defaults to localhost for development
  static const String apiBaseUrl = String.fromEnvironment(
    'API_BASE_URL',
    defaultValue: 'http://localhost:3000',
  );

  /// Current environment (development, staging, production)
  static const String environment = String.fromEnvironment(
    'ENVIRONMENT',
    defaultValue: 'development',
  );

  /// Whether running in production mode
  static bool get isProduction => environment == 'production';

  /// Whether running in development mode
  static bool get isDevelopment => environment == 'development';

  /// Whether to enable debug logging
  static bool get enableLogging => !isProduction;

  /// Whether to enable offline mode sync
  static const bool enableOfflineSync = bool.fromEnvironment(
    'ENABLE_OFFLINE_SYNC',
    defaultValue: true,
  );

  /// Sync interval in seconds
  static const int syncIntervalSeconds = int.fromEnvironment(
    'SYNC_INTERVAL_SECONDS',
    defaultValue: 30,
  );

  /// Print configuration for debugging
  static void printConfig() {
    if (enableLogging) {
      debugPrint('VaultSync Configuration:');
      debugPrint('  API Base URL: $apiBaseUrl');
      debugPrint('  Environment: $environment');
      debugPrint('  Offline Sync: $enableOfflineSync');
      debugPrint('  Sync Interval: ${syncIntervalSeconds}s');
    }
  }
}
