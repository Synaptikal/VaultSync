import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:intl/intl.dart';
import '../../providers/transaction_provider.dart';
import 'widgets/receipt_dialog.dart';

/// Transactions Screen (TASK-AUD-001k: Refactored to use TransactionProvider)
///
/// Now uses TransactionProvider for all operations.
/// Enables better state management and cached transaction access.

class TransactionsScreen extends StatefulWidget {
  const TransactionsScreen({super.key});

  @override
  State<TransactionsScreen> createState() => _TransactionsScreenState();
}

class _TransactionsScreenState extends State<TransactionsScreen>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;
  DateTimeRange? _dateRange;
  String _typeFilter = 'all';

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 3, vsync: this);
    _dateRange = DateTimeRange(
      start: DateTime.now().subtract(const Duration(days: 7)),
      end: DateTime.now(),
    );

    WidgetsBinding.instance.addPostFrameCallback((_) {
      _loadTransactions();
    });
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  Future<void> _loadTransactions() async {
    final provider = context.read<TransactionProvider>();
    await provider.loadTransactions(
      transactionType: _typeFilter == 'all' ? null : _typeFilter,
      limit: 100,
    );
  }

  List<Map<String, dynamic>> _getFilteredTransactions(
      TransactionProvider provider) {
    if (_dateRange == null) return provider.transactions;

    return provider.transactions.where((t) {
      final date = DateTime.tryParse(t['created_at'] ?? '');
      if (date == null) return true;
      return date
              .isAfter(_dateRange!.start.subtract(const Duration(days: 1))) &&
          date.isBefore(_dateRange!.end.add(const Duration(days: 1)));
    }).toList();
  }

  List<Map<String, dynamic>> _getTodaysTransactions(
      TransactionProvider provider) {
    final today = DateTime.now();
    return provider.transactions.where((t) {
      final date = DateTime.tryParse(t['created_at'] ?? '');
      if (date == null) return false;
      return date.year == today.year &&
          date.month == today.month &&
          date.day == today.day;
    }).toList();
  }

  double _getTodaysSalesTotal(TransactionProvider provider) {
    return _getTodaysTransactions(provider)
        .where((t) => t['transaction_type'] == 'Sale')
        .fold(0.0, (sum, t) => sum + (t['total_amount'] ?? 0.0));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Transactions'),
        actions: [
          // Offline indicator
          Consumer<TransactionProvider>(
            builder: (context, provider, _) {
              if (provider.isOffline) {
                return const Padding(
                  padding: EdgeInsets.symmetric(horizontal: 8),
                  child: Chip(
                    avatar: Icon(Icons.cloud_off, size: 16),
                    label: Text('Offline'),
                    backgroundColor: Colors.orange,
                  ),
                );
              }
              return const SizedBox.shrink();
            },
          ),
          IconButton(
            icon: const Icon(Icons.calendar_today),
            tooltip: 'Date Range',
            onPressed: _selectDateRange,
          ),
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: 'Refresh',
            onPressed: _loadTransactions,
          ),
        ],
        bottom: TabBar(
          controller: _tabController,
          onTap: (index) {
            setState(() {
              _typeFilter = ['all', 'Sale', 'Buy'][index];
            });
            _loadTransactions();
          },
          tabs: const [
            Tab(text: 'All'),
            Tab(text: 'Sales'),
            Tab(text: 'Trade-Ins'),
          ],
        ),
      ),
      body: Consumer<TransactionProvider>(
        builder: (context, provider, _) {
          final filteredTransactions = _getFilteredTransactions(provider);
          final todaysTransactions = _getTodaysTransactions(provider);
          final todaysSalesTotal = _getTodaysSalesTotal(provider);

          return Column(
            children: [
              // Daily summary
              _buildDailySummary(todaysTransactions.length, todaysSalesTotal),

              // Transaction list
              Expanded(
                child: provider.isLoading && provider.transactions.isEmpty
                    ? const Center(child: CircularProgressIndicator())
                    : provider.error != null && provider.transactions.isEmpty
                        ? Center(
                            child: Column(
                              mainAxisAlignment: MainAxisAlignment.center,
                              children: [
                                const Icon(Icons.error_outline,
                                    size: 48, color: Colors.red),
                                const SizedBox(height: 16),
                                Text('Error: ${provider.error}'),
                                const SizedBox(height: 16),
                                ElevatedButton(
                                  onPressed: _loadTransactions,
                                  child: const Text('Retry'),
                                ),
                              ],
                            ),
                          )
                        : filteredTransactions.isEmpty
                            ? const Center(child: Text('No transactions found'))
                            : RefreshIndicator(
                                onRefresh: _loadTransactions,
                                child:
                                    _buildTransactionList(filteredTransactions),
                              ),
              ),
            ],
          );
        },
      ),
    );
  }

  Widget _buildDailySummary(int todaysCount, double todaysSales) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.primaryContainer.withAlpha(77),
      ),
      child: Row(
        children: [
          Expanded(
            child: _SummaryCard(
              icon: Icons.receipt_long,
              title: "Today's Transactions",
              value: '$todaysCount',
              color: Colors.blue,
            ),
          ),
          const SizedBox(width: 16),
          Expanded(
            child: _SummaryCard(
              icon: Icons.attach_money,
              title: "Today's Sales",
              value: '\$${todaysSales.toStringAsFixed(2)}',
              color: Colors.green,
            ),
          ),
          const SizedBox(width: 16),
          Expanded(
            child: _SummaryCard(
              icon: Icons.date_range,
              title: 'Date Range',
              value: _dateRange != null
                  ? '${DateFormat.MMMd().format(_dateRange!.start)} - ${DateFormat.MMMd().format(_dateRange!.end)}'
                  : 'All Time',
              color: Colors.purple,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildTransactionList(List<Map<String, dynamic>> transactions) {
    return ListView.separated(
      padding: const EdgeInsets.all(16),
      itemCount: transactions.length,
      separatorBuilder: (_, __) => const Divider(height: 1),
      itemBuilder: (context, index) {
        final transaction = transactions[index];
        return _TransactionTile(
          transaction: transaction,
          onTap: () => _showTransactionDetails(transaction),
        );
      },
    );
  }

  Future<void> _selectDateRange() async {
    final picked = await showDateRangePicker(
      context: context,
      firstDate: DateTime(2020),
      lastDate: DateTime.now(),
      initialDateRange: _dateRange,
    );

    if (picked != null) {
      setState(() => _dateRange = picked);
    }
  }

  void _showTransactionDetails(Map<String, dynamic> transaction) {
    showDialog(
      context: context,
      builder: (context) => _TransactionDetailDialog(transaction: transaction),
    );
  }
}

class _SummaryCard extends StatelessWidget {
  final IconData icon;
  final String title;
  final String value;
  final Color color;

  const _SummaryCard({
    required this.icon,
    required this.title,
    required this.value,
    required this.color,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: color.withAlpha(25),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withAlpha(77)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(icon, size: 16, color: color),
              const SizedBox(width: 4),
              Text(title, style: TextStyle(fontSize: 12, color: color)),
            ],
          ),
          const SizedBox(height: 4),
          Text(value,
              style: TextStyle(
                  fontSize: 18, fontWeight: FontWeight.bold, color: color)),
        ],
      ),
    );
  }
}

class _TransactionTile extends StatelessWidget {
  final Map<String, dynamic> transaction;
  final VoidCallback onTap;

  const _TransactionTile({required this.transaction, required this.onTap});

  @override
  Widget build(BuildContext context) {
    final type = transaction['transaction_type'] ?? 'Unknown';
    final isSale = type == 'Sale';
    final total = (transaction['total_amount'] ?? 0.0) as double;
    final date = DateTime.tryParse(transaction['created_at'] ?? '');
    final itemCount = (transaction['items'] as List?)?.length ?? 0;

    return ListTile(
      leading: CircleAvatar(
        backgroundColor:
            isSale ? Colors.green.shade100 : Colors.orange.shade100,
        child: Icon(
          isSale ? Icons.shopping_cart : Icons.swap_horiz,
          color: isSale ? Colors.green : Colors.orange,
        ),
      ),
      title: Text(
        'Transaction #${(transaction['transaction_uuid'] as String?)?.substring(0, 8) ?? 'N/A'}',
        style: const TextStyle(fontWeight: FontWeight.bold),
      ),
      subtitle: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('$type • $itemCount items'),
          if (date != null)
            Text(
              DateFormat.yMMMd().add_jm().format(date.toLocal()),
              style: Theme.of(context).textTheme.bodySmall,
            ),
        ],
      ),
      isThreeLine: true,
      trailing: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          Text(
            '\$${total.toStringAsFixed(2)}',
            style: TextStyle(
              fontWeight: FontWeight.bold,
              fontSize: 16,
              color: isSale ? Colors.green : Colors.orange,
            ),
          ),
          Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              IconButton(
                icon: const Icon(Icons.receipt_long, size: 20),
                tooltip: 'Print Receipt',
                onPressed: () => _printReceipt(context),
              ),
              const Icon(Icons.chevron_right),
            ],
          ),
        ],
      ),
      onTap: onTap,
    );
  }

  void _printReceipt(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => ReceiptDialog(transaction: transaction),
    );
  }
}

class _TransactionDetailDialog extends StatelessWidget {
  final Map<String, dynamic> transaction;

  const _TransactionDetailDialog({required this.transaction});

  @override
  Widget build(BuildContext context) {
    final type = transaction['transaction_type'] ?? 'Unknown';
    final isSale = type == 'Sale';
    final total = (transaction['total_amount'] ?? 0.0) as double;
    final date = DateTime.tryParse(transaction['created_at'] ?? '');
    final items = (transaction['items'] as List?) ?? [];

    return AlertDialog(
      title: Row(
        children: [
          Icon(isSale ? Icons.shopping_cart : Icons.swap_horiz,
              color: isSale ? Colors.green : Colors.orange),
          const SizedBox(width: 8),
          const Text('Transaction Details'),
        ],
      ),
      content: SizedBox(
        width: 500,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Header
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  children: [
                    _DetailRow(
                        'Transaction ID',
                        (transaction['transaction_uuid'] as String?)
                                ?.substring(0, 8) ??
                            'N/A'),
                    _DetailRow('Type', type),
                    if (date != null)
                      _DetailRow('Date',
                          DateFormat.yMMMd().add_jm().format(date.toLocal())),
                    _DetailRow('Status', transaction['status'] ?? 'Completed'),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),

            // Items
            const Text('Items',
                style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16)),
            const SizedBox(height: 8),
            Container(
              constraints: const BoxConstraints(maxHeight: 200),
              child: ListView.builder(
                shrinkWrap: true,
                itemCount: items.length,
                itemBuilder: (context, index) {
                  final item = items[index] as Map<String, dynamic>;
                  return ListTile(
                    dense: true,
                    title: Text(item['product_name'] ?? 'Unknown Product'),
                    subtitle: Text(
                        'Qty: ${item['quantity']} @ \$${(item['unit_price'] ?? 0.0).toStringAsFixed(2)}'),
                    trailing: Text(
                      '\$${((item['quantity'] ?? 1) * (item['unit_price'] ?? 0.0)).toStringAsFixed(2)}',
                      style: const TextStyle(fontWeight: FontWeight.bold),
                    ),
                  );
                },
              ),
            ),
            const Divider(),

            // Total
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                const Text('Total',
                    style:
                        TextStyle(fontWeight: FontWeight.bold, fontSize: 18)),
                Text(
                  '\$${total.toStringAsFixed(2)}',
                  style: TextStyle(
                    fontWeight: FontWeight.bold,
                    fontSize: 24,
                    color: isSale ? Colors.green : Colors.orange,
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
      actions: [
        TextButton.icon(
          icon: const Icon(Icons.receipt_long),
          label: const Text('Print Receipt'),
          onPressed: () {
            Navigator.pop(context);
            showDialog(
              context: context,
              builder: (context) => ReceiptDialog(transaction: transaction),
            );
          },
        ),
        TextButton.icon(
          icon: const Icon(Icons.undo),
          label: const Text('Process Return'),
          onPressed: () {
            Navigator.pop(context);
            _showReturnDialog(context);
          },
        ),
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Close'),
        ),
      ],
    );
  }

  void _showReturnDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Row(
          children: [
            Icon(Icons.undo, color: Colors.orange),
            SizedBox(width: 8),
            Text('Process Return'),
          ],
        ),
        content: const Text(
            'Return functionality will be available in the next update.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('OK'),
          ),
        ],
      ),
    );
  }
}

class _DetailRow extends StatelessWidget {
  final String label;
  final String value;

  const _DetailRow(this.label, this.value);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: const TextStyle(color: Colors.grey)),
          Text(value, style: const TextStyle(fontWeight: FontWeight.w500)),
        ],
      ),
    );
  }
}
