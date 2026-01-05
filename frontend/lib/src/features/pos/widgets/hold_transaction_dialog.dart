import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:intl/intl.dart';
import '../../../providers/cart_provider.dart';

/// Dialog to hold the current transaction for later
class HoldTransactionDialog extends StatefulWidget {
  const HoldTransactionDialog({super.key});

  @override
  State<HoldTransactionDialog> createState() => _HoldTransactionDialogState();
}

class _HoldTransactionDialogState extends State<HoldTransactionDialog> {
  final _nameController = TextEditingController();

  @override
  void initState() {
    super.initState();
    final cart = context.read<CartProvider>();
    _nameController.text = 'Hold ${cart.heldTransactions.length + 1}';
  }

  @override
  void dispose() {
    _nameController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Row(
        children: [
          Icon(Icons.pause_circle_outline, color: Colors.orange),
          SizedBox(width: 8),
          Text('Hold Transaction'),
        ],
      ),
      content: SizedBox(
        width: 400,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'This will save the current cart for later. You can recall it from the Held Transactions panel.',
            ),
            const SizedBox(height: 16),
            TextField(
              controller: _nameController,
              decoration: const InputDecoration(
                labelText: 'Hold Name',
                hintText: 'e.g., "Customer Bob - waiting for price check"',
                border: OutlineInputBorder(),
                prefixIcon: Icon(Icons.label),
              ),
              autofocus: true,
            ),
            const SizedBox(height: 16),
            
            // Cart summary
            Consumer<CartProvider>(
              builder: (context, cart, _) {
                return Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: Colors.grey.shade100,
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text('Cart Summary', style: TextStyle(fontWeight: FontWeight.bold)),
                      const SizedBox(height: 8),
                      Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          Text('Sale Items: ${cart.saleItems.length}'),
                          Text('\$${cart.saleTotal.toStringAsFixed(2)}'),
                        ],
                      ),
                      if (cart.tradeInItems.isNotEmpty) ...[
                        Row(
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: [
                            Text('Trade-In Items: ${cart.tradeInItems.length}'),
                            Text('-\$${cart.tradeInTotal.toStringAsFixed(2)}', style: const TextStyle(color: Colors.orange)),
                          ],
                        ),
                      ],
                      if (cart.customer != null) ...[
                        const Divider(),
                        Row(
                          children: [
                            const Icon(Icons.person, size: 16),
                            const SizedBox(width: 4),
                            Text(cart.customer!.name),
                          ],
                        ),
                      ],
                    ],
                  ),
                );
              },
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Cancel'),
        ),
        FilledButton.icon(
          onPressed: () {
            context.read<CartProvider>().holdCurrentTransaction(name: _nameController.text);
            Navigator.pop(context, true);
          },
          icon: const Icon(Icons.pause),
          label: const Text('Hold'),
          style: FilledButton.styleFrom(backgroundColor: Colors.orange),
        ),
      ],
    );
  }
}

/// Panel showing held transactions for recall
class HeldTransactionsPanel extends StatelessWidget {
  const HeldTransactionsPanel({super.key});

  @override
  Widget build(BuildContext context) {
    return Consumer<CartProvider>(
      builder: (context, cart, _) {
        if (cart.heldTransactions.isEmpty) {
          return const Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Icon(Icons.pause_circle_outline, size: 48, color: Colors.grey),
                SizedBox(height: 8),
                Text('No held transactions', style: TextStyle(color: Colors.grey)),
              ],
            ),
          );
        }

        return ListView.builder(
          padding: const EdgeInsets.all(8),
          itemCount: cart.heldTransactions.length,
          itemBuilder: (context, index) {
            final held = cart.heldTransactions[index];
            final itemCount = held.saleItems.length + held.tradeInItems.length;
            final total = held.saleItems.fold(0.0, (sum, i) => sum + i.total) -
                held.tradeInItems.fold(0.0, (sum, i) => sum + i.total);

            return Card(
              child: ListTile(
                leading: CircleAvatar(
                  backgroundColor: Colors.orange.shade100,
                  child: Text('${index + 1}', style: TextStyle(color: Colors.orange.shade800)),
                ),
                title: Text(held.name, style: const TextStyle(fontWeight: FontWeight.bold)),
                subtitle: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('$itemCount items • ${DateFormat.jm().format(held.heldAt)}'),
                    if (held.customer != null)
                      Text('Customer: ${held.customer!.name}', style: const TextStyle(fontSize: 12)),
                  ],
                ),
                isThreeLine: held.customer != null,
                trailing: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  crossAxisAlignment: CrossAxisAlignment.end,
                  children: [
                    Text(
                      '\$${total.abs().toStringAsFixed(2)}',
                      style: TextStyle(
                        fontWeight: FontWeight.bold,
                        color: total >= 0 ? Colors.green : Colors.orange,
                      ),
                    ),
                    Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        IconButton(
                          icon: const Icon(Icons.replay, size: 20),
                          tooltip: 'Recall',
                          onPressed: () => cart.recallTransaction(held.id),
                        ),
                        IconButton(
                          icon: const Icon(Icons.delete, size: 20, color: Colors.red),
                          tooltip: 'Delete',
                          onPressed: () => _confirmDelete(context, cart, held.id, held.name),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            );
          },
        );
      },
    );
  }

  void _confirmDelete(BuildContext context, CartProvider cart, String holdId, String holdName) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Delete Held Transaction?'),
        content: Text('Are you sure you want to delete "$holdName"? This cannot be undone.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () {
              cart.deleteHeldTransaction(holdId);
              Navigator.pop(context);
            },
            style: FilledButton.styleFrom(backgroundColor: Colors.red),
            child: const Text('Delete'),
          ),
        ],
      ),
    );
  }
}
