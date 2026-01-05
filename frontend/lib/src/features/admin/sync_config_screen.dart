import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:intl/intl.dart';
import '../../services/api_service.dart';
import '../../services/offline_sync_service.dart';
import '../../shared/sync_status_widget.dart';

class SyncConfigScreen extends StatefulWidget {
  const SyncConfigScreen({super.key});

  @override
  State<SyncConfigScreen> createState() => _SyncConfigScreenState();
}

class _SyncConfigScreenState extends State<SyncConfigScreen> {
  bool _isLoading = true;
  Map<String, dynamic>? _serverStatus;

  @override
  void initState() {
    super.initState();
    _loadStatus();
  }

  Future<void> _loadStatus() async {
    setState(() => _isLoading = true);
    try {
      final status = await context.read<ApiService>().getSyncStatus();
      if (mounted) {
        setState(() {
          _serverStatus = status;
          _isLoading = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() => _isLoading = false);
        ScaffoldMessenger.of(context)
            .showSnackBar(SnackBar(content: Text('Error: $e')));
      }
    }
  }

  Future<void> _triggerPeerSync() async {
    try {
      ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Requesting server peer sync...')));
      await context.read<ApiService>().triggerPeerSync();
      _loadStatus();
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context)
            .showSnackBar(SnackBar(content: Text('Sync failed: $e')));
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final syncService = context.watch<OfflineSyncService>();

    return Scaffold(
      appBar: AppBar(
        title: const Text('Sync Configuration'),
        actions: const [SyncIndicator()],
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Local Device Sync Status
            const Text('📱 This Device',
                style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
            const SizedBox(height: 8),
            const SyncStatusWidget(),
            const SizedBox(height: 24),

            // Device Details
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text('Device Sync Details',
                        style: TextStyle(fontWeight: FontWeight.bold)),
                    const Divider(),
                    _DetailRow(
                        'Status', syncService.isOnline ? 'Online' : 'Offline'),
                    _DetailRow('Pending Changes',
                        '${syncService.pendingChangesCount}'),
                    _DetailRow(
                        'Last Sync',
                        syncService.lastSuccessfulSync != null
                            ? DateFormat.yMMMd()
                                .add_jm()
                                .format(syncService.lastSuccessfulSync!)
                            : 'Never'),
                    if (syncService.lastError != null)
                      _DetailRow('Last Error', syncService.lastError!,
                          isError: true),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 8),
            Row(
              children: [
                Expanded(
                  child: FilledButton.icon(
                    onPressed: syncService.isOnline
                        ? () => syncService.syncNow()
                        : null,
                    icon: syncService.isSyncing
                        ? const SizedBox(
                            width: 16,
                            height: 16,
                            child: CircularProgressIndicator(
                                strokeWidth: 2, color: Colors.white))
                        : const Icon(Icons.sync),
                    label:
                        Text(syncService.isSyncing ? 'Syncing...' : 'Sync Now'),
                  ),
                ),
                const SizedBox(width: 8),
                OutlinedButton(
                  onPressed: () async {
                    final confirm = await showDialog<bool>(
                      context: context,
                      builder: (context) => AlertDialog(
                        title: const Text('Reset Sync State?'),
                        content: const Text(
                            'This will clear the sync history and re-sync all data from the server.'),
                        actions: [
                          TextButton(
                              onPressed: () => Navigator.pop(context, false),
                              child: const Text('Cancel')),
                          FilledButton(
                              onPressed: () => Navigator.pop(context, true),
                              child: const Text('Reset')),
                        ],
                      ),
                    );
                    if (confirm == true) {
                      await syncService.resetSyncState();
                      if (!context.mounted) return;
                      ScaffoldMessenger.of(context).showSnackBar(
                          const SnackBar(content: Text('Sync state reset')));
                    }
                  },
                  child: const Text('Reset'),
                ),
              ],
            ),

            const SizedBox(height: 32),
            const Divider(),
            const SizedBox(height: 16),

            // Server Status
            const Text('🖥️ Server Status',
                style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
            const SizedBox(height: 8),
            if (_isLoading)
              const Center(child: CircularProgressIndicator())
            else if (_serverStatus == null)
              Card(
                color: Colors.orange.shade50,
                child: const ListTile(
                  leading: Icon(Icons.warning, color: Colors.orange),
                  title: Text('Server status unavailable'),
                  subtitle: Text('Make sure the backend is running'),
                ),
              )
            else
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    children: [
                      Row(
                        children: [
                          Icon(Icons.hub,
                              size: 32, color: Colors.green.shade700),
                          const SizedBox(width: 16),
                          const Text('Backend Node',
                              style: TextStyle(
                                  fontSize: 16, fontWeight: FontWeight.bold)),
                        ],
                      ),
                      const Divider(),
                      _DetailRow(
                          'Node ID', _serverStatus!['node_id'] ?? 'Unknown'),
                      _DetailRow('Is Synced',
                          _serverStatus!['is_synced']?.toString() ?? 'false'),
                      _DetailRow('Pending Changes',
                          _serverStatus!['pending_changes']?.toString() ?? '0'),
                      _DetailRow('Connected Peers',
                          _serverStatus!['connected_peers']?.toString() ?? '0'),
                      _DetailRow(
                          'Last Sync', _serverStatus!['last_sync'] ?? 'Never'),
                    ],
                  ),
                ),
              ),
            const SizedBox(height: 8),
            SizedBox(
              width: double.infinity,
              child: OutlinedButton.icon(
                onPressed: _triggerPeerSync,
                icon: const Icon(Icons.cloud_sync),
                label: const Text('Trigger Server Peer Sync'),
              ),
            ),

            const SizedBox(height: 32),
            const Divider(),
            const SizedBox(height: 16),

            // Network Settings
            const Text('⚙️ Settings',
                style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
            const SizedBox(height: 8),
            Card(
              child: Column(
                children: [
                  SwitchListTile(
                    title: const Text('Auto-Sync'),
                    subtitle:
                        const Text('Automatically sync changes in background'),
                    value: true, // Would be configurable
                    onChanged: (v) {},
                  ),
                  const Divider(height: 1),
                  SwitchListTile(
                    title: const Text('Sync on Wi-Fi Only'),
                    subtitle:
                        const Text('Save mobile data by syncing only on Wi-Fi'),
                    value: false, // Would be configurable
                    onChanged: (v) {},
                  ),
                  const Divider(height: 1),
                  ListTile(
                    title: const Text('Sync Interval'),
                    subtitle: const Text('How often to check for changes'),
                    trailing: DropdownButton<int>(
                      value: 5,
                      items: const [
                        DropdownMenuItem(value: 1, child: Text('1 min')),
                        DropdownMenuItem(value: 5, child: Text('5 min')),
                        DropdownMenuItem(value: 15, child: Text('15 min')),
                        DropdownMenuItem(value: 30, child: Text('30 min')),
                      ],
                      onChanged: (v) {},
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _DetailRow extends StatelessWidget {
  final String label;
  final String value;
  final bool isError;

  const _DetailRow(this.label, this.value, {this.isError = false});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(label, style: const TextStyle(color: Colors.grey)),
          const SizedBox(width: 16),
          Flexible(
            child: Text(
              value,
              style: TextStyle(
                fontWeight: FontWeight.bold,
                color: isError ? Colors.red : null,
              ),
              textAlign: TextAlign.right,
            ),
          ),
        ],
      ),
    );
  }
}
