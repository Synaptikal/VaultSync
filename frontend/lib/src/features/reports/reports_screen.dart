import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../providers/reports_provider.dart';

/// Reports Screen (Refactored to use ReportsProvider)
///
/// Fixes TASK-AUD-004 (FutureBuilder Anti-Pattern).
/// Uses ReportsProvider for state management and caching.

class ReportsScreen extends StatefulWidget {
  const ReportsScreen({super.key});

  @override
  State<ReportsScreen> createState() => _ReportsScreenState();
}

class _ReportsScreenState extends State<ReportsScreen>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 4, vsync: this);

    // Initial data load when screen opens
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final provider = context.read<ReportsProvider>();
      provider.loadSalesReport(
          startDate: DateTime.now().subtract(const Duration(days: 30)),
          endDate: DateTime.now());
      provider.loadInventoryValuation();
      provider.loadTopSellers();
      provider.loadLowStock();
    });
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Reports'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: 'Refresh All',
            onPressed: () {
              final provider = context.read<ReportsProvider>();
              if (_tabController.index == 0) {
                provider.loadSalesReport(
                    startDate:
                        DateTime.now().subtract(const Duration(days: 30)),
                    endDate: DateTime.now());
              } else if (_tabController.index == 1) {
                provider.loadInventoryValuation();
              } else if (_tabController.index == 2) {
                provider.loadTopSellers();
              } else {
                provider.loadLowStock();
              }
            },
          ),
        ],
        bottom: TabBar(
          controller: _tabController,
          isScrollable: true,
          tabs: const [
            Tab(icon: Icon(Icons.attach_money), text: 'Sales'),
            Tab(icon: Icon(Icons.inventory), text: 'Inventory Value'),
            Tab(icon: Icon(Icons.trending_up), text: 'Top Sellers'),
            Tab(icon: Icon(Icons.warning_amber), text: 'Low Stock'),
          ],
        ),
      ),
      body: TabBarView(
        controller: _tabController,
        children: const [
          _SalesReportTab(),
          _InventoryValuationTab(),
          _TopSellersTab(),
          _LowStockTab(),
        ],
      ),
    );
  }
}

class _SalesReportTab extends StatefulWidget {
  const _SalesReportTab();

  @override
  State<_SalesReportTab> createState() => _SalesReportTabState();
}

class _SalesReportTabState extends State<_SalesReportTab> {
  DateTimeRange? _dateRange;

  @override
  void initState() {
    super.initState();
    _dateRange = DateTimeRange(
      start: DateTime.now().subtract(const Duration(days: 30)),
      end: DateTime.now(),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Consumer<ReportsProvider>(builder: (context, provider, child) {
      final reportData = provider.salesReport;

      return Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Date Range Selector
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Row(
                  children: [
                    const Icon(Icons.calendar_today),
                    const SizedBox(width: 16),
                    Expanded(
                      child: _dateRange != null
                          ? Text(
                              '${_formatDate(_dateRange!.start)} - ${_formatDate(_dateRange!.end)}',
                              style: Theme.of(context).textTheme.titleMedium,
                            )
                          : const Text('Select date range'),
                    ),
                    ElevatedButton(
                      onPressed: () => _selectDateRange(context),
                      child: const Text('Change'),
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),

            // Report Content
            if (provider.isLoading && reportData == null)
              const Expanded(child: Center(child: CircularProgressIndicator()))
            else if (provider.error != null && reportData == null)
              Expanded(
                child: Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      const Icon(Icons.error_outline,
                          size: 48, color: Colors.red),
                      const SizedBox(height: 16),
                      Text('Error: ${provider.error}'),
                      const SizedBox(height: 16),
                      ElevatedButton(
                        onPressed: () => provider.loadSalesReport(
                            startDate: _dateRange?.start,
                            endDate: _dateRange?.end),
                        child: const Text('Retry'),
                      ),
                    ],
                  ),
                ),
              )
            else if (reportData != null)
              Expanded(
                child: RefreshIndicator(
                  onRefresh: () => provider.loadSalesReport(
                      startDate: _dateRange?.start, endDate: _dateRange?.end),
                  child: SingleChildScrollView(
                    physics: const AlwaysScrollableScrollPhysics(),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        // Summary Cards
                        Row(
                          children: [
                            Expanded(
                              child: _ReportStatCard(
                                title: 'Total Sales',
                                value:
                                    '\$${(reportData['total_sales'] ?? 0).toStringAsFixed(2)}',
                                icon: Icons.attach_money,
                                color: Colors.green,
                              ),
                            ),
                            const SizedBox(width: 16),
                            Expanded(
                              child: _ReportStatCard(
                                title: 'Transactions',
                                value:
                                    '${reportData['transaction_count'] ?? 0}',
                                icon: Icons.receipt_long,
                                color: Colors.blue,
                              ),
                            ),
                          ],
                        ),
                        const SizedBox(height: 16),
                        Row(
                          children: [
                            Expanded(
                              child: _ReportStatCard(
                                title: 'Average Sale',
                                value:
                                    '\$${(reportData['average_sale'] ?? 0).toStringAsFixed(2)}',
                                icon: Icons.analytics,
                                color: Colors.purple,
                              ),
                            ),
                            const SizedBox(width: 16),
                            Expanded(
                              child: _ReportStatCard(
                                title: 'Items Sold',
                                value: '${reportData['total_items'] ?? 0}',
                                icon: Icons.shopping_cart,
                                color: Colors.orange,
                              ),
                            ),
                          ],
                        ),
                        const SizedBox(height: 24),

                        // Top Sellers in this period
                        Text('Top Sellers',
                            style: Theme.of(context).textTheme.titleLarge),
                        const SizedBox(height: 8),
                        if (reportData['top_sellers'] != null)
                          ...(reportData['top_sellers'] as List).map((item) =>
                              ListTile(
                                leading: CircleAvatar(
                                  backgroundColor: Colors.green.shade100,
                                  child: Text('${item['rank'] ?? ''}'),
                                ),
                                title: Text(item['product_name'] ?? 'Unknown'),
                                subtitle:
                                    Text('Qty: ${item['quantity_sold'] ?? 0}'),
                                trailing: Text(
                                  '\$${(item['revenue'] ?? 0).toStringAsFixed(2)}',
                                  style: const TextStyle(
                                      fontWeight: FontWeight.bold),
                                ),
                              )),
                      ],
                    ),
                  ),
                ),
              ),
          ],
        ),
      );
    });
  }

  Future<void> _selectDateRange(BuildContext context) async {
    final picked = await showDateRangePicker(
      context: context,
      firstDate: DateTime(2020),
      lastDate: DateTime.now(),
      initialDateRange: _dateRange,
    );

    if (picked != null) {
      if (mounted) {
        setState(() {
          _dateRange = picked;
        });
        context.read<ReportsProvider>().loadSalesReport(
              startDate: picked.start,
              endDate: picked.end,
            );
      }
    }
  }

  String _formatDate(DateTime date) {
    return '${date.month}/${date.day}/${date.year}';
  }
}

class _InventoryValuationTab extends StatelessWidget {
  const _InventoryValuationTab();

  @override
  Widget build(BuildContext context) {
    return Consumer<ReportsProvider>(
      builder: (context, provider, child) {
        final data = provider.inventoryValuation;

        if (provider.isLoading && data == null) {
          return const Center(child: CircularProgressIndicator());
        }

        if (provider.error != null && data == null) {
          return Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Text('Error: ${provider.error}'),
                ElevatedButton(
                  onPressed: () => provider.loadInventoryValuation(),
                  child: const Text('Retry'),
                ),
              ],
            ),
          );
        }

        if (data == null) return const Center(child: Text('No data'));

        final totalValue = data['total_value'] ?? 0.0;
        final byCategory = (data['by_category'] as Map<String, dynamic>?) ?? {};
        final byCondition =
            (data['by_condition'] as Map<String, dynamic>?) ?? {};

        return RefreshIndicator(
          onRefresh: () => provider.loadInventoryValuation(),
          child: SingleChildScrollView(
            physics: const AlwaysScrollableScrollPhysics(),
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Total Value Card
                Card(
                  color: Colors.green.shade50,
                  child: Padding(
                    padding: const EdgeInsets.all(24.0),
                    child: Column(
                      children: [
                        const Icon(Icons.account_balance_wallet,
                            size: 48, color: Colors.green),
                        const SizedBox(height: 8),
                        Text(
                          'Total Inventory Value',
                          style: Theme.of(context).textTheme.titleMedium,
                        ),
                        const SizedBox(height: 4),
                        Text(
                          '\$${totalValue.toStringAsFixed(2)}',
                          style: Theme.of(context)
                              .textTheme
                              .headlineLarge
                              ?.copyWith(
                                fontWeight: FontWeight.bold,
                                color: Colors.green.shade800,
                              ),
                        ),
                      ],
                    ),
                  ),
                ),
                const SizedBox(height: 24),

                // By Category
                Text('Value by Category',
                    style: Theme.of(context).textTheme.titleLarge),
                const SizedBox(height: 8),
                Card(
                  child: Column(
                    children: byCategory.entries
                        .map((entry) => ListTile(
                              title: Text(entry.key),
                              trailing: Text(
                                '\$${(entry.value as num).toStringAsFixed(2)}',
                                style: const TextStyle(
                                    fontWeight: FontWeight.bold),
                              ),
                            ))
                        .toList(),
                  ),
                ),
                const SizedBox(height: 24),

                // By Condition
                Text('Value by Condition',
                    style: Theme.of(context).textTheme.titleLarge),
                const SizedBox(height: 8),
                Card(
                  child: Column(
                    children: byCondition.entries
                        .map((entry) => ListTile(
                              leading: _getConditionIcon(entry.key),
                              title: Text(entry.key),
                              trailing: Text(
                                '\$${(entry.value as num).toStringAsFixed(2)}',
                                style: const TextStyle(
                                    fontWeight: FontWeight.bold),
                              ),
                            ))
                        .toList(),
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _getConditionIcon(String condition) {
    final color = switch (condition.toLowerCase()) {
      'nm' || 'near_mint' => Colors.green,
      'lp' || 'lightly_played' => Colors.lightGreen,
      'mp' || 'moderately_played' => Colors.yellow.shade700,
      'hp' || 'heavily_played' => Colors.orange,
      'dmg' || 'damaged' => Colors.red,
      _ => Colors.grey,
    };
    return CircleAvatar(
      radius: 16,
      backgroundColor: color.withAlpha(51),
      child: Icon(Icons.star, color: color, size: 18),
    );
  }
}

class _TopSellersTab extends StatelessWidget {
  const _TopSellersTab();

  @override
  Widget build(BuildContext context) {
    return Consumer<ReportsProvider>(
      builder: (context, provider, child) {
        final data = provider.topSellers;

        if (provider.isLoading && data == null) {
          return const Center(child: CircularProgressIndicator());
        }

        if (provider.error != null && data == null) {
          return Center(child: Text('Error: ${provider.error}'));
        }

        if (data == null) return const Center(child: Text('No data'));

        final products = (data['products'] as List?) ?? [];

        return RefreshIndicator(
          onRefresh: () => provider.loadTopSellers(),
          child: ListView.builder(
            physics: const AlwaysScrollableScrollPhysics(),
            padding: const EdgeInsets.all(16),
            itemCount: products.length,
            itemBuilder: (context, index) {
              final product = products[index] as Map<String, dynamic>;
              return Card(
                child: ListTile(
                  leading: CircleAvatar(
                    backgroundColor:
                        index < 3 ? Colors.amber : Colors.grey.shade300,
                    child: Text(
                      '${index + 1}',
                      style: TextStyle(
                        fontWeight: FontWeight.bold,
                        color: index < 3 ? Colors.white : Colors.black,
                      ),
                    ),
                  ),
                  title: Text(product['product_name'] ?? 'Unknown'),
                  subtitle: Text('Category: ${product['category'] ?? 'N/A'}'),
                  trailing: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    crossAxisAlignment: CrossAxisAlignment.end,
                    children: [
                      Text(
                        '\$${(product['revenue'] ?? 0).toStringAsFixed(2)}',
                        style: const TextStyle(
                            fontWeight: FontWeight.bold, fontSize: 16),
                      ),
                      Text(
                        '${product['quantity_sold'] ?? 0} sold',
                        style: Theme.of(context).textTheme.bodySmall,
                      ),
                    ],
                  ),
                ),
              );
            },
          ),
        );
      },
    );
  }
}

class _LowStockTab extends StatelessWidget {
  const _LowStockTab();

  @override
  Widget build(BuildContext context) {
    return Consumer<ReportsProvider>(
      builder: (context, provider, child) {
        final data = provider.lowStock;

        if (provider.isLoading && data == null) {
          return const Center(child: CircularProgressIndicator());
        }

        if (provider.error != null && data == null) {
          return Center(child: Text('Error: ${provider.error}'));
        }

        if (data == null) return const Center(child: Text('No data'));

        final items = (data['items'] as List?) ?? [];

        if (items.isEmpty) {
          return RefreshIndicator(
            onRefresh: () => provider.loadLowStock(),
            child: SingleChildScrollView(
              physics: const AlwaysScrollableScrollPhysics(),
              child: SizedBox(
                height: MediaQuery.of(context).size.height - 200,
                child: const Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.check_circle, size: 64, color: Colors.green),
                      SizedBox(height: 16),
                      Text('No low stock items!',
                          style: TextStyle(fontSize: 18)),
                      Text('All inventory levels are healthy.'),
                    ],
                  ),
                ),
              ),
            ),
          );
        }

        return RefreshIndicator(
          onRefresh: () => provider.loadLowStock(),
          child: ListView.builder(
            physics: const AlwaysScrollableScrollPhysics(),
            padding: const EdgeInsets.all(16),
            itemCount: items.length,
            itemBuilder: (context, index) {
              final item = items[index] as Map<String, dynamic>;
              final quantity = item['quantity_on_hand'] ?? 0;
              final urgency = quantity == 0
                  ? 'Critical'
                  : quantity <= 2
                      ? 'Low'
                      : 'Warning';
              final urgencyColor = quantity == 0
                  ? Colors.red
                  : quantity <= 2
                      ? Colors.orange
                      : Colors.yellow.shade700;

              return Card(
                child: ListTile(
                  leading: CircleAvatar(
                    backgroundColor: urgencyColor.withAlpha(51),
                    child: Icon(
                      quantity == 0 ? Icons.error : Icons.warning,
                      color: urgencyColor,
                    ),
                  ),
                  title: Text(item['product_name'] ?? 'Unknown'),
                  subtitle: Text('Condition: ${item['condition'] ?? 'N/A'}'),
                  trailing: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    crossAxisAlignment: CrossAxisAlignment.end,
                    children: [
                      Text(
                        '$quantity',
                        style: TextStyle(
                          fontWeight: FontWeight.bold,
                          fontSize: 20,
                          color: urgencyColor,
                        ),
                      ),
                      Container(
                        padding: const EdgeInsets.symmetric(
                            horizontal: 8, vertical: 2),
                        decoration: BoxDecoration(
                          color: urgencyColor.withAlpha(51),
                          borderRadius: BorderRadius.circular(4),
                        ),
                        child: Text(
                          urgency,
                          style: TextStyle(
                              fontSize: 10,
                              color: urgencyColor,
                              fontWeight: FontWeight.bold),
                        ),
                      ),
                    ],
                  ),
                ),
              );
            },
          ),
        );
      },
    );
  }
}

class _ReportStatCard extends StatelessWidget {
  final String title;
  final String value;
  final IconData icon;
  final Color color;

  const _ReportStatCard({
    required this.title,
    required this.value,
    required this.icon,
    required this.color,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          children: [
            Icon(icon, size: 32, color: color),
            const SizedBox(height: 8),
            Text(title, style: Theme.of(context).textTheme.bodySmall),
            const SizedBox(height: 4),
            Text(
              value,
              style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                    fontWeight: FontWeight.bold,
                    color: color,
                  ),
            ),
          ],
        ),
      ),
    );
  }
}
