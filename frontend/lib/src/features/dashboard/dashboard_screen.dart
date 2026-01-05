import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';
import '../../providers/product_provider.dart';
import '../../providers/customer_provider.dart';
import '../../services/api_service.dart';
import '../../services/offline_sync_service.dart';
import '../../shared/sync_status_widget.dart';

class DashboardScreen extends StatefulWidget {
  const DashboardScreen({super.key});

  @override
  State<DashboardScreen> createState() => _DashboardScreenState();
}

class _DashboardScreenState extends State<DashboardScreen> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      context.read<ProductProvider>().loadProducts();
      context.read<CustomerProvider>().loadCustomers();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Dashboard'),
        actions: const [
          // Use the new SyncIndicator widget
          SyncIndicator(),
        ],
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Sync Status Widget (shows when offline or pending changes)
            Consumer<OfflineSyncService>(
              builder: (context, syncService, child) {
                if (!syncService.isOnline || syncService.hasPendingChanges) {
                  return const Column(
                    children: [
                      SyncStatusWidget(),
                      SizedBox(height: 16),
                    ],
                  );
                }
                return const SizedBox.shrink();
              },
            ),

            // Pricing Dashboard Widget
            const _PricingDashboardWidget(),
            const SizedBox(height: 24),

            // Stats Row - "Graded Slabs" Style
            LayoutBuilder(
              builder: (context, constraints) {
                final isWide = constraints.maxWidth > 600;
                return Row(
                  children: [
                    Expanded(
                      child: _SlabStatCard(
                        title: 'Total Products',
                        label: 'INVENTORY',
                        grade: '10', // Gem Mint 10
                        icon: Icons.inventory_2,
                        color: Colors.blueAccent,
                        valueBuilder: (context) => Consumer<ProductProvider>(
                          builder: (context, provider, _) => Text(
                            '${provider.products.length}',
                            style: Theme.of(context)
                                .textTheme
                                .headlineMedium
                                ?.copyWith(
                                  fontWeight: FontWeight.bold,
                                  color: Colors.black87,
                                ),
                          ),
                        ),
                      ),
                    ),
                    const SizedBox(width: 16),
                    Expanded(
                      child: _SlabStatCard(
                        title: 'Total Customers',
                        label: 'CRM',
                        grade: '9.5', // Mint 9.5
                        icon: Icons.people,
                        color: Colors.amber,
                        valueBuilder: (context) => Consumer<CustomerProvider>(
                          builder: (context, provider, _) => Text(
                            '${provider.customers.length}',
                            style: Theme.of(context)
                                .textTheme
                                .headlineMedium
                                ?.copyWith(
                                  fontWeight: FontWeight.bold,
                                  color: Colors.black87,
                                ),
                          ),
                        ),
                      ),
                    ),
                    if (isWide) ...[
                      const SizedBox(width: 16),
                      Expanded(
                        child: _SlabStatCard(
                          title: 'Daily Sales',
                          label: 'REVENUE',
                          grade: 'PR', // Pristine
                          icon: Icons.attach_money,
                          color: Colors.green,
                          valueBuilder: (context) => Text(
                            '\$1,250', // Placeholder
                            style: Theme.of(context)
                                .textTheme
                                .headlineMedium
                                ?.copyWith(
                                  fontWeight: FontWeight.bold,
                                  color: Colors.black87,
                                ),
                          ),
                        ),
                      ),
                    ],
                  ],
                );
              },
            ),
            const SizedBox(height: 24),

            // Charts Section (Placeholder)
            Text('Market Analytics',
                style: Theme.of(context).textTheme.titleLarge),
            const SizedBox(height: 16),
            Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: Theme.of(context).cardColor,
                borderRadius: BorderRadius.circular(16),
                border: Border.all(color: Colors.grey.withAlpha(51)),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      const Icon(Icons.analytics, color: Colors.indigo),
                      const SizedBox(width: 8),
                      Text('Daily Snapshot',
                          style: Theme.of(context).textTheme.titleLarge),
                    ],
                  ),
                  const SizedBox(height: 16),
                  Row(
                    children: [
                      const Expanded(
                        child: _SnapshotItem(
                          label: 'Gross Sales',
                          value: '\$1,450.00',
                          trend: '+12%',
                          isPositive: true,
                        ),
                      ),
                      _VerticalDivider(),
                      const Expanded(
                        child: _SnapshotItem(
                          label: 'Net Profit',
                          value: '\$320.50',
                          trend: '+8%',
                          isPositive: true,
                        ),
                      ),
                      _VerticalDivider(),
                      const Expanded(
                        child: _SnapshotItem(
                          label: 'Low Stock',
                          value: '12 Items',
                          trend: 'Attention',
                          isPositive: false,
                          isAlert: true,
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
            const SizedBox(height: 24),

            // Quick Actions
            Text('Quick Actions',
                style: Theme.of(context).textTheme.titleLarge),
            const SizedBox(height: 16),
            GridView.count(
              shrinkWrap: true,
              physics: const NeverScrollableScrollPhysics(),
              crossAxisCount: MediaQuery.of(context).size.width > 600 ? 5 : 3,
              crossAxisSpacing: 16,
              mainAxisSpacing: 16,
              children: [
                _ActionCard(
                  title: 'New Sale',
                  icon: Icons.point_of_sale,
                  color: Colors.green.shade100,
                  iconColor: Colors.green.shade800,
                  onTap: () => context.go('/pos'),
                ),
                _ActionCard(
                  title: 'Add Product',
                  icon: Icons.add_box,
                  color: Colors.blue.shade100,
                  iconColor: Colors.blue.shade800,
                  onTap: () => context.go('/inventory'),
                ),
                _ActionCard(
                  title: 'Add Customer',
                  icon: Icons.person_add,
                  color: Colors.orange.shade100,
                  iconColor: Colors.orange.shade800,
                  onTap: () => context.go('/customers'),
                ),
                _ActionCard(
                  title: 'Admin',
                  icon: Icons.admin_panel_settings,
                  color: Colors.red.shade100,
                  iconColor: Colors.red.shade800,
                  onTap: () => context.go('/admin'),
                ),
                _ActionCard(
                  title: 'Reports',
                  icon: Icons.bar_chart,
                  color: Colors.purple.shade100,
                  iconColor: Colors.purple.shade800,
                  onTap: () => context.go('/reports'),
                ),
                _ActionCard(
                  title: 'Transactions',
                  icon: Icons.receipt_long,
                  color: Colors.teal.shade100,
                  iconColor: Colors.teal.shade800,
                  onTap: () => context.go('/transactions'),
                ),
                _ActionCard(
                  title: 'Cash Drawer',
                  icon: Icons.point_of_sale,
                  color: Colors.cyan.shade100,
                  iconColor: Colors.cyan.shade800,
                  onTap: () => context.go('/cash-drawer'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class _PricingDashboardWidget extends StatelessWidget {
  const _PricingDashboardWidget();

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<Map<String, dynamic>>(
      future:
          Provider.of<ApiService>(context, listen: false).getPricingDashboard(),
      builder: (context, snapshot) {
        if (snapshot.connectionState == ConnectionState.waiting) {
          return const LinearProgressIndicator();
        }
        if (snapshot.hasError) {
          return Text('Pricing Data Unavailable: ${snapshot.error}',
              style: const TextStyle(color: Colors.red));
        }

        final data = snapshot.data;
        if (data == null) return const SizedBox.shrink();

        final trends = data['market_trends'] as Map<String, dynamic>;
        final alerts =
            (data['volatility_alerts'] as List).cast<Map<String, dynamic>>();

        return Card(
          elevation: 2,
          child: Padding(
            padding: const EdgeInsets.all(16.0),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Text('Market Pulse',
                        style: Theme.of(context).textTheme.titleLarge),
                    Text('Last Sync: ${data['last_sync']}',
                        style: Theme.of(context).textTheme.bodySmall),
                  ],
                ),
                const SizedBox(height: 16),
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceAround,
                  children: [
                    _TrendItem(
                        icon: Icons.trending_up,
                        color: Colors.green,
                        label: 'Up',
                        value: '${trends['up']}'),
                    _TrendItem(
                        icon: Icons.trending_flat,
                        color: Colors.grey,
                        label: 'Stable',
                        value: '${trends['stable']}'),
                    _TrendItem(
                        icon: Icons.trending_down,
                        color: Colors.red,
                        label: 'Down',
                        value: '${trends['down']}'),
                  ],
                ),
                if (alerts.isNotEmpty) ...[
                  const Divider(height: 24),
                  const Text('Volatility Alerts',
                      style: TextStyle(fontWeight: FontWeight.bold)),
                  const SizedBox(height: 8),
                  ...alerts.map((alert) => ListTile(
                        dense: true,
                        contentPadding: EdgeInsets.zero,
                        leading: Icon(
                          alert['direction'] == 'up'
                              ? Icons.arrow_upward
                              : Icons.arrow_downward,
                          color: alert['direction'] == 'up'
                              ? Colors.red
                              : Colors
                                  .green, // Volatility is risk usually, but up is good for inventory value? Let's use red for extreme volatility.
                        ),
                        title: Text(alert['product_name']),
                        trailing: Text(
                          '${alert['change_percent']}%',
                          style: TextStyle(
                            fontWeight: FontWeight.bold,
                            color: alert['direction'] == 'up'
                                ? Colors.green
                                : Colors.red,
                          ),
                        ),
                      )),
                ],
              ],
            ),
          ),
        );
      },
    );
  }
}

class _TrendItem extends StatelessWidget {
  final IconData icon;
  final Color color;
  final String label;
  final String value;

  const _TrendItem(
      {required this.icon,
      required this.color,
      required this.label,
      required this.value});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Icon(icon, color: color, size: 32),
        const SizedBox(height: 4),
        Text(value, style: Theme.of(context).textTheme.headlineSmall),
        Text(label, style: Theme.of(context).textTheme.bodySmall),
      ],
    );
  }
}

class _SlabStatCard extends StatelessWidget {
  final String title;
  final String label;
  final String grade;
  final IconData icon;
  final Color color;
  final WidgetBuilder valueBuilder;

  const _SlabStatCard({
    required this.title,
    required this.label,
    required this.grade,
    required this.icon,
    required this.color,
    required this.valueBuilder,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: Colors.grey.shade300, width: 2),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withAlpha(13),
            blurRadius: 10,
            offset: const Offset(0, 4),
          ),
        ],
      ),
      child: Column(
        children: [
          // Header (Slab Label)
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: BoxDecoration(
              color: color.withAlpha(25),
              borderRadius:
                  const BorderRadius.vertical(top: Radius.circular(10)),
              border: Border(bottom: BorderSide(color: color.withAlpha(77))),
            ),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        label,
                        style: TextStyle(
                          fontSize: 10,
                          fontWeight: FontWeight.bold,
                          color: color.withAlpha(204),
                          letterSpacing: 1.2,
                        ),
                      ),
                      Text(
                        title,
                        style: const TextStyle(
                          fontSize: 12,
                          fontWeight: FontWeight.w600,
                          color: Colors.black87,
                        ),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ],
                  ),
                ),
                Container(
                  padding:
                      const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                  decoration: BoxDecoration(
                    color: Colors.white,
                    borderRadius: BorderRadius.circular(4),
                    border: Border.all(color: color.withAlpha(128)),
                  ),
                  child: Text(
                    grade,
                    style: TextStyle(
                      fontWeight: FontWeight.bold,
                      color: color,
                    ),
                  ),
                ),
              ],
            ),
          ),
          // Body
          Padding(
            padding: const EdgeInsets.all(16.0),
            child: Row(
              children: [
                Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: Colors.grey.shade100,
                    shape: BoxShape.circle,
                  ),
                  child: Icon(icon, color: color),
                ),
                const SizedBox(width: 16),
                Expanded(child: valueBuilder(context)),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _ActionCard extends StatelessWidget {
  final String title;
  final IconData icon;
  final VoidCallback onTap;
  final Color color;
  final Color iconColor;

  const _ActionCard({
    required this.title,
    required this.icon,
    required this.onTap,
    required this.color,
    required this.iconColor,
  });

  @override
  Widget build(BuildContext context) {
    return Material(
      color: color,
      borderRadius: BorderRadius.circular(16),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(16),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(icon, size: 32, color: iconColor),
            const SizedBox(height: 8),
            Text(
              title,
              style: TextStyle(
                fontWeight: FontWeight.bold,
                color: iconColor,
              ),
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
    );
  }
}

class _SnapshotItem extends StatelessWidget {
  final String label;
  final String value;
  final String trend;
  final bool isPositive;
  final bool isAlert;

  const _SnapshotItem({
    required this.label,
    required this.value,
    required this.trend,
    this.isPositive = true,
    this.isAlert = false,
  });

  @override
  Widget build(BuildContext context) {
    Color trendColor =
        isAlert ? Colors.orange : (isPositive ? Colors.green : Colors.red);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label, style: const TextStyle(color: Colors.grey)),
        const SizedBox(height: 4),
        Text(value,
            style: const TextStyle(fontSize: 24, fontWeight: FontWeight.bold)),
        const SizedBox(height: 4),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
          decoration: BoxDecoration(
            color: trendColor.withAlpha(25),
            borderRadius: BorderRadius.circular(4),
          ),
          child: Text(
            trend,
            style: TextStyle(
              color: trendColor,
              fontWeight: FontWeight.bold,
              fontSize: 12,
            ),
          ),
        ),
      ],
    );
  }
}

class _VerticalDivider extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Container(
      height: 40,
      width: 1,
      color: Colors.grey.withAlpha(51),
      margin: const EdgeInsets.symmetric(horizontal: 16),
    );
  }
}
