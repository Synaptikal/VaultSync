import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:uuid/uuid.dart';
import '../../../api/generated/models/product.dart';
import '../../../api/generated/models/condition.dart';
import '../../../api/generated/models/variant_type.dart';
import '../../../api/generated/models/inventory_item.dart';
import '../../../providers/inventory_provider.dart';

/// Bulk Add Dialog (TASK-AUD-001h: Refactored to use InventoryProvider)
///
/// Now uses InventoryProvider.addItem() instead of direct ApiService calls.
/// This enables offline inventory addition with local caching.

class BulkAddDialog extends StatefulWidget {
  final Product product;
  const BulkAddDialog({super.key, required this.product});

  @override
  State<BulkAddDialog> createState() => _BulkAddDialogState();
}

class _BulkAddDialogState extends State<BulkAddDialog> {
  final _formKey = GlobalKey<FormState>();
  final _qtyController = TextEditingController(text: '1');
  final _priceController = TextEditingController();
  final _locationController = TextEditingController();

  Condition _condition = Condition.nm;
  VariantType _variant = VariantType.normal;
  bool _isSaving = false;
  String? _error;

  @override
  void dispose() {
    _qtyController.dispose();
    _priceController.dispose();
    _locationController.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    if (!_formKey.currentState!.validate()) return;
    setState(() {
      _isSaving = true;
      _error = null;
    });

    try {
      final qty = int.parse(_qtyController.text);
      final price = double.tryParse(_priceController.text);

      // Create InventoryItem using the model
      final item = InventoryItem(
        inventoryUuid: const Uuid().v4(),
        productUuid: widget.product.productUuid,
        condition: _condition,
        quantityOnHand: qty,
        locationTag: _locationController.text,
        specificPrice: price,
        variantType: _variant,
        minStockLevel: 0,
        serializedDetails: null,
      );

      // Use InventoryProvider for offline-first inventory creation
      final provider = context.read<InventoryProvider>();
      await provider.addItem(item);

      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('Inventory added successfully'),
            backgroundColor: Colors.green,
          ),
        );
        Navigator.pop(context, true);
      }
    } catch (e) {
      if (mounted) {
        setState(() => _error = e.toString());
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Error: $e'), backgroundColor: Colors.red),
        );
      }
    } finally {
      if (mounted) setState(() => _isSaving = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('Bulk Add', style: Theme.of(context).textTheme.headlineSmall),
          Text(widget.product.name,
              style: Theme.of(context).textTheme.titleMedium),
        ],
      ),
      content: SingleChildScrollView(
        child: Form(
          key: _formKey,
          child: SizedBox(
            width: 400,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                // Error display
                if (_error != null)
                  Container(
                    width: double.infinity,
                    padding: const EdgeInsets.all(8),
                    margin: const EdgeInsets.only(bottom: 16),
                    decoration: BoxDecoration(
                      color: Colors.red.shade100,
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(_error!,
                        style: const TextStyle(color: Colors.red)),
                  ),
                Row(
                  children: [
                    Expanded(
                      child: DropdownButtonFormField<Condition>(
                        value: _condition,
                        decoration:
                            const InputDecoration(labelText: 'Condition'),
                        items: Condition.$valuesDefined
                            .map((c) =>
                                DropdownMenuItem(value: c, child: Text(c.name)))
                            .toList(),
                        onChanged: (v) => setState(() => _condition = v!),
                      ),
                    ),
                    const SizedBox(width: 16),
                    Expanded(
                      child: DropdownButtonFormField<VariantType>(
                        value: _variant,
                        decoration: const InputDecoration(labelText: 'Variant'),
                        items: VariantType.$valuesDefined
                            .map((v) =>
                                DropdownMenuItem(value: v, child: Text(v.name)))
                            .toList(),
                        onChanged: (v) => setState(() => _variant = v!),
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 16),
                TextFormField(
                  controller: _qtyController,
                  decoration: const InputDecoration(
                    labelText: 'Quantity',
                    border: OutlineInputBorder(),
                  ),
                  keyboardType: TextInputType.number,
                  validator: (v) {
                    if (v == null || v.isEmpty) return 'Required';
                    if (int.tryParse(v) == null) return 'Invalid Number';
                    if (int.parse(v) <= 0) return 'Must be > 0';
                    return null;
                  },
                ),
                const SizedBox(height: 16),
                TextFormField(
                  controller: _locationController,
                  decoration: const InputDecoration(
                    labelText: 'Location (Box/Bin)',
                    border: OutlineInputBorder(),
                    prefixIcon: Icon(Icons.location_on),
                  ),
                ),
                const SizedBox(height: 16),
                TextFormField(
                  controller: _priceController,
                  decoration: const InputDecoration(
                    labelText: 'Override Price (Optional)',
                    border: OutlineInputBorder(),
                    prefixText: '\$ ',
                    helperText: 'Leave empty to use Market Price',
                  ),
                  keyboardType:
                      const TextInputType.numberWithOptions(decimal: true),
                ),
              ],
            ),
          ),
        ),
      ),
      actions: [
        TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel')),
        FilledButton(
          onPressed: _isSaving ? null : _submit,
          child: _isSaving
              ? const SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(strokeWidth: 2))
              : const Text('Add Items'),
        ),
      ],
    );
  }
}
