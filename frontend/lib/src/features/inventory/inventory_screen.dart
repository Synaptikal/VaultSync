import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../providers/inventory_provider.dart';
import 'widgets/inventory_matrix_view.dart';
import 'widgets/receive_stock_dialog.dart';
import 'widgets/inventory_detail_dialog.dart';
import '../pricing/pricing_screen.dart';

/// Inventory Screen (TASK-AUD-001b: Refactored to use Repository Pattern)
///
/// Now uses InventoryProvider instead of direct ApiService calls.
/// Benefits:
/// - Works offline (loads from local DB)
/// - Proper state management (no FutureBuilder in build())
/// - Shows sync status
/// - Better error handling

class InventoryScreen extends StatefulWidget {
  const InventoryScreen({super.key});

  @override
  State<InventoryScreen> createState() => _InventoryScreenState();
}

class _InventoryScreenState extends State<InventoryScreen>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 4, vsync: this);
    
    // Load inventory data on screen init (proper pattern)
    WidgetsBinding.instance.addPostFrameCallback((_) {
      context.read<InventoryProvider>().loadInventory();
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
        title: const Text('Inventory Management'),
        actions: [
          // Sync status indicator
          Consumer<InventoryProvider>(
            builder: (context, provider, _) {
              final stats = provider.syncStats;
              final unsynced = stats['unsynced'] ?? 0;
              if (unsynced > 0) {
                return Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 8),
                  child: Chip(
                    avatar: const Icon(Icons.sync, size: 16),
                    label: Text('$unsynced pending'),
                    backgroundColor: Colors.orange.shade100,
                  ),
                );
              }
              return const SizedBox.shrink();
            },
          ),
          // Refresh button
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: () => context.read<InventoryProvider>().refresh(),
            tooltip: 'Refresh from server',
          ),
        ],
        bottom: TabBar(
          controller: _tabController,
          tabs: const [
            Tab(icon: Icon(Icons.dashboard), text: 'Overview'),
            Tab(icon: Icon(Icons.list), text: 'Items'),
            Tab(icon: Icon(Icons.grid_on), text: 'Matrix'),
            Tab(icon: Icon(Icons.upload_file), text: 'Bulk Ops'),
          ],
        ),
      ),
      body: TabBarView(
        controller: _tabController,
        children: [
          _InventoryOverviewTab(
              onNavigateToMatrix: () => _tabController.animateTo(2)),
          const _InventoryListTab(),
          const InventoryMatrixView(),
          const Center(
              child: Text(
                  "Bulk Operations (Coming Soon)")), // TODO: Move bulk widget here
        ],
      ),
    );
  }
}

class _InventoryOverviewTab extends StatefulWidget {
  final VoidCallback onNavigateToMatrix;

  const _InventoryOverviewTab({required this.onNavigateToMatrix});

  @override
  State<_InventoryOverviewTab> createState() => _InventoryOverviewTabState();
}

class _InventoryOverviewTabState extends State<_InventoryOverviewTab> {
  int _lowStockCount = 0;
  bool _isLoadingStats = true;

  @override
  void initState() {
    super.initState();
    _loadStats();
  }

  Future<void> _loadStats() async {
    final provider = context.read<InventoryProvider>();
    await provider.loadLowStock(threshold: 3);
    if (mounted) {
      setState(() {
        _lowStockCount = provider.items.length;
        _isLoadingStats = false;
      });
      // Reload full inventory for the list tab
      provider.loadInventory();
    }
  }

  @override
  Widget build(BuildContext context) {
    return Consumer<InventoryProvider>(
      builder: (context, provider, _) {
        final stats = provider.syncStats;
        final totalItems = stats['total'] ?? 0;
        
        return SingleChildScrollView(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Offline indicator
              if (provider.isOffline)
                Container(
                  width: double.infinity,
                  padding: const EdgeInsets.all(12),
                  margin: const EdgeInsets.only(bottom: 16),
                  decoration: BoxDecoration(
                    color: Colors.orange.shade100,
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: const Row(
                    children: [
                      Icon(Icons.cloud_off, color: Colors.orange),
                      SizedBox(width: 8),
                      Text('Offline Mode - Showing cached data'),
                    ],
                  ),
                ),
              
              Text('Inventory Health',
                  style: Theme.of(context).textTheme.headlineSmall),
              const SizedBox(height: 16),
              // Summary Cards Row
              Row(
                children: [
                  _SummaryCard(
                    title: 'Total Items',
                    value: totalItems.toString(),
                    icon: Icons.inventory_2,
                    color: Colors.blue,
                    isLoading: provider.isLoading,
                  ),
                  const SizedBox(width: 16),
                  _SummaryCard(
                    title: 'Low Stock',
                    value: _lowStockCount.toString(),
                    icon: Icons.warning_amber,
                    color: Colors.orange,
                    isLoading: _isLoadingStats,
                  ),
                  const SizedBox(width: 16),
                  _SummaryCard(
                    title: 'Unsynced',
                    value: (stats['unsynced'] ?? 0).toString(),
                    icon: Icons.sync_problem,
                    color: (stats['unsynced'] ?? 0) > 0 ? Colors.red : Colors.green,
                    isLoading: provider.isLoading,
                  ),
                ],
              ),
              const SizedBox(height: 32),

              Text('Actions', style: Theme.of(context).textTheme.titleLarge),
              const SizedBox(height: 16),
              Wrap(
                spacing: 16,
                runSpacing: 16,
                children: [
                  ActionCard(
                    title: 'View Matrix',
                    subtitle: 'Pivot view by condition',
                    icon: Icons.grid_on,
                    onTap: widget.onNavigateToMatrix,
                  ),
                  ActionCard(
                    title: 'Receive Stock',
                    subtitle: 'Add new items',
                    icon: Icons.add_box,
                    onTap: () {
                      showDialog(
                          context: context,
                          builder: (_) => const ReceiveStockDialog());
                    },
                  ),
                  ActionCard(
                    title: 'Price Check',
                    subtitle: 'Review pricing',
                    icon: Icons.price_check,
                    onTap: () {
                      Navigator.push(context,
                          MaterialPageRoute(builder: (_) => const PricingScreen()));
                    },
                  ),
                  ActionCard(
                    title: 'Sync Now',
                    subtitle: 'Push pending changes',
                    icon: Icons.cloud_upload,
                    onTap: () async {
                      final synced = await provider.syncPending();
                      if (context.mounted) {
                        ScaffoldMessenger.of(context).showSnackBar(
                          SnackBar(content: Text('Synced $synced items')),
                        );
                      }
                    },
                  ),
                ],
              ),
            ],
          ),
        );
      },
    );
  }
}

class _SummaryCard extends StatelessWidget {
  final String title;
  final String value;
  final IconData icon;
  final Color color;
  final bool isLoading;

  const _SummaryCard({
    required this.title,
    required this.value,
    required this.icon,
    required this.color,
    this.isLoading = false,
  });

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: Card(
        elevation: 2,
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Icon(icon, color: color, size: 32),
              const SizedBox(height: 8),
              Text(title, style: TextStyle(color: Colors.grey.shade600)),
              const SizedBox(height: 4),
              isLoading
                  ? const SizedBox(
                      width: 20,
                      height: 20,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : Text(
                      value,
                      style: const TextStyle(
                          fontSize: 24, fontWeight: FontWeight.bold),
                    ),
            ],
          ),
        ),
      ),
    );
  }
}

class ActionCard extends StatelessWidget {
  final String title;
  final String subtitle;
  final IconData icon;
  final VoidCallback onTap;

  const ActionCard({
    super.key,
    required this.title,
    required this.subtitle,
    required this.icon,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      clipBehavior: Clip.hardEdge,
      child: InkWell(
        onTap: onTap,
        child: Container(
          width: 200,
          padding: const EdgeInsets.all(16),
          child: Column(
            children: [
              Icon(icon, size: 48, color: Theme.of(context).primaryColor),
              const SizedBox(height: 12),
              Text(title,
                  style: const TextStyle(
                      fontWeight: FontWeight.bold, fontSize: 16)),
              Text(subtitle,
                  style: TextStyle(color: Colors.grey.shade600, fontSize: 12)),
            ],
          ),
        ),
      ),
    );
  }
}

class _InventoryListTab extends StatefulWidget {
  const _InventoryListTab();

  @override
  State<_InventoryListTab> createState() => _InventoryListTabState();
}

class _InventoryListTabState extends State<_InventoryListTab> {
  final _searchController = TextEditingController();
  String _searchQuery = '';

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(8.0),
          child: TextField(
            controller: _searchController,
            decoration: const InputDecoration(
              hintText: 'Search inventory...',
              prefixIcon: Icon(Icons.search),
              border: OutlineInputBorder(),
            ),
            onChanged: (val) {
              setState(() => _searchQuery = val.toLowerCase());
            },
          ),
        ),
        Expanded(
          child: Consumer<InventoryProvider>(
            builder: (context, provider, _) {
              if (provider.isLoading && provider.items.isEmpty) {
                return const Center(child: CircularProgressIndicator());
              }

              if (provider.error != null && provider.items.isEmpty) {
                return Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      const Icon(Icons.error_outline, size: 48, color: Colors.red),
                      const SizedBox(height: 16),
                      Text('Error: ${provider.error}'),
                      const SizedBox(height: 16),
                      ElevatedButton(
                        onPressed: () => provider.loadInventory(),
                        child: const Text('Retry'),
                      ),
                    ],
                  ),
                );
              }

              // Filter items based on search
              final items = provider.items.where((item) {
                if (_searchQuery.isEmpty) return true;
                return item.productUuid.toLowerCase().contains(_searchQuery) ||
                    item.locationTag.toLowerCase().contains(_searchQuery) ||
                    item.condition.name.toLowerCase().contains(_searchQuery);
              }).toList();

              if (items.isEmpty) {
                return const Center(
                  child: Text('No inventory items found'),
                );
              }

              return RefreshIndicator(
                onRefresh: () => provider.refresh(),
                child: ListView.separated(
                  itemCount: items.length,
                  separatorBuilder: (c, i) => const Divider(height: 1),
                  itemBuilder: (context, index) {
                    final item = items[index];
                    return ListTile(
                      title: Text('Product: ${item.productUuid.substring(0, 8)}'),
                      subtitle: Text('Condition: ${item.condition.name} | Location: ${item.locationTag}'),
                      trailing: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Text(
                            'Qty: ${item.quantityOnHand}',
                            style: TextStyle(
                              fontWeight: FontWeight.bold,
                              color: item.quantityOnHand <= 3 ? Colors.red : null,
                            ),
                          ),
                          const SizedBox(width: 8),
                          const Icon(Icons.chevron_right),
                        ],
                      ),
                      onTap: () {
                        showDialog(
                          context: context,
                          builder: (_) => InventoryDetailDialog(item: item),
                        );
                      },
                    );
                  },
                ),
              );
            },
          ),
        ),
      ],
    );
  }
}
