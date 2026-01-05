import 'package:flutter/material.dart';
import '../../../models/sync_conflict.dart';
import '../../../services/api_client.dart';
import '../../../services/api_exceptions.dart';

/// Conflict Resolution Screen (PHASE 4)
///
/// Displays pending CRDT conflicts and allows user to resolve them.
/// Features:
/// - List of pending conflicts
/// - Side-by-side state comparison
/// - Resolution buttons (LocalWins/RemoteWins)
/// - Conflict details and metadata
///
/// Usage:
/// ```dart
/// Navigator.push(
///   context,
///   MaterialPageRoute(
///     builder: (context) => ConflictResolutionScreen(),
///   ),
/// );
/// ```

class ConflictResolutionScreen extends StatefulWidget {
  final ApiClient apiClient;

  const ConflictResolutionScreen({
    Key? key,
    required this.apiClient,
  }) : super(key: key);

  @override
  State<ConflictResolutionScreen> createState() =>
      _ConflictResolutionScreenState();
}

class _ConflictResolutionScreenState extends State<ConflictResolutionScreen> {
  List<SyncConflict>? _conflicts;
  bool _isLoading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadConflicts();
  }

  Future<void> _loadConflicts() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });

    try {
      final data = await widget.apiClient.getPendingConflicts();
      final conflicts =
          data.map((json) => SyncConflict.fromJson(json)).toList();

      if (mounted) {
        setState(() {
          _conflicts = conflicts;
          _isLoading = false;
        });
      }
    } on ApiException catch (e) {
      if (mounted) {
        setState(() {
          _error = e.message;
          _isLoading = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _error = 'Failed to load conflicts: ${e.toString()}';
          _isLoading = false;
        });
      }
    }
  }

  Future<void> _resolveConflict(
    SyncConflict conflict,
    ResolutionStrategy strategy,
  ) async {
    try {
      await widget.apiClient.resolveConflict(
        conflict.conflictUuid,
        strategy.apiValue,
      );

      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Conflict resolved using ${strategy.displayName}'),
            backgroundColor: Colors.green,
          ),
        );

        // Reload conflicts
        await _loadConflicts();
      }
    } on ApiException catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Failed to resolve: ${e.message}'),
            backgroundColor: Colors.red,
          ),
        );
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Resolve Conflicts'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _loadConflicts,
            tooltip: 'Refresh',
          ),
        ],
      ),
      body: _buildBody(),
    );
  }

  Widget _buildBody() {
    if (_isLoading) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            CircularProgressIndicator(),
            SizedBox(height: 16),
            Text('Loading conflicts...'),
          ],
        ),
      );
    }

    if (_error != null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.error_outline, size: 64, color: Colors.red),
            const SizedBox(height: 16),
            Text(_error!, textAlign: TextAlign.center),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: _loadConflicts,
              child: const Text('Retry'),
            ),
          ],
        ),
      );
    }

    if (_conflicts == null || _conflicts!.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.check_circle, size: 64, color: Colors.green[300]),
            const SizedBox(height: 16),
            const Text(
              'No conflicts to resolve',
              style: TextStyle(fontSize: 18, fontWeight: FontWeight.w500),
            ),
            const SizedBox(height: 8),
            const Text(
              'All changes are in sync!',
              style: TextStyle(color: Colors.grey),
            ),
          ],
        ),
      );
    }

    return RefreshIndicator(
      onRefresh: _loadConflicts,
      child: ListView.builder(
        padding: const EdgeInsets.all(16),
        itemCount: _conflicts!.length,
        itemBuilder: (context, index) {
          final conflict = _conflicts![index];
          return ConflictCard(
            conflict: conflict,
            onResolve: (strategy) => _resolveConflict(conflict, strategy),
          );
        },
      ),
    );
  }
}

/// Conflict Card Widget
///
/// Displays a single conflict with expand/collapse for details
class ConflictCard extends StatelessWidget {
  final SyncConflict conflict;
  final Function(ResolutionStrategy) onResolve;

  const ConflictCard({
    Key? key,
    required this.conflict,
    required this.onResolve,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.only(bottom: 16),
      elevation: 2,
      child: ExpansionTile(
        leading: _buildSeverityIcon(),
        title: Text(
          conflict.conflictTypeName,
          style: const TextStyle(fontWeight: FontWeight.bold),
        ),
        subtitle: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const SizedBox(height: 4),
            Text('${conflict.resourceType} • ${conflict.timeAgo}'),
            const SizedBox(height: 4),
            Text(
              'Remote: ${conflict.remoteNodeId}',
              style: TextStyle(fontSize: 12, color: Colors.grey[600]),
            ),
          ],
        ),
        children: [
          _buildConflictDetails(context),
          _buildResolutionButtons(context),
        ],
      ),
    );
  }

  Widget _buildSeverityIcon() {
    IconData icon;
    Color color;

    switch (conflict.severity) {
      case ConflictSeverity.high:
        icon = Icons.error;
        color = Colors.red;
        break;
      case ConflictSeverity.medium:
        icon = Icons.warning;
        color = Colors.orange;
        break;
      case ConflictSeverity.low:
        icon = Icons.info;
        color = Colors.blue;
        break;
    }

    return Icon(icon, color: color, size: 32);
  }

  Widget _buildConflictDetails(BuildContext context) {
    final differences = conflict.getFieldDifferences();

    if (differences.isEmpty) {
      return const Padding(
        padding: EdgeInsets.all(16),
        child: Text('No field differences detected'),
      );
    }

    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Field Differences:',
            style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16),
          ),
          const SizedBox(height: 12),
          ...differences.entries.map((entry) => _buildFieldDifference(
                entry.key,
                entry.value,
              )),
        ],
      ),
    );
  }

  Widget _buildFieldDifference(String fieldName, FieldDifference diff) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  diff.displayName,
                  style: const TextStyle(
                    fontWeight: FontWeight.w500,
                    fontSize: 13,
                  ),
                ),
                const SizedBox(height: 4),
                Container(
                  padding: const EdgeInsets.all(8),
                  decoration: BoxDecoration(
                    color: Colors.blue[50],
                    borderRadius: BorderRadius.circular(4),
                    border: Border.all(color: Colors.blue[200]!),
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text(
                        'Local (This Device):',
                        style: TextStyle(
                          fontSize: 11,
                          fontWeight: FontWeight.bold,
                          color: Colors.blue,
                        ),
                      ),
                      const SizedBox(height: 2),
                      Text(
                        diff.formatValue(diff.localValue),
                        style: const TextStyle(fontSize: 12),
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 8),
                Container(
                  padding: const EdgeInsets.all(8),
                  decoration: BoxDecoration(
                    color: Colors.orange[50],
                    borderRadius: BorderRadius.circular(4),
                    border: Border.all(color: Colors.orange[200]!),
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'Remote (${conflict.remoteNodeId}):',
                        style: const TextStyle(
                          fontSize: 11,
                          fontWeight: FontWeight.bold,
                          color: Colors.orange,
                        ),
                      ),
                      const SizedBox(height: 2),
                      Text(
                        diff.formatValue(diff.remoteValue),
                        style: const TextStyle(fontSize: 12),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildResolutionButtons(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Row(
        children: [
          Expanded(
            child: OutlinedButton.icon(
              onPressed: () => _showConfirmDialog(
                context,
                ResolutionStrategy.localWins,
                'Keep your local changes?',
                'This will overwrite the remote version.',
              ),
              icon: const Icon(Icons.smartphone),
              label: const Text('Keep Local'),
              style: OutlinedButton.styleFrom(
                foregroundColor: Colors.blue,
                side: const BorderSide(color: Colors.blue),
              ),
            ),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: ElevatedButton.icon(
              onPressed: () => _showConfirmDialog(
                context,
                ResolutionStrategy.remoteWins,
                'Use remote changes?',
                'This will overwrite your local version.',
              ),
              icon: const Icon(Icons.cloud),
              label: const Text('Use Remote'),
              style: ElevatedButton.styleFrom(
                backgroundColor: Colors.orange,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Future<void> _showConfirmDialog(
    BuildContext context,
    ResolutionStrategy strategy,
    String title,
    String message,
  ) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(title),
        content: Text(message),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('Confirm'),
          ),
        ],
      ),
    );

    if (confirmed == true) {
      onResolve(strategy);
    }
  }
}
