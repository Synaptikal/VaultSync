import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:uuid/uuid.dart';
import '../../../services/api_service.dart';
import '../../../api/generated/models/product.dart';
import '../../../api/generated/models/condition.dart';
import '../../../api/generated/models/variant_type.dart';

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

  @override
  void dispose() {
    _qtyController.dispose();
    _priceController.dispose();
    _locationController.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    if (!_formKey.currentState!.validate()) return;
    setState(() => _isSaving = true);

    try {
      final qty = int.parse(_qtyController.text);
      final price = double.tryParse(_priceController.text);

      final data = {
        'inventory_uuid': const Uuid().v4(),
        'product_uuid': widget.product.productUuid,
        'condition': _condition.toJson(),
        'variant_type': _variant.toJson(),
        'quantity_on_hand': qty,
        'location_tag': _locationController.text,
        'specific_price': price,
        'serialized_details': null,
      };

      await context.read<ApiService>().addInventory(data);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(content: Text('Inventory Added successfully')));
        Navigator.pop(context, true);
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context)
            .showSnackBar(SnackBar(content: Text('Error: $e')));
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
                Row(
                  children: [
                    Expanded(
                      child: DropdownButtonFormField<Condition>(
                        initialValue: _condition,
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
                        initialValue: _variant,
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
