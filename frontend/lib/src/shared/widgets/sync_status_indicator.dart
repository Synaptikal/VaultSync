import 'package:flutter/material.dart';
import '../../services/sync_queue_service.dart';
import '../../services/connectivity_service.dart';

/// Sync Status Indicator Widget (PHASE 3 - UI Component)
///
/// Displays current sync status in the app bar or bottom of screen.
/// Shows:
/// - Online/Offline status
/// - Pending sync count
/// - Last sync time
/// - Tap to manually trigger sync
///
/// Usage:
/// ```dart
/// AppBar(
///   actions: [
///     SyncStatusIndicator(
///       syncQueueService: syncQueue,
///       connectivityService: connectivity,
///     ),
///   ],
/// )
/// ```

class SyncStatusIndicator extends StatefulWidget {
  final SyncQueueService syncQueueService;
  final ConnectivityService connectivityService;
  final bool showLabel;

  const SyncStatusIndicator({
    Key? key,
    required this.syncQueueService,
    required this.connectivityService,
    this.showLabel = true,
  }) : super(key: key);

  @override
  State<SyncStatusIndicator> createState() => _SyncStatusIndicatorState();
}

class _SyncStatusIndicatorState extends State<SyncStatusIndicator> {
  bool _isOnline = true;
  int _pendingCount = 0;
  bool _isSyncing = false;

  @override
  void initState() {
    super.initState();
    _loadStatus();
    _listenToConnectivity();
  }

  Future<void> _loadStatus() async {
    final isOnline = await widget.connectivityService.isOnline;
    final pending = await widget.syncQueueService.getCount();

    if (mounted) {
      setState(() {
        _isOnline = isOnline;
        _pendingCount = pending;
      });
    }
  }

  void _listenToConnectivity() {
    widget.connectivityService.onConnectivityChanged.listen((isOnline) {
      if (mounted) {
        setState(() {
          _isOnline = isOnline;
        });

        // Reload pending count when connectivity changes
        _loadStatus();
      }
    });
  }

  Future<void> _manualSync() async {
    if (!_isOnline || _isSyncing) return;

    setState(() {
      _isSyncing = true;
    });

    try {
      final (successCount, failureCount) =
          await widget.syncQueueService.processQueue();

      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              'Synced $successCount items${failureCount > 0 ? ', $failureCount failed' : ''}',
            ),
            backgroundColor: failureCount > 0 ? Colors.orange : Colors.green,
            duration: const Duration(seconds: 3),
          ),
        );
      }

      await _loadStatus();
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Sync failed: ${e.toString()}'),
            backgroundColor: Colors.red,
          ),
        );
      }
    } finally {
      if (mounted) {
        setState(() {
          _isSyncing = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: _pendingCount > 0 && _isOnline ? _manualSync : null,
      borderRadius: BorderRadius.circular(8),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            _buildStatusIcon(),
            if (widget.showLabel) ...[
              const SizedBox(width: 8),
              _buildStatusText(),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildStatusIcon() {
    if (_isSyncing) {
      return const SizedBox(
        width: 16,
        height: 16,
        child: CircularProgressIndicator(
          strokeWidth: 2,
          valueColor: AlwaysStoppedAnimation<Color>(Colors.blue),
        ),
      );
    }

    if (!_isOnline) {
      return const Icon(
        Icons.cloud_off,
        color: Colors.grey,
        size: 20,
      );
    }

    if (_pendingCount > 0) {
      return Stack(
        children: [
          const Icon(
            Icons.cloud_upload,
            color: Colors.orange,
            size: 20,
          ),
          Positioned(
            right: 0,
            top: 0,
            child: Container(
              padding: const EdgeInsets.all(2),
              decoration: const BoxDecoration(
                color: Colors.red,
                shape: BoxShape.circle,
              ),
              constraints: const BoxConstraints(
                minWidth: 12,
                minHeight: 12,
              ),
              child: Text(
                _pendingCount > 9 ? '9+' : '$_pendingCount',
                style: const TextStyle(
                  color: Colors.white,
                  fontSize: 8,
                  fontWeight: FontWeight.bold,
                ),
                textAlign: TextAlign.center,
              ),
            ),
          ),
        ],
      );
    }

    return const Icon(
      Icons.cloud_done,
      color: Colors.green,
      size: 20,
    );
  }

  Widget _buildStatusText() {
    String text;
    Color color;

    if (_isSyncing) {
      text = 'Syncing...';
      color = Colors.blue;
    } else if (!_isOnline) {
      text = 'Offline';
      color = Colors.grey;
    } else if (_pendingCount > 0) {
      text = '$_pendingCount pending';
      color = Colors.orange;
    } else {
      text = 'All synced';
      color = Colors.green;
    }

    return Text(
      text,
      style: TextStyle(
        color: color,
        fontSize: 12,
        fontWeight: FontWeight.w500,
      ),
    );
  }
}

/// Sync Status Card - Full details
///
/// Expandable card showing detailed sync status
class SyncStatusCard extends StatelessWidget {
  final SyncQueueService syncQueueService;
  final ConnectivityService connectivityService;

  const SyncStatusCard({
    Key? key,
    required this.syncQueueService,
    required this.connectivityService,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<(bool, int)>(
      future: Future.wait([
        connectivityService.isOnline,
        syncQueueService.getCount(),
      ]).then((results) => (results[0] as bool, results[1] as int)),
      builder: (context, snapshot) {
        if (!snapshot.hasData) {
          return const Card(
            child: ListTile(
              leading: CircularProgressIndicator(),
              title: Text('Loading sync status...'),
            ),
          );
        }

        final (isOnline, pendingCount) = snapshot.data!;

        return Card(
          child: ExpansionTile(
            leading: Icon(
              isOnline ? Icons.cloud_done : Icons.cloud_off,
              color: isOnline ? Colors.green : Colors.grey,
            ),
            title: Text(
              isOnline ? 'Online' : 'Offline',
              style: const TextStyle(fontWeight: FontWeight.bold),
            ),
            subtitle: Text(
              pendingCount > 0
                  ? '$pendingCount items waiting to sync'
                  : 'All changes synced',
            ),
            children: [
              if (pendingCount > 0)
                ListTile(
                  leading: const Icon(Icons.sync, color: Colors.blue),
                  title: const Text('Sync Now'),
                  subtitle: const Text('Manually trigger sync'),
                  trailing: const Icon(Icons.chevron_right),
                  onTap: () async {
                    final (success, failure) =
                        await syncQueueService.processQueue();
                    if (context.mounted) {
                      ScaffoldMessenger.of(context).showSnackBar(
                        SnackBar(
                          content: Text('Synced: $success, Failed: $failure'),
                        ),
                      );
                    }
                  },
                ),
              ListTile(
                leading: const Icon(Icons.info_outline),
                title: const Text('View Sync Details'),
                trailing: const Icon(Icons.chevron_right),
                onTap: () {
                  // Navigate to detailed sync status screen
                },
              ),
            ],
          ),
        );
      },
    );
  }
}
