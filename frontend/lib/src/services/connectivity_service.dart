import 'dart:async';
import 'package:connectivity_plus/connectivity_plus.dart';

/// Connectivity Service (PHASE 3 - Network Monitoring)
///
/// Provides a clean interface for checking network status.
/// Notifies listeners when connectivity changes.
///
/// Usage:
/// ```dart
/// final connectivity = ConnectivityService();
///
/// // Check current status
/// if (await connectivity.isOnline) {
///   // Sync now
/// }
///
/// // Listen to changes
/// connectivity.onConnectivityChanged.listen((isOnline) {
///   if (isOnline) {
///     triggerSync();
///   }
/// });
/// ```

class ConnectivityService {
  final Connectivity _connectivity;
  final _controller = StreamController<bool>.broadcast();

  ConnectivityService({Connectivity? connectivity})
      : _connectivity = connectivity ?? Connectivity() {
    _init();
  }

  /// Stream of connectivity changes (true = online, false = offline)
  Stream<bool> get onConnectivityChanged => _controller.stream;

  /// Check if currently online
  Future<bool> get isOnline async {
    final result = await _connectivity.checkConnectivity();
    return _isConnected(result);
  }

  /// Check if currently offline
  Future<bool> get isOffline async {
    return !(await isOnline);
  }

  /// Initialize connectivity listener
  void _init() {
    _connectivity.onConnectivityChanged.listen((result) {
      final isConnected = _isConnected(result);
      _controller.add(isConnected);

      print(
          '[Connectivity] Status changed: ${isConnected ? 'ONLINE' : 'OFFLINE'}');
    });
  }

  /// Determine if connection type means "online"
  bool _isConnected(ConnectivityResult result) {
    return result != ConnectivityResult.none;
  }

  /// Get current connection type
  Future<ConnectivityResult> get connectionType async {
    return await _connectivity.checkConnectivity();
  }

  /// Get human-readable connection type
  Future<String> get connectionName async {
    final result = await connectionType;

    switch (result) {
      case ConnectivityResult.wifi:
        return 'Wi-Fi';
      case ConnectivityResult.mobile:
        return 'Mobile Data';
      case ConnectivityResult.ethernet:
        return 'Ethernet';
      case ConnectivityResult.vpn:
        return 'VPN';
      case ConnectivityResult.bluetooth:
        return 'Bluetooth';
      case ConnectivityResult.other:
        return 'Other';
      case ConnectivityResult.none:
        return 'Offline';
    }
  }

  /// Check if on Wi-Fi (for large downloads)
  Future<bool> get isWifi async {
    final result = await connectionType;
    return result == ConnectivityResult.wifi;
  }

  /// Check if on mobile data (may want to limit sync)
  Future<bool> get isMobileData async {
    final result = await connectionType;
    return result == ConnectivityResult.mobile;
  }

  /// Dispose resources
  void dispose() {
    _controller.close();
  }
}

/// Connectivity Status Widget Helper
///
/// Provides status information for UI display
class ConnectivityStatus {
  final bool isOnline;
  final String connectionType;
  final DateTime lastCheck;

  ConnectivityStatus({
    required this.isOnline,
    required this.connectionType,
    DateTime? lastCheck,
  }) : lastCheck = lastCheck ?? DateTime.now();

  /// Get status color for UI
  String get statusColor {
    return isOnline ? '#4CAF50' : '#F44336'; // Green : Red
  }

  /// Get status icon
  String get statusIcon {
    if (!isOnline) return '🔴';

    switch (connectionType) {
      case 'Wi-Fi':
        return '📶';
      case 'Mobile Data':
        return '📱';
      case 'Ethernet':
        return '🔌';
      default:
        return '🟢';
    }
  }

  /// Get status text
  String get statusText {
    return isOnline ? 'Online ($connectionType)' : 'Offline';
  }
}
