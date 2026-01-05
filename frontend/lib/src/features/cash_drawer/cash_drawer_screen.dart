import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:intl/intl.dart';
import '../../services/api_service.dart';

/// Cash drawer management and end-of-day reports
class CashDrawerScreen extends StatefulWidget {
  const CashDrawerScreen({super.key});

  @override
  State<CashDrawerScreen> createState() => _CashDrawerScreenState();
}

class _CashDrawerScreenState extends State<CashDrawerScreen> {
  bool _isLoading = true;
  Map<String, dynamic> _drawerState = {};
  List<Map<String, dynamic>> _todaysTransactions = [];

  // Cash denomination controllers
  final Map<String, TextEditingController> _denominationControllers = {
    '100': TextEditingController(text: '0'),
    '50': TextEditingController(text: '0'),
    '20': TextEditingController(text: '0'),
    '10': TextEditingController(text: '0'),
    '5': TextEditingController(text: '0'),
    '1': TextEditingController(text: '0'),
    '0.25': TextEditingController(text: '0'),
    '0.10': TextEditingController(text: '0'),
    '0.05': TextEditingController(text: '0'),
    '0.01': TextEditingController(text: '0'),
  };

  @override
  void initState() {
    super.initState();
    _loadData();
  }

  @override
  void dispose() {
    for (var controller in _denominationControllers.values) {
      controller.dispose();
    }
    super.dispose();
  }

  Future<void> _loadData() async {
    setState(() => _isLoading = true);

    try {
      // Load today's transactions
      final api = context.read<ApiService>();
      final response = await api.getTransactions(limit: 200);
      final today = DateTime.now();
      _todaysTransactions = response.where((t) {
        final date = DateTime.tryParse(t['created_at'] ?? '');
        if (date == null) return false;
        return date.year == today.year &&
            date.month == today.month &&
            date.day == today.day;
      }).toList();

      // Calculate drawer state
      _drawerState = _calculateDrawerState();
    } catch (e) {
      // Handle error
    } finally {
      setState(() => _isLoading = false);
    }
  }

  Map<String, dynamic> _calculateDrawerState() {
    double cashSales = 0;
    double cardSales = 0;
    double cashPayouts = 0;
    int transactionCount = _todaysTransactions.length;

    for (var t in _todaysTransactions) {
      final type = t['transaction_type'] ?? 'Sale';
      final total = (t['total_amount'] ?? 0.0) as double;
      final paymentMethod = t['payment_method'] ?? 'cash';

      if (type == 'Sale') {
        if (paymentMethod == 'cash') {
          cashSales += total;
        } else {
          cardSales += total;
        }
      } else if (type == 'Buy') {
        cashPayouts += total;
      }
    }

    return {
      'starting_cash': 200.0, // Default starting float
      'cash_sales': cashSales,
      'card_sales': cardSales,
      'cash_payouts': cashPayouts,
      'transaction_count': transactionCount,
      'expected_cash': 200.0 + cashSales - cashPayouts,
    };
  }

  double get _countedCash {
    double total = 0;
    _denominationControllers.forEach((denom, controller) {
      final count = int.tryParse(controller.text) ?? 0;
      total += double.parse(denom) * count;
    });
    return total;
  }

  double get _variance => _countedCash - (_drawerState['expected_cash'] ?? 0.0);

  @override
  Widget build(BuildContext context) {
    if (_isLoading) {
      return const Scaffold(
        body: Center(child: CircularProgressIndicator()),
      );
    }

    return Scaffold(
      appBar: AppBar(
        title: const Text('Cash Drawer'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _loadData,
          ),
        ],
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Summary cards
            _buildSummarySection(),
            const SizedBox(height: 24),

            // Cash count section
            Text('Cash Count',
                style: Theme.of(context).textTheme.headlineSmall),
            const SizedBox(height: 16),
            _buildCashCountSection(),
            const SizedBox(height: 24),

            // Variance
            _buildVarianceSection(),
            const SizedBox(height: 24),

            // Actions
            _buildActionsSection(),
          ],
        ),
      ),
    );
  }

  Widget _buildSummarySection() {
    return GridView.count(
      crossAxisCount: 4,
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      crossAxisSpacing: 16,
      mainAxisSpacing: 16,
      childAspectRatio: 1.8,
      children: [
        _SummaryTile(
          title: 'Transactions',
          value: '${_drawerState['transaction_count']}',
          icon: Icons.receipt_long,
          color: Colors.blue,
        ),
        _SummaryTile(
          title: 'Cash Sales',
          value: '\$${(_drawerState['cash_sales'] ?? 0.0).toStringAsFixed(2)}',
          icon: Icons.payments,
          color: Colors.green,
        ),
        _SummaryTile(
          title: 'Card Sales',
          value: '\$${(_drawerState['card_sales'] ?? 0.0).toStringAsFixed(2)}',
          icon: Icons.credit_card,
          color: Colors.purple,
        ),
        _SummaryTile(
          title: 'Cash Payouts',
          value:
              '\$${(_drawerState['cash_payouts'] ?? 0.0).toStringAsFixed(2)}',
          icon: Icons.money_off,
          color: Colors.orange,
        ),
      ],
    );
  }

  Widget _buildCashCountSection() {
    final billDenominations = ['100', '50', '20', '10', '5', '1'];
    final coinDenominations = ['0.25', '0.10', '0.05', '0.01'];

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('Bills',
                style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16)),
            const SizedBox(height: 12),
            Wrap(
              spacing: 16,
              runSpacing: 8,
              children: billDenominations
                  .map((d) => _DenominationInput(
                        denomination: d,
                        controller: _denominationControllers[d]!,
                        onChanged: () => setState(() {}),
                      ))
                  .toList(),
            ),
            const Divider(height: 32),
            const Text('Coins',
                style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16)),
            const SizedBox(height: 12),
            Wrap(
              spacing: 16,
              runSpacing: 8,
              children: coinDenominations
                  .map((d) => _DenominationInput(
                        denomination: d,
                        controller: _denominationControllers[d]!,
                        onChanged: () => setState(() {}),
                        isCoin: true,
                      ))
                  .toList(),
            ),
            const Divider(height: 32),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                const Text('Total Counted:',
                    style:
                        TextStyle(fontWeight: FontWeight.bold, fontSize: 18)),
                Text(
                  '\$${_countedCash.toStringAsFixed(2)}',
                  style: const TextStyle(
                      fontWeight: FontWeight.bold,
                      fontSize: 24,
                      color: Colors.blue),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildVarianceSection() {
    final expected = _drawerState['expected_cash'] ?? 0.0;
    final isOver = _variance > 0;
    final isBalanced = _variance.abs() < 0.01;

    return Card(
      color: isBalanced
          ? Colors.green.shade50
          : (isOver ? Colors.blue.shade50 : Colors.red.shade50),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text('Expected Cash',
                        style: TextStyle(color: Colors.grey)),
                    Text('\$${expected.toStringAsFixed(2)}',
                        style: const TextStyle(
                            fontSize: 20, fontWeight: FontWeight.bold)),
                  ],
                ),
                Icon(
                  isBalanced
                      ? Icons.check_circle
                      : (isOver ? Icons.arrow_upward : Icons.arrow_downward),
                  size: 48,
                  color: isBalanced
                      ? Colors.green
                      : (isOver ? Colors.blue : Colors.red),
                ),
                Column(
                  crossAxisAlignment: CrossAxisAlignment.end,
                  children: [
                    Text(isBalanced ? 'Balanced' : (isOver ? 'Over' : 'Short'),
                        style: const TextStyle(color: Colors.grey)),
                    Text(
                      '${isOver ? '+' : ''}\$${_variance.toStringAsFixed(2)}',
                      style: TextStyle(
                        fontSize: 20,
                        fontWeight: FontWeight.bold,
                        color: isBalanced
                            ? Colors.green
                            : (isOver ? Colors.blue : Colors.red),
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildActionsSection() {
    return Wrap(
      spacing: 16,
      runSpacing: 16,
      children: [
        FilledButton.icon(
          icon: const Icon(Icons.open_in_new),
          label: const Text('Open Drawer'),
          onPressed: _openDrawer,
        ),
        OutlinedButton.icon(
          icon: const Icon(Icons.attach_money),
          label: const Text('Add Cash'),
          onPressed: _showAddCashDialog,
        ),
        OutlinedButton.icon(
          icon: const Icon(Icons.money_off),
          label: const Text('Remove Cash'),
          onPressed: _showRemoveCashDialog,
        ),
        FilledButton.icon(
          icon: const Icon(Icons.summarize),
          label: const Text('Generate EOD Report'),
          onPressed: _generateEODReport,
          style: FilledButton.styleFrom(backgroundColor: Colors.green),
        ),
      ],
    );
  }

  void _openDrawer() {
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('Cash drawer opened'),
        backgroundColor: Colors.green,
      ),
    );
  }

  void _showAddCashDialog() {
    _showCashAdjustmentDialog('Add Cash', 'Amount to add:', true);
  }

  void _showRemoveCashDialog() {
    _showCashAdjustmentDialog('Remove Cash', 'Amount to remove:', false);
  }

  void _showCashAdjustmentDialog(String title, String label, bool isAdd) {
    final controller = TextEditingController();
    final reasonController = TextEditingController();

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(title),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: controller,
              decoration: InputDecoration(
                labelText: label,
                prefixText: '\$ ',
                border: const OutlineInputBorder(),
              ),
              keyboardType:
                  const TextInputType.numberWithOptions(decimal: true),
            ),
            const SizedBox(height: 16),
            TextField(
              controller: reasonController,
              decoration: const InputDecoration(
                labelText: 'Reason',
                hintText: 'e.g., Bank deposit, Petty cash',
                border: OutlineInputBorder(),
              ),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () {
              Navigator.pop(context);
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(
                  content: Text(
                      '${isAdd ? 'Added' : 'Removed'} \$${controller.text}'),
                  backgroundColor: Colors.green,
                ),
              );
            },
            child: const Text('Confirm'),
          ),
        ],
      ),
    );
  }

  void _generateEODReport() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Row(
          children: [
            Icon(Icons.summarize, color: Colors.green),
            SizedBox(width: 8),
            Text('End of Day Report'),
          ],
        ),
        content: SizedBox(
          width: 400,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('Date: ${DateFormat.yMMMMd().format(DateTime.now())}',
                  style: const TextStyle(fontWeight: FontWeight.bold)),
              const Divider(),
              _ReportRow('Starting Cash',
                  '\$${(_drawerState['starting_cash'] ?? 0.0).toStringAsFixed(2)}'),
              _ReportRow('Cash Sales',
                  '+\$${(_drawerState['cash_sales'] ?? 0.0).toStringAsFixed(2)}'),
              _ReportRow('Cash Payouts',
                  '-\$${(_drawerState['cash_payouts'] ?? 0.0).toStringAsFixed(2)}'),
              const Divider(),
              _ReportRow('Expected Cash',
                  '\$${(_drawerState['expected_cash'] ?? 0.0).toStringAsFixed(2)}'),
              _ReportRow(
                  'Counted Cash', '\$${_countedCash.toStringAsFixed(2)}'),
              _ReportRow('Variance',
                  '${_variance >= 0 ? '+' : ''}\$${_variance.toStringAsFixed(2)}',
                  color: _variance.abs() < 0.01 ? Colors.green : Colors.red),
              const Divider(),
              _ReportRow('Card Sales',
                  '\$${(_drawerState['card_sales'] ?? 0.0).toStringAsFixed(2)}'),
              _ReportRow('Total Sales',
                  '\$${((_drawerState['cash_sales'] ?? 0.0) + (_drawerState['card_sales'] ?? 0.0)).toStringAsFixed(2)}',
                  isBold: true),
            ],
          ),
        ),
        actions: [
          TextButton.icon(
            icon: const Icon(Icons.print),
            label: const Text('Print'),
            onPressed: () {
              Navigator.pop(context);
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(
                  content: Text('EOD Report sent to printer'),
                  backgroundColor: Colors.green,
                ),
              );
            },
          ),
          TextButton.icon(
            icon: const Icon(Icons.save),
            label: const Text('Save & Close Day'),
            onPressed: () {
              Navigator.pop(context);
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(
                  content: Text('Day closed successfully'),
                  backgroundColor: Colors.green,
                ),
              );
            },
          ),
        ],
      ),
    );
  }
}

class _SummaryTile extends StatelessWidget {
  final String title;
  final String value;
  final IconData icon;
  final Color color;

  const _SummaryTile({
    required this.title,
    required this.value,
    required this.icon,
    required this.color,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: color.withAlpha(25), // 0.1 opacity = ~25/255
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: color.withAlpha(77)), // 0.3 opacity = ~77/255
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Row(
            children: [
              Icon(icon, size: 20, color: color),
              const SizedBox(width: 8),
              Text(title,
                  style: TextStyle(color: color, fontWeight: FontWeight.w500)),
            ],
          ),
          const SizedBox(height: 8),
          Text(value,
              style: TextStyle(
                  fontSize: 20, fontWeight: FontWeight.bold, color: color)),
        ],
      ),
    );
  }
}

class _DenominationInput extends StatelessWidget {
  final String denomination;
  final TextEditingController controller;
  final VoidCallback onChanged;
  final bool isCoin;

  const _DenominationInput({
    required this.denomination,
    required this.controller,
    required this.onChanged,
    this.isCoin = false,
  });

  @override
  Widget build(BuildContext context) {
    final denomValue = double.parse(denomination);
    final label = isCoin ? '${(denomValue * 100).toInt()}¢' : '\$$denomination';

    return SizedBox(
      width: 100,
      child: TextField(
        controller: controller,
        decoration: InputDecoration(
          labelText: label,
          border: const OutlineInputBorder(),
          isDense: true,
        ),
        keyboardType: TextInputType.number,
        textAlign: TextAlign.center,
        onChanged: (_) => onChanged(),
      ),
    );
  }
}

class _ReportRow extends StatelessWidget {
  final String label;
  final String value;
  final Color? color;
  final bool isBold;

  const _ReportRow(this.label, this.value, {this.color, this.isBold = false});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label,
              style: TextStyle(
                  fontWeight: isBold ? FontWeight.bold : FontWeight.normal)),
          Text(value,
              style: TextStyle(
                fontWeight: isBold ? FontWeight.bold : FontWeight.normal,
                color: color,
              )),
        ],
      ),
    );
  }
}
