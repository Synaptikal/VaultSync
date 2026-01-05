import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../services/api_service.dart';
import 'widgets/inventory_matrix_view.dart';
import 'widgets/receive_stock_dialog.dart';
import 'widgets/inventory_detail_dialog.dart';
import '../pricing/pricing_screen.dart';
import '../../api/generated/models/inventory_item.dart';

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
          _InventoryListTab(),
          const InventoryMatrixView(),
          const Center(
              child: Text(
                  "Bulk Operations (Coming Soon)")), // TODO: Move bulk widget here
        ],
      ),
    );
  }
}

class _InventoryOverviewTab extends StatelessWidget {
  final VoidCallback onNavigateToMatrix;

  const _InventoryOverviewTab({required this.onNavigateToMatrix});

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('Inventory Health',
              style: Theme.of(context).textTheme.headlineSmall),
          const SizedBox(height: 16),
          // Summary Cards Row
          Row(
            children: [
              _SummaryCard(
                title: 'Total Items',
                value: 'Loading...',
                icon: Icons.inventory_2,
                color: Colors.blue,
                future: context
                    .read<ApiService>()
                    .getInventory(limit: 1)
                    .then((l) => "Coming Soon"), // Need stats endpoint
              ),
              const SizedBox(width: 16),
              _SummaryCard(
                title: 'Low Stock',
                value: '...',
                icon: Icons.warning_amber,
                color: Colors.orange,
                future: context
                    .read<ApiService>()
                    .getLowStockItems()
                    .then((l) => l.length.toString()),
              ),
              const SizedBox(width: 16),
              _SummaryCard(
                title: 'Total Value',
                value: '\$ --',
                icon: Icons.attach_money,
                color: Colors.green,
                future: context
                    .read<ApiService>()
                    .getInventoryValuation()
                    .then((m) => "\$${m['total_value']}"),
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
                onTap: onNavigateToMatrix,
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
            ],
          ),
        ],
      ),
    );
  }
}

class _SummaryCard extends StatelessWidget {
  final String title;
  final String value;
  final IconData icon;
  final Color color;
  final Future<String>? future;

  const _SummaryCard({
    required this.title,
    required this.value,
    required this.icon,
    required this.color,
    this.future,
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
              FutureBuilder<String>(
                future: future,
                initialData: value,
                builder: (context, snapshot) {
                  return Text(
                    snapshot.data ?? '--',
                    style: const TextStyle(
                        fontSize: 24, fontWeight: FontWeight.bold),
                  );
                },
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
  @override
  State<_InventoryListTab> createState() => _InventoryListTabState();
}

class _InventoryListTabState extends State<_InventoryListTab> {
  // Reuse existing list logic here roughly
  final _searchController = TextEditingController();

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
              // Implement filtering
            },
          ),
        ),
        Expanded(
          child: FutureBuilder<List<InventoryItem>>(
            future: context.read<ApiService>().getInventory(),
            builder: (context, snapshot) {
              if (snapshot.connectionState == ConnectionState.waiting)
                return const Center(child: CircularProgressIndicator());
              if (snapshot.hasError)
                return Center(child: Text('Error: ${snapshot.error}'));

              final items = snapshot.data ?? [];

              return ListView.separated(
                itemCount: items.length,
                separatorBuilder: (c, i) => const Divider(height: 1),
                itemBuilder: (context, index) {
                  final item = items[index];
                  // Need to resolve product name
                  // Ideally the API would return expanded objects or we define a ViewModel
                  return ListTile(
                    title: Text('Product: ${item.productUuid.substring(0, 8)}'),
                    subtitle: Text('Condition: ${item.condition.name}'),
                    trailing: Text('Qty: ${item.quantityOnHand}',
                        style: const TextStyle(fontWeight: FontWeight.bold)),
                    onTap: () {
                      showDialog(
                        context: context,
                        builder: (_) => InventoryDetailDialog(item: item),
                      );
                    },
                  );
                },
              );
            },
          ),
        ),
      ],
    );
  }
}
