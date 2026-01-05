import 'package:flutter/material.dart';
import '../../../providers/cart_provider.dart';

/// Dialog for applying discounts to the cart or individual items
class DiscountDialog extends StatefulWidget {
  final bool isCartDiscount;
  final double currentSubtotal;
  final CartItem? item;
  final int? itemIndex;

  const DiscountDialog({
    super.key,
    this.isCartDiscount = true,
    required this.currentSubtotal,
    this.item,
    this.itemIndex,
  });

  @override
  State<DiscountDialog> createState() => _DiscountDialogState();
}

class _DiscountDialogState extends State<DiscountDialog> {
  DiscountType _type = DiscountType.percentage;
  final _valueController = TextEditingController();
  final _reasonController = TextEditingController();

  // Preset discount buttons
  static const List<double> _percentPresets = [5, 10, 15, 20, 25, 50];
  static const List<double> _fixedPresets = [1, 2, 5, 10, 20, 50];

  double get _previewDiscount {
    final value = double.tryParse(_valueController.text) ?? 0;
    if (_type == DiscountType.percentage) {
      return widget.currentSubtotal * (value / 100);
    }
    return value.clamp(0, widget.currentSubtotal);
  }

  double get _afterDiscount => widget.currentSubtotal - _previewDiscount;

  @override
  void dispose() {
    _valueController.dispose();
    _reasonController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Row(
        children: [
          Icon(Icons.discount, color: Theme.of(context).colorScheme.primary),
          const SizedBox(width: 8),
          Text(widget.isCartDiscount ? 'Cart Discount' : 'Item Discount'),
        ],
      ),
      content: SizedBox(
        width: 400,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Target info
            if (!widget.isCartDiscount && widget.item != null)
              Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: Colors.grey.shade100,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Row(
                  children: [
                    const Icon(Icons.shopping_cart, size: 20),
                    const SizedBox(width: 8),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(widget.item!.product.name, style: const TextStyle(fontWeight: FontWeight.bold)),
                          Text('${widget.item!.quantity}x @ \$${widget.item!.price.toStringAsFixed(2)}'),
                        ],
                      ),
                    ),
                  ],
                ),
              )
            else
              Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: Colors.green.shade50,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    const Text('Cart Subtotal:'),
                    Text('\$${widget.currentSubtotal.toStringAsFixed(2)}', 
                      style: const TextStyle(fontWeight: FontWeight.bold, fontSize: 18)),
                  ],
                ),
              ),
            const SizedBox(height: 16),
            
            // Discount type selector
            const Text('Discount Type', style: TextStyle(fontWeight: FontWeight.bold)),
            const SizedBox(height: 8),
            SegmentedButton<DiscountType>(
              segments: const [
                ButtonSegment(value: DiscountType.percentage, label: Text('Percentage %'), icon: Icon(Icons.percent)),
                ButtonSegment(value: DiscountType.fixed, label: Text('Fixed \$'), icon: Icon(Icons.attach_money)),
              ],
              selected: {_type},
              onSelectionChanged: (selection) {
                setState(() {
                  _type = selection.first;
                  _valueController.clear();
                });
              },
            ),
            const SizedBox(height: 16),
            
            // Quick presets
            const Text('Quick Select', style: TextStyle(fontWeight: FontWeight.bold)),
            const SizedBox(height: 8),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: (_type == DiscountType.percentage ? _percentPresets : _fixedPresets)
                  .map((value) => ActionChip(
                        label: Text(_type == DiscountType.percentage ? '$value%' : '\$$value'),
                        onPressed: () {
                          setState(() => _valueController.text = value.toString());
                        },
                        backgroundColor: _valueController.text == value.toString()
                            ? Theme.of(context).colorScheme.primaryContainer
                            : null,
                      ))
                  .toList(),
            ),
            const SizedBox(height: 16),
            
            // Custom value input
            TextField(
              controller: _valueController,
              decoration: InputDecoration(
                labelText: _type == DiscountType.percentage ? 'Discount %' : 'Discount \$',
                border: const OutlineInputBorder(),
                prefixText: _type == DiscountType.fixed ? '\$ ' : null,
                suffixText: _type == DiscountType.percentage ? '%' : null,
              ),
              keyboardType: const TextInputType.numberWithOptions(decimal: true),
              onChanged: (_) => setState(() {}),
            ),
            const SizedBox(height: 16),
            
            // Reason (optional)
            TextField(
              controller: _reasonController,
              decoration: const InputDecoration(
                labelText: 'Reason (optional)',
                hintText: 'e.g., Loyalty discount, Damaged packaging',
                border: OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 16),
            
            // Preview
            AnimatedContainer(
              duration: const Duration(milliseconds: 200),
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: _previewDiscount > 0 ? Colors.green.shade50 : Colors.grey.shade100,
                borderRadius: BorderRadius.circular(8),
                border: Border.all(
                  color: _previewDiscount > 0 ? Colors.green.shade300 : Colors.grey.shade300,
                ),
              ),
              child: Column(
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      const Text('Subtotal:'),
                      Text('\$${widget.currentSubtotal.toStringAsFixed(2)}'),
                    ],
                  ),
                  if (_previewDiscount > 0) ...[
                    const SizedBox(height: 4),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Text('Discount:', style: TextStyle(color: Colors.green.shade700)),
                        Text('-\$${_previewDiscount.toStringAsFixed(2)}', 
                          style: TextStyle(color: Colors.green.shade700, fontWeight: FontWeight.bold)),
                      ],
                    ),
                    const Divider(),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        const Text('New Total:', style: TextStyle(fontWeight: FontWeight.bold)),
                        Text('\$${_afterDiscount.toStringAsFixed(2)}', 
                          style: const TextStyle(fontWeight: FontWeight.bold, fontSize: 20)),
                      ],
                    ),
                  ],
                ],
              ),
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
          onPressed: (_valueController.text.isEmpty || double.tryParse(_valueController.text) == null)
              ? null
              : () {
                  final value = double.parse(_valueController.text);
                  final discount = Discount(
                    name: _type == DiscountType.percentage ? '$value% Off' : '\$$value Off',
                    type: _type,
                    value: value,
                    reason: _reasonController.text.isNotEmpty ? _reasonController.text : null,
                  );
                  Navigator.pop(context, discount);
                },
          icon: const Icon(Icons.check),
          label: const Text('Apply Discount'),
        ),
      ],
    );
  }
}
