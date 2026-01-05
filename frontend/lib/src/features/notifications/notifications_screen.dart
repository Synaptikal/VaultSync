import 'package:flutter/material.dart';

class NotificationsScreen extends StatefulWidget {
  const NotificationsScreen({super.key});

  @override
  State<NotificationsScreen> createState() => _NotificationsScreenState();
}

class _NotificationsScreenState extends State<NotificationsScreen>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;

  // Simulated notifications - in production these would come from the backend
  final List<Map<String, dynamic>> _allNotifications = [
    {
      'id': '1',
      'title': 'Sync Completed',
      'message': 'Successfully synced 14 items with the main server.',
      'time': DateTime.now().subtract(const Duration(minutes: 2)),
      'type': 'sync',
      'read': false,
    },
    {
      'id': '2',
      'title': 'Low Stock Warning',
      'message': 'Product "Black Lotus" is down to 1 unit.',
      'time': DateTime.now().subtract(const Duration(hours: 1)),
      'type': 'inventory',
      'read': false,
    },
    {
      'id': '3',
      'title': '🎯 Wants List Match!',
      'message':
          'John Smith is looking for "Charizard VMAX" - just acquired in buylist!',
      'time': DateTime.now().subtract(const Duration(minutes: 30)),
      'type': 'wants',
      'read': false,
      'customer_uuid': 'abc-123',
      'product_name': 'Charizard VMAX',
    },
    {
      'id': '4',
      'title': '🎯 Wants List Match!',
      'message':
          'Sarah Connor wants "Pikachu VMAX" (NM) - max \$50. You just got one for \$35!',
      'time': DateTime.now().subtract(const Duration(hours: 2)),
      'type': 'wants',
      'read': true,
      'customer_uuid': 'def-456',
      'product_name': 'Pikachu VMAX',
    },
    {
      'id': '5',
      'title': 'Event Reminder',
      'message':
          'Friday Night Magic starts in 2 hours. 12 participants registered.',
      'time': DateTime.now().subtract(const Duration(hours: 4)),
      'type': 'event',
      'read': true,
    },
    {
      'id': '6',
      'title': 'Price Alert',
      'message': 'Market value for "Underground Sea" dropped 15% in 24h.',
      'time': DateTime.now().subtract(const Duration(hours: 5)),
      'type': 'pricing',
      'read': true,
    },
  ];

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 3, vsync: this);
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  List<Map<String, dynamic>> _getFilteredNotifications(String filter) {
    if (filter == 'all') return _allNotifications;
    if (filter == 'wants') {
      return _allNotifications.where((n) => n['type'] == 'wants').toList();
    }
    if (filter == 'unread') {
      return _allNotifications.where((n) => n['read'] == false).toList();
    }
    return _allNotifications;
  }

  void _markAsRead(String id) {
    setState(() {
      final notif = _allNotifications.firstWhere((n) => n['id'] == id);
      notif['read'] = true;
    });
  }

  void _markAllAsRead() {
    setState(() {
      for (var notif in _allNotifications) {
        notif['read'] = true;
      }
    });
  }

  String _formatTime(DateTime time) {
    final diff = DateTime.now().difference(time);
    if (diff.inMinutes < 60) return '${diff.inMinutes}m ago';
    if (diff.inHours < 24) return '${diff.inHours}h ago';
    return '${diff.inDays}d ago';
  }

  @override
  Widget build(BuildContext context) {
    final unreadCount =
        _allNotifications.where((n) => n['read'] == false).length;
    final wantsCount =
        _allNotifications.where((n) => n['type'] == 'wants').length;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Notifications'),
        actions: [
          if (unreadCount > 0)
            TextButton.icon(
              onPressed: _markAllAsRead,
              icon: const Icon(Icons.done_all, color: Colors.white),
              label: const Text('Mark All Read',
                  style: TextStyle(color: Colors.white)),
            ),
        ],
        bottom: TabBar(
          controller: _tabController,
          tabs: [
            Tab(text: 'All (${_allNotifications.length})'),
            Tab(
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Text('Wants'),
                  if (wantsCount > 0) ...[
                    const SizedBox(width: 4),
                    Container(
                      padding: const EdgeInsets.symmetric(
                          horizontal: 6, vertical: 2),
                      decoration: BoxDecoration(
                        color: Colors.green,
                        borderRadius: BorderRadius.circular(10),
                      ),
                      child: Text('$wantsCount',
                          style: const TextStyle(
                              fontSize: 12, color: Colors.white)),
                    ),
                  ],
                ],
              ),
            ),
            Tab(
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Text('Unread'),
                  if (unreadCount > 0) ...[
                    const SizedBox(width: 4),
                    Container(
                      padding: const EdgeInsets.symmetric(
                          horizontal: 6, vertical: 2),
                      decoration: BoxDecoration(
                        color: Colors.red,
                        borderRadius: BorderRadius.circular(10),
                      ),
                      child: Text('$unreadCount',
                          style: const TextStyle(
                              fontSize: 12, color: Colors.white)),
                    ),
                  ],
                ],
              ),
            ),
          ],
        ),
      ),
      body: TabBarView(
        controller: _tabController,
        children: [
          _buildNotificationList('all'),
          _buildNotificationList('wants'),
          _buildNotificationList('unread'),
        ],
      ),
    );
  }

  Widget _buildNotificationList(String filter) {
    final notifications = _getFilteredNotifications(filter);

    if (notifications.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              filter == 'wants' ? Icons.checklist : Icons.notifications_none,
              size: 64,
              color: Colors.grey.shade400,
            ),
            const SizedBox(height: 16),
            Text(
              filter == 'wants' ? 'No wants matches yet' : 'All caught up!',
              style: const TextStyle(fontSize: 18, color: Colors.grey),
            ),
            if (filter == 'wants')
              const Padding(
                padding: EdgeInsets.all(16),
                child: Text(
                  'When a customer\'s wanted item comes in through a buylist,\nyou\'ll be notified here.',
                  textAlign: TextAlign.center,
                  style: TextStyle(color: Colors.grey),
                ),
              ),
          ],
        ),
      );
    }

    return ListView.separated(
      itemCount: notifications.length,
      separatorBuilder: (_, __) => const Divider(height: 1),
      itemBuilder: (context, index) {
        final n = notifications[index];
        return _NotificationTile(
          notification: n,
          onTap: () => _markAsRead(n['id']),
          formatTime: _formatTime,
        );
      },
    );
  }
}

class _NotificationTile extends StatelessWidget {
  final Map<String, dynamic> notification;
  final VoidCallback onTap;
  final String Function(DateTime) formatTime;

  const _NotificationTile({
    required this.notification,
    required this.onTap,
    required this.formatTime,
  });

  @override
  Widget build(BuildContext context) {
    final isRead = notification['read'] as bool;
    final type = notification['type'] as String;

    IconData icon;
    Color iconColor;

    switch (type) {
      case 'wants':
        icon = Icons.stars;
        iconColor = Colors.green;
        break;
      case 'sync':
        icon = Icons.sync;
        iconColor = Colors.blue;
        break;
      case 'inventory':
        icon = Icons.inventory;
        iconColor = Colors.orange;
        break;
      case 'event':
        icon = Icons.event;
        iconColor = Colors.purple;
        break;
      case 'pricing':
        icon = Icons.trending_down;
        iconColor = Colors.red;
        break;
      default:
        icon = Icons.notifications;
        iconColor = Colors.grey;
    }

    return Container(
      color: isRead ? null : Colors.blue.withValues(alpha: 0.05),
      child: ListTile(
        leading: Stack(
          children: [
            CircleAvatar(
              backgroundColor: iconColor.withValues(alpha: 0.2),
              child: Icon(icon, color: iconColor),
            ),
            if (!isRead)
              Positioned(
                right: 0,
                top: 0,
                child: Container(
                  width: 12,
                  height: 12,
                  decoration: const BoxDecoration(
                    color: Colors.red,
                    shape: BoxShape.circle,
                  ),
                ),
              ),
          ],
        ),
        title: Text(
          notification['title'],
          style: TextStyle(
              fontWeight: isRead ? FontWeight.normal : FontWeight.bold),
        ),
        subtitle: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(notification['message']),
            const SizedBox(height: 4),
            Text(
              formatTime(notification['time']),
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ),
        isThreeLine: true,
        trailing: type == 'wants'
            ? FilledButton(
                onPressed: () {
                  // Navigate to customer or create contact
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(
                        content: Text(
                            'Contact customer about: ${notification['product_name']}')),
                  );
                },
                child: const Text('Contact'),
              )
            : const Icon(Icons.chevron_right),
        onTap: onTap,
      ),
    );
  }
}
