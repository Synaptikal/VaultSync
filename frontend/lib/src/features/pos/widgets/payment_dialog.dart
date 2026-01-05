import 'package:flutter/material.dart';
import '../../../api/generated/models/customer.dart';

/// Payment method options
enum PaymentMethod {
  cash,
  card,
  storeCredit,
  split,
}

/// Payment dialog for finalizing transactions
class PaymentDialog extends StatefulWidget {
  final double saleTotal;
  final double tradeInTotal;
  final Customer? customer;

  const PaymentDialog({
    super.key,
    required this.saleTotal,
    required this.tradeInTotal,
    this.customer,
  });

  @override
  State<PaymentDialog> createState() => _PaymentDialogState();
}

class _PaymentDialogState extends State<PaymentDialog> {
  PaymentMethod _selectedMethod = PaymentMethod.cash;
  final _cashReceivedController = TextEditingController();
  double _cashReceived = 0.0;

  double get netTotal => widget.saleTotal - widget.tradeInTotal;
  double get amountDue => netTotal > 0 ? netTotal : 0;
  double get storeOwes => netTotal < 0 ? netTotal.abs() : 0;
  double get change => _cashReceived - amountDue;
  double get availableCredit => widget.customer?.storeCredit ?? 0.0;

  @override
  void initState() {
    super.initState();
    // If customer owes money to store, default to cash
    // If store owes customer, they might want store credit
    if (storeOwes > 0) {
      _selectedMethod = PaymentMethod.storeCredit;
    }
  }

  @override
  Widget build(BuildContext context) {
    final isCustomerPaying = amountDue > 0;

    return Dialog(
      child: Container(
        width: 500,
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Header
            Row(
              children: [
                Icon(
                  Icons.payment,
                  size: 32,
                  color: Theme.of(context).colorScheme.primary,
                ),
                const SizedBox(width: 12),
                const Text(
                  'Complete Transaction',
                  style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold),
                ),
              ],
            ),
            const SizedBox(height: 24),

            // Transaction summary
            _buildSummaryCard(),
            const SizedBox(height: 24),

            // Payment method selection
            if (isCustomerPaying) ...[
              const Text(
                'Payment Method',
                style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16),
              ),
              const SizedBox(height: 12),
              _buildPaymentMethodGrid(),
              const SizedBox(height: 16),

              // Cash received input (if cash selected)
              if (_selectedMethod == PaymentMethod.cash) _buildCashInput(),
            ] else ...[
              const Text(
                'Payout Method',
                style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16),
              ),
              const SizedBox(height: 12),
              _buildPayoutMethodGrid(),
            ],

            const SizedBox(height: 24),

            // Actions
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                TextButton(
                  onPressed: () => Navigator.pop(context),
                  child: const Text('Cancel'),
                ),
                const SizedBox(width: 16),
                ElevatedButton.icon(
                  icon: const Icon(Icons.check_circle),
                  label: Text(
                    isCustomerPaying
                        ? 'Confirm Payment'
                        : 'Confirm Payout',
                  ),
                  onPressed: _canComplete() ? _completeTransaction : null,
                  style: ElevatedButton.styleFrom(
                    backgroundColor: Colors.green,
                    foregroundColor: Colors.white,
                    padding: const EdgeInsets.symmetric(
                      horizontal: 24,
                      vertical: 12,
                    ),
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSummaryCard() {
    return Card(
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            _buildSummaryRow('Sale Total', widget.saleTotal, Colors.green),
            const SizedBox(height: 8),
            _buildSummaryRow('Trade-In Credit', -widget.tradeInTotal, Colors.orange),
            const Divider(height: 24),
            _buildSummaryRow(
              netTotal >= 0 ? 'Customer Pays' : 'Store Owes',
              netTotal.abs(),
              netTotal >= 0 ? Colors.green.shade700 : Colors.orange.shade700,
              isLarge: true,
            ),
            if (widget.customer != null && availableCredit > 0) ...[
              const SizedBox(height: 8),
              _buildSummaryRow(
                'Available Store Credit',
                availableCredit,
                Colors.blue,
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildSummaryRow(String label, double amount, Color color, {bool isLarge = false}) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        Text(
          label,
          style: TextStyle(
            fontSize: isLarge ? 18 : 14,
            fontWeight: isLarge ? FontWeight.bold : FontWeight.normal,
          ),
        ),
        Text(
          '\$${amount.toStringAsFixed(2)}',
          style: TextStyle(
            fontSize: isLarge ? 28 : 16,
            fontWeight: FontWeight.bold,
            color: color,
          ),
        ),
      ],
    );
  }

  Widget _buildPaymentMethodGrid() {
    return Wrap(
      spacing: 12,
      runSpacing: 12,
      children: [
        _PaymentMethodCard(
          icon: Icons.money,
          label: 'Cash',
          isSelected: _selectedMethod == PaymentMethod.cash,
          onTap: () => setState(() => _selectedMethod = PaymentMethod.cash),
        ),
        _PaymentMethodCard(
          icon: Icons.credit_card,
          label: 'Card',
          isSelected: _selectedMethod == PaymentMethod.card,
          onTap: () => setState(() => _selectedMethod = PaymentMethod.card),
        ),
        if (widget.customer != null && availableCredit >= amountDue)
          _PaymentMethodCard(
            icon: Icons.account_balance_wallet,
            label: 'Store Credit',
            subtitle: '\$${availableCredit.toStringAsFixed(2)} available',
            isSelected: _selectedMethod == PaymentMethod.storeCredit,
            onTap: () => setState(() => _selectedMethod = PaymentMethod.storeCredit),
          ),
        _PaymentMethodCard(
          icon: Icons.call_split,
          label: 'Split',
          isSelected: _selectedMethod == PaymentMethod.split,
          onTap: () => setState(() => _selectedMethod = PaymentMethod.split),
        ),
      ],
    );
  }

  Widget _buildPayoutMethodGrid() {
    return Wrap(
      spacing: 12,
      runSpacing: 12,
      children: [
        _PaymentMethodCard(
          icon: Icons.money,
          label: 'Cash',
          subtitle: 'Pay out cash',
          isSelected: _selectedMethod == PaymentMethod.cash,
          onTap: () => setState(() => _selectedMethod = PaymentMethod.cash),
        ),
        _PaymentMethodCard(
          icon: Icons.account_balance_wallet,
          label: 'Store Credit',
          subtitle: 'Add to account',
          isSelected: _selectedMethod == PaymentMethod.storeCredit,
          onTap: () => setState(() => _selectedMethod = PaymentMethod.storeCredit),
          isRecommended: widget.customer != null,
        ),
      ],
    );
  }

  Widget _buildCashInput() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Cash Received',
              style: TextStyle(fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: _cashReceivedController,
              decoration: InputDecoration(
                prefixText: '\$ ',
                border: const OutlineInputBorder(),
                suffixIcon: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    // Quick amount buttons
                    _QuickAmountButton(
                      amount: amountDue,
                      label: 'Exact',
                      onPressed: () => _setCashReceived(amountDue),
                    ),
                    _QuickAmountButton(
                      amount: (amountDue / 10).ceil() * 10.0,
                      label: null,
                      onPressed: () => _setCashReceived((amountDue / 10).ceil() * 10.0),
                    ),
                    _QuickAmountButton(
                      amount: (amountDue / 20).ceil() * 20.0,
                      label: null,
                      onPressed: () => _setCashReceived((amountDue / 20).ceil() * 20.0),
                    ),
                  ],
                ),
              ),
              keyboardType: TextInputType.number,
              onChanged: (value) {
                setState(() {
                  _cashReceived = double.tryParse(value) ?? 0.0;
                });
              },
            ),
            if (_cashReceived >= amountDue && _cashReceived > 0) ...[
              const SizedBox(height: 12),
              Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: Colors.green.shade50,
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(color: Colors.green.shade200),
                ),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    const Text(
                      'Change Due:',
                      style: TextStyle(fontWeight: FontWeight.bold),
                    ),
                    Text(
                      '\$${change.toStringAsFixed(2)}',
                      style: TextStyle(
                        fontSize: 24,
                        fontWeight: FontWeight.bold,
                        color: Colors.green.shade700,
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }

  void _setCashReceived(double amount) {
    _cashReceivedController.text = amount.toStringAsFixed(2);
    setState(() => _cashReceived = amount);
  }

  bool _canComplete() {
    if (amountDue <= 0) return true; // Store owes customer
    
    switch (_selectedMethod) {
      case PaymentMethod.cash:
        return _cashReceived >= amountDue;
      case PaymentMethod.card:
        return true; // Card processing would validate
      case PaymentMethod.storeCredit:
        return availableCredit >= amountDue;
      case PaymentMethod.split:
        return true; // Would need split validation
    }
  }

  void _completeTransaction() {
    Navigator.pop(context, {
      'confirmed': true,
      'paymentMethod': _selectedMethod.name,
      'cashReceived': _cashReceived,
      'change': change,
    });
  }
}

class _PaymentMethodCard extends StatelessWidget {
  final IconData icon;
  final String label;
  final String? subtitle;
  final bool isSelected;
  final bool isRecommended;
  final VoidCallback onTap;

  const _PaymentMethodCard({
    required this.icon,
    required this.label,
    this.subtitle,
    required this.isSelected,
    this.isRecommended = false,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return Material(
      color: isSelected
          ? Theme.of(context).colorScheme.primaryContainer
          : Theme.of(context).colorScheme.surface,
      borderRadius: BorderRadius.circular(12),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(12),
        child: Container(
          width: 110,
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: isSelected
                  ? Theme.of(context).colorScheme.primary
                  : Colors.grey.shade300,
              width: isSelected ? 2 : 1,
            ),
          ),
          child: Column(
            children: [
              Stack(
                children: [
                  Icon(
                    icon,
                    size: 32,
                    color: isSelected
                        ? Theme.of(context).colorScheme.primary
                        : Colors.grey,
                  ),
                  if (isRecommended)
                    Positioned(
                      right: -4,
                      top: -4,
                      child: Container(
                        padding: const EdgeInsets.all(2),
                        decoration: const BoxDecoration(
                          color: Colors.green,
                          shape: BoxShape.circle,
                        ),
                        child: const Icon(Icons.star, size: 10, color: Colors.white),
                      ),
                    ),
                ],
              ),
              const SizedBox(height: 8),
              Text(
                label,
                style: TextStyle(
                  fontWeight: isSelected ? FontWeight.bold : FontWeight.normal,
                ),
              ),
              if (subtitle != null)
                Text(
                  subtitle!,
                  style: const TextStyle(fontSize: 10, color: Colors.grey),
                  textAlign: TextAlign.center,
                ),
            ],
          ),
        ),
      ),
    );
  }
}

class _QuickAmountButton extends StatelessWidget {
  final double amount;
  final String? label;
  final VoidCallback onPressed;

  const _QuickAmountButton({
    required this.amount,
    this.label,
    required this.onPressed,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(right: 4),
      child: TextButton(
        onPressed: onPressed,
        style: TextButton.styleFrom(
          minimumSize: const Size(50, 36),
          padding: const EdgeInsets.symmetric(horizontal: 8),
        ),
        child: Text(label ?? '\$${amount.toStringAsFixed(0)}'),
      ),
    );
  }
}
