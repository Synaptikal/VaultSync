import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../services/api_service.dart';

class PricingScreen extends StatefulWidget {
  const PricingScreen({super.key});

  @override
  State<PricingScreen> createState() => _PricingScreenState();
}

class _PricingScreenState extends State<PricingScreen> {
  bool _isLoading = true;
  Map<String, dynamic>? _dashboardData;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadData();
  }

  Future<void> _loadData() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });

    try {
      final data = await context.read<ApiService>().getPricingDashboard();
      setState(() {
        _dashboardData = data;
        _isLoading = false;
      });
    } catch (e) {
      setState(() {
        _error = e.toString();
        _isLoading = false;
      });
    }
  }

  Future<void> _syncPrices() async {
    try {
      ScaffoldMessenger.of(context)
          .showSnackBar(const SnackBar(content: Text('Syncing prices...')));
      await context.read<ApiService>().triggerPriceSync();
      _loadData(); // Reload dashboard
    } catch (e) {
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text('Sync failed: $e')));
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Pricing & Market'),
        actions: [
          IconButton(
            icon: const Icon(Icons.sync),
            tooltip: 'Sync Prices Now',
            onPressed: _syncPrices,
          ),
        ],
      ),
      body: _buildBody(),
    );
  }

  Widget _buildBody() {
    if (_isLoading) return const Center(child: CircularProgressIndicator());
    if (_error != null) return Center(child: Text('Error: $_error'));
    if (_dashboardData == null) return const Center(child: Text('No data'));

    final trends = _dashboardData!['market_trends'] as Map<String, dynamic>;
    final alerts = _dashboardData!['volatility_alerts'] as List<dynamic>? ?? [];

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Status Card
          Card(
            color: _getStatusColor(_dashboardData!['status']),
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Row(
                children: [
                  const Icon(Icons.cloud_sync, color: Colors.white, size: 32),
                  const SizedBox(width: 16),
                  Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('Provider Status: ${_dashboardData!['status']}',
                          style: const TextStyle(
                              color: Colors.white,
                              fontWeight: FontWeight.bold)),
                      Text('Last Sync: ${_dashboardData!['last_sync']}',
                          style: const TextStyle(color: Colors.white70)),
                    ],
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 24),

          // Market Trends
          const Text('Market Trends (24h)',
              style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
          const SizedBox(height: 16),
          Row(
            children: [
              _TrendCard(
                  label: 'Gliding Up',
                  count: trends['up'] ?? 0,
                  color: Colors.green,
                  icon: Icons.trending_up),
              const SizedBox(width: 16),
              _TrendCard(
                  label: 'Stable',
                  count: trends['stable'] ?? 0,
                  color: Colors.blue,
                  icon: Icons.trending_flat),
              const SizedBox(width: 16),
              _TrendCard(
                  label: 'Dropping',
                  count: trends['down'] ?? 0,
                  color: Colors.red,
                  icon: Icons.trending_down),
            ],
          ),

          const SizedBox(height: 32),

          // Volatility Alerts
          const Text('Volatility Alerts',
              style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
          const SizedBox(height: 8),
          if (alerts.isEmpty)
            const Card(
              child: ListTile(
                leading: Icon(Icons.check_circle, color: Colors.green),
                title: Text('No volatility alerts'),
                subtitle: Text('Market seems stable for your inventory.'),
              ),
            )
          else
            Column(
              children: alerts
                  .map((alert) => Card(
                        color: Colors.orange.shade50,
                        child: ListTile(
                          leading:
                              const Icon(Icons.warning, color: Colors.orange),
                          title:
                              Text(alert['product_name'] ?? 'Unknown Product'),
                          subtitle: Text(
                              'Price changed by ${alert['percent_change']}%'),
                          trailing: const Icon(Icons.chevron_right),
                          onTap: () {
                            // Navigate to details
                          },
                        ),
                      ))
                  .toList(),
            ),
        ],
      ),
    );
  }

  Color _getStatusColor(String? status) {
    if (status == 'Healthy' || status == 'Active') return Colors.green.shade700;
    if (status == 'Needs Refresh') return Colors.orange.shade700;
    return Colors.red.shade700;
  }
}

class _TrendCard extends StatelessWidget {
  final String label;
  final int count;
  final Color color;
  final IconData icon;

  const _TrendCard(
      {required this.label,
      required this.count,
      required this.color,
      required this.icon});

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: Container(
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          color: color.withValues(alpha: 0.1),
          borderRadius: BorderRadius.circular(12),
          border: Border.all(color: color.withValues(alpha: 0.3)),
        ),
        child: Column(
          children: [
            Icon(icon, color: color, size: 28),
            const SizedBox(height: 8),
            Text(count.toString(),
                style: TextStyle(
                    fontSize: 24, fontWeight: FontWeight.bold, color: color)),
            Text(label, style: TextStyle(color: color, fontSize: 12)),
          ],
        ),
      ),
    );
  }
}
