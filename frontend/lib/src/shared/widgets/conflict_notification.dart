import 'package:flutter/material.dart';
import '../../services/api_client.dart';
import '../../models/sync_conflict.dart';
import '../../features/sync/screens/conflict_resolution_screen.dart';

/// Conflict Notification Badge (PHASE 4)
///
/// Shows a badge when conflicts are detected.
/// Taps navigate to conflict resolution screen.
///
/// Usage:
/// ```dart
/// AppBar(
///   actions: [
///     ConflictNotificationBadge(apiClient: apiClient),
///   ],
/// )
/// ```

class ConflictNotificationBadge extends StatefulWidget {
  final ApiClient apiClient;
  final Duration pollingInterval;

  const ConflictNotificationBadge({
    Key? key,
    required this.apiClient,
    this.pollingInterval = const Duration(minutes: 1),
  }) : super(key: key);

  @override
  State<ConflictNotificationBadge> createState() =>
      _ConflictNotificationBadgeState();
}

class _ConflictNotificationBadgeState extends State<ConflictNotificationBadge> {
  int _conflictCount = 0;
  bool _isLoading = false;

  @override
  void initState() {
    super.initState();
    _checkConflicts();
    _startPolling();
  }

  void _startPolling() {
    Future.delayed(widget.pollingInterval, () {
      if (mounted) {
        _checkConflicts();
        _startPolling();
      }
    });
  }

  Future<void> _checkConflicts() async {
    if (_isLoading) return;

    setState(() {
      _isLoading = true;
    });

    try {
      final data = await widget.apiClient.getPendingConflicts();

      if (mounted) {
        setState(() {
          _conflictCount = data.length;
          _isLoading = false;
        });
      }
    } catch (e) {
      // Silent fail - don't interrupt user
      if (mounted) {
        setState(() {
          _isLoading = false;
        });
      }
    }
  }

  void _navigateToResolution() {
    Navigator.of(context)
        .push(
      MaterialPageRoute(
        builder: (context) => ConflictResolutionScreen(
          apiClient: widget.apiClient,
        ),
      ),
    )
        .then((_) {
      // Refresh count when returning
      _checkConflicts();
    });
  }

  @override
  Widget build(BuildContext context) {
    if (_conflictCount == 0) {
      return const SizedBox.shrink();
    }

    return IconButton(
      icon: Stack(
        children: [
          const Icon(Icons.warning_amber_rounded),
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
                minWidth: 16,
                minHeight: 16,
              ),
              child: Text(
                _conflictCount > 9 ? '9+' : '$_conflictCount',
                style: const TextStyle(
                  color: Colors.white,
                  fontSize: 10,
                  fontWeight: FontWeight.bold,
                ),
                textAlign: TextAlign.center,
              ),
            ),
          ),
        ],
      ),
      onPressed: _navigateToResolution,
      tooltip:
          '$_conflictCount conflict${_conflictCount > 1 ? 's' : ''} to resolve',
      color: Colors.orange,
    );
  }
}

/// Conflict Alert Dialog
///
/// Shows a dialog when a conflict is detected
class ConflictAlertDialog extends StatelessWidget {
  final int conflictCount;
  final VoidCallback onResolve;

  const ConflictAlertDialog({
    Key? key,
    required this.conflictCount,
    required this.onResolve,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      icon: const Icon(Icons.warning_amber_rounded,
          color: Colors.orange, size: 48),
      title: Text(
          '$conflictCount Sync Conflict${conflictCount > 1 ? 's' : ''} Detected'),
      content: Text(
        'Changes made on different terminals are in conflict. '
        'Please review and resolve to continue syncing.',
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Later'),
        ),
        ElevatedButton(
          onPressed: () {
            Navigator.of(context).pop();
            onResolve();
          },
          style: ElevatedButton.styleFrom(
            backgroundColor: Colors.orange,
          ),
          child: const Text('Resolve Now'),
        ),
      ],
    );
  }

  /// Show the dialog
  static void show(
      BuildContext context, int conflictCount, VoidCallback onResolve) {
    showDialog(
      context: context,
      builder: (context) => ConflictAlertDialog(
        conflictCount: conflictCount,
        onResolve: onResolve,
      ),
    );
  }
}

/// Conflict Summary Card
///
/// Compact card showing conflict summary
class ConflictSummaryCard extends StatelessWidget {
  final List<SyncConflict> conflicts;
  final VoidCallback onTap;

  const ConflictSummaryCard({
    Key? key,
    required this.conflicts,
    required this.onTap,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    if (conflicts.isEmpty) return const SizedBox.shrink();

    final highSeverity =
        conflicts.where((c) => c.severity == ConflictSeverity.high).length;
    final mediumSeverity =
        conflicts.where((c) => c.severity == ConflictSeverity.medium).length;

    return Card(
      color: Colors.orange[50],
      margin: const EdgeInsets.all(16),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              const Icon(
                Icons.warning_amber_rounded,
                color: Colors.orange,
                size: 48,
              ),
              const SizedBox(width: 16),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      '${conflicts.length} Sync Conflict${conflicts.length > 1 ? 's' : ''}',
                      style: const TextStyle(
                        fontWeight: FontWeight.bold,
                        fontSize: 16,
                      ),
                    ),
                    const SizedBox(height: 4),
                    if (highSeverity > 0)
                      Text(
                        '$highSeverity critical',
                        style: const TextStyle(color: Colors.red),
                      ),
                    if (mediumSeverity > 0)
                      Text(
                        '$mediumSeverity requiring attention',
                        style: const TextStyle(color: Colors.orange),
                      ),
                    const SizedBox(height: 8),
                    const Text(
                      'Tap to resolve →',
                      style: TextStyle(
                        color: Colors.blue,
                        fontWeight: FontWeight.w500,
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
