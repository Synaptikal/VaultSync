import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../../providers/cart_provider.dart';

/// Dialog for selecting tax zone
class TaxZoneDialog extends StatelessWidget {
  const TaxZoneDialog({super.key});

  @override
  Widget build(BuildContext context) {
    return Consumer<CartProvider>(
      builder: (context, cart, _) {
        return AlertDialog(
          title: const Row(
            children: [
              Icon(Icons.location_on, color: Colors.blue),
              SizedBox(width: 8),
              Text('Tax Zone'),
            ],
          ),
          content: SizedBox(
            width: 400,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'Select the tax zone for this transaction. Different zones have different tax rates.',
                ),
                const SizedBox(height: 16),

                // Tax zone options
                ...TaxZone.allZones.map((zone) => _TaxZoneTile(
                      zone: zone,
                      isSelected: cart.taxZone.id == zone.id,
                      subtotal: cart.saleTotal,
                      onTap: () {
                        cart.setTaxZone(zone);
                        Navigator.pop(context);
                      },
                    )),

                const SizedBox(height: 16),

                // Custom tax zone
                OutlinedButton.icon(
                  onPressed: () => _showCustomTaxDialog(context, cart),
                  icon: const Icon(Icons.add),
                  label: const Text('Custom Tax Rate'),
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('Cancel'),
            ),
          ],
        );
      },
    );
  }

  void _showCustomTaxDialog(BuildContext context, CartProvider cart) {
    final controller = TextEditingController();
    final nameController = TextEditingController();

    showDialog(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('Custom Tax Rate'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: nameController,
              decoration: const InputDecoration(
                labelText: 'Zone Name',
                hintText: 'e.g., Special Event Tax',
                border: OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 16),
            TextField(
              controller: controller,
              decoration: const InputDecoration(
                labelText: 'Tax Rate',
                hintText: 'e.g., 7.5',
                suffixText: '%',
                border: OutlineInputBorder(),
              ),
              keyboardType:
                  const TextInputType.numberWithOptions(decimal: true),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(dialogContext),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () {
              final rate = double.tryParse(controller.text);
              if (rate != null) {
                final customZone = TaxZone(
                  id: 'custom_${DateTime.now().millisecondsSinceEpoch}',
                  name: nameController.text.isNotEmpty
                      ? nameController.text
                      : 'Custom ($rate%)',
                  rate: rate / 100,
                );
                cart.setTaxZone(customZone);
                Navigator.pop(dialogContext);
                Navigator.pop(context);
              }
            },
            child: const Text('Apply'),
          ),
        ],
      ),
    );
  }
}

class _TaxZoneTile extends StatelessWidget {
  final TaxZone zone;
  final bool isSelected;
  final double subtotal;
  final VoidCallback onTap;

  const _TaxZoneTile({
    required this.zone,
    required this.isSelected,
    required this.subtotal,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final taxAmount = subtotal * zone.rate;

    return Card(
      color: isSelected ? Theme.of(context).colorScheme.primaryContainer : null,
      child: ListTile(
        leading: Icon(
          isSelected
              ? Icons.radio_button_checked
              : Icons.radio_button_unchecked,
          color: isSelected ? Theme.of(context).colorScheme.primary : null,
        ),
        title: Text(
          zone.name,
          style: TextStyle(
            fontWeight: isSelected ? FontWeight.bold : FontWeight.normal,
          ),
        ),
        subtitle: Text('${(zone.rate * 100).toStringAsFixed(2)}% tax rate'),
        trailing: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          crossAxisAlignment: CrossAxisAlignment.end,
          children: [
            Text(
              '+\$${taxAmount.toStringAsFixed(2)}',
              style: TextStyle(
                fontWeight: FontWeight.bold,
                color: zone.rate == 0 ? Colors.green : Colors.grey.shade700,
              ),
            ),
            Text(
              'Total: \$${(subtotal + taxAmount).toStringAsFixed(2)}',
              style: const TextStyle(fontSize: 12),
            ),
          ],
        ),
        onTap: onTap,
      ),
    );
  }
}

/// Compact tax zone selector for the checkout bar
class TaxZoneChip extends StatelessWidget {
  const TaxZoneChip({super.key});

  @override
  Widget build(BuildContext context) {
    return Consumer<CartProvider>(
      builder: (context, cart, _) {
        return ActionChip(
          avatar: const Icon(Icons.location_on, size: 18),
          label: Text(cart.taxZone.name),
          onPressed: () {
            showDialog(
              context: context,
              builder: (context) => const TaxZoneDialog(),
            );
          },
        );
      },
    );
  }
}
