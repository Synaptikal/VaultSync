import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../services/offline_sync_service.dart';
import 'package:intl/intl.dart';

/// Widget that shows the current sync status and allows manual sync
class SyncStatusWidget extends StatelessWidget {
  final bool showLabel;
  final bool compact;
  
  const SyncStatusWidget({
    super.key,
    this.showLabel = true,
    this.compact = false,
  });

  @override
  Widget build(BuildContext context) {
    return Consumer<OfflineSyncService>(
      builder: (context, syncService, child) {
        return compact 
            ? _buildCompact(context, syncService)
            : _buildFull(context, syncService);
      },
    );
  }

  Widget _buildCompact(BuildContext context, OfflineSyncService syncService) {
    IconData icon;
    Color color;
    String tooltip;

    if (syncService.isSyncing) {
      return const SizedBox(
        width: 24,
        height: 24,
        child: CircularProgressIndicator(strokeWidth: 2),
      );
    }

    if (!syncService.isOnline) {
      icon = Icons.cloud_off;
      color = Colors.orange;
      tooltip = 'Offline - ${syncService.pendingChangesCount} changes pending';
    } else if (syncService.hasPendingChanges) {
      icon = Icons.sync;
      color = Colors.blue;
      tooltip = '${syncService.pendingChangesCount} changes to sync';
    } else if (syncService.lastError != null) {
      icon = Icons.sync_problem;
      color = Colors.red;
      tooltip = 'Sync error: ${syncService.lastError}';
    } else {
      icon = Icons.cloud_done;
      color = Colors.green;
      tooltip = syncService.lastSuccessfulSync != null
          ? 'Last sync: ${DateFormat.jm().format(syncService.lastSuccessfulSync!)}'
          : 'Synced';
    }

    return IconButton(
      icon: Stack(
        children: [
          Icon(icon, color: color),
          if (syncService.hasPendingChanges)
            Positioned(
              right: 0,
              top: 0,
              child: Container(
                padding: const EdgeInsets.all(2),
                decoration: const BoxDecoration(
                  color: Colors.red,
                  shape: BoxShape.circle,
                ),
                child: Text(
                  '${syncService.pendingChangesCount}',
                  style: const TextStyle(fontSize: 8, color: Colors.white),
                ),
              ),
            ),
        ],
      ),
      tooltip: tooltip,
      onPressed: syncService.isOnline ? () => syncService.syncNow() : null,
    );
  }

  Widget _buildFull(BuildContext context, OfflineSyncService syncService) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Row(
              children: [
                _buildStatusIcon(syncService),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        _getStatusTitle(syncService),
                        style: const TextStyle(fontWeight: FontWeight.bold),
                      ),
                      Text(
                        _getStatusSubtitle(syncService),
                        style: Theme.of(context).textTheme.bodySmall,
                      ),
                    ],
                  ),
                ),
                if (syncService.isSyncing)
                  const SizedBox(
                    width: 20,
                    height: 20,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                else if (syncService.isOnline)
                  TextButton(
                    onPressed: () => syncService.syncNow(),
                    child: const Text('Sync Now'),
                  ),
              ],
            ),
            if (syncService.hasPendingChanges) ...[
              const SizedBox(height: 12),
              LinearProgressIndicator(
                value: null,
                backgroundColor: Colors.grey.shade200,
              ),
              const SizedBox(height: 4),
              Text(
                '${syncService.pendingChangesCount} changes waiting to sync',
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ],
            if (syncService.lastError != null) ...[
              const SizedBox(height: 12),
              Container(
                padding: const EdgeInsets.all(8),
                decoration: BoxDecoration(
                  color: Colors.red.shade50,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Row(
                  children: [
                    Icon(Icons.error_outline, color: Colors.red.shade700, size: 16),
                    const SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        syncService.lastError!,
                        style: TextStyle(color: Colors.red.shade700, fontSize: 12),
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildStatusIcon(OfflineSyncService syncService) {
    IconData icon;
    Color bgColor;
    Color iconColor;

    if (!syncService.isOnline) {
      icon = Icons.cloud_off;
      bgColor = Colors.orange.shade100;
      iconColor = Colors.orange.shade700;
    } else if (syncService.hasPendingChanges) {
      icon = Icons.sync;
      bgColor = Colors.blue.shade100;
      iconColor = Colors.blue.shade700;
    } else if (syncService.lastError != null) {
      icon = Icons.sync_problem;
      bgColor = Colors.red.shade100;
      iconColor = Colors.red.shade700;
    } else {
      icon = Icons.cloud_done;
      bgColor = Colors.green.shade100;
      iconColor = Colors.green.shade700;
    }

    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: bgColor,
        shape: BoxShape.circle,
      ),
      child: Icon(icon, color: iconColor),
    );
  }

  String _getStatusTitle(OfflineSyncService syncService) {
    if (syncService.isSyncing) return 'Syncing...';
    if (!syncService.isOnline) return 'Offline Mode';
    if (syncService.hasPendingChanges) return 'Changes Pending';
    if (syncService.lastError != null) return 'Sync Error';
    return 'All Synced';
  }

  String _getStatusSubtitle(OfflineSyncService syncService) {
    if (syncService.isSyncing) return 'Please wait...';
    if (!syncService.isOnline) return 'Changes will sync when connected';
    if (syncService.lastSuccessfulSync != null) {
      return 'Last sync: ${DateFormat.yMMMd().add_jm().format(syncService.lastSuccessfulSync!)}';
    }
    return 'Never synced';
  }
}

/// Compact sync indicator for app bar
class SyncIndicator extends StatelessWidget {
  const SyncIndicator({super.key});

  @override
  Widget build(BuildContext context) {
    return const SyncStatusWidget(compact: true);
  }
}
