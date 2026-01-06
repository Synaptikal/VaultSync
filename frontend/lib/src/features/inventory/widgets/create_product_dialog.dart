import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:uuid/uuid.dart';
import '../../../api/generated/models/product.dart';
import '../../../providers/product_provider.dart';

/// Create Product Dialog (TASK-AUD-001g: Refactored to use ProductProvider)
///
/// Now uses ProductProvider.addProduct() instead of direct ApiService calls.
/// This enables offline product creation with local caching.

class CreateProductDialog extends StatefulWidget {
  const CreateProductDialog({super.key});

  @override
  State<CreateProductDialog> createState() => _CreateProductDialogState();
}

class _CreateProductDialogState extends State<CreateProductDialog> {
  final _formKey = GlobalKey<FormState>();
  final _nameController = TextEditingController();
  final _setController = TextEditingController();
  final _numberController = TextEditingController();
  final _yearController = TextEditingController();

  String _category = 'TCG';
  final _categories = [
    'TCG',
    'SportsCard',
    'Comic',
    'Bobblehead',
    'Apparel',
    'Figure',
    'Accessory',
    'Other'
  ];

  bool _isSaving = false;
  String? _error;

  @override
  void dispose() {
    _nameController.dispose();
    _setController.dispose();
    _numberController.dispose();
    _yearController.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    if (!_formKey.currentState!.validate()) return;
    setState(() {
      _isSaving = true;
      _error = null;
    });

    try {
      final data = {
        'product_uuid': const Uuid().v4(),
        'name': _nameController.text,
        'category': _category,
        'set_code': _setController.text.isNotEmpty ? _setController.text : null,
        'collector_number':
            _numberController.text.isNotEmpty ? _numberController.text : null,
        'release_year': int.tryParse(_yearController.text),
        'barcode': null,
        'metadata': {},
      };

      // Use ProductProvider for offline-first product creation
      final provider = context.read<ProductProvider>();
      await provider.addProduct(data);

      // Get the created product (last in list after reload)
      final products = provider.products;
      Product? createdProduct;
      if (products.isNotEmpty) {
        // Find the product we just created by UUID
        final uuid = data['product_uuid'] as String;
        createdProduct =
            products.where((p) => p.productUuid == uuid).firstOrNull;
      }

      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Created ${_nameController.text}'),
            backgroundColor: Colors.green,
          ),
        );
        Navigator.pop(context, createdProduct);
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
      title: const Text('New Product'),
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
                DropdownButtonFormField<String>(
                  value: _category,
                  decoration: const InputDecoration(labelText: 'Category'),
                  items: _categories
                      .map((c) => DropdownMenuItem(value: c, child: Text(c)))
                      .toList(),
                  onChanged: (v) => setState(() => _category = v!),
                ),
                const SizedBox(height: 16),
                TextFormField(
                  controller: _nameController,
                  decoration: const InputDecoration(
                    labelText: 'Product Name',
                    border: OutlineInputBorder(),
                    hintText: 'e.g. Charizard',
                  ),
                  validator: (v) => v == null || v.isEmpty ? 'Required' : null,
                ),
                const SizedBox(height: 16),
                Row(
                  children: [
                    Expanded(
                      child: TextFormField(
                        controller: _setController,
                        decoration: const InputDecoration(
                          labelText: 'Set / Series',
                          border: OutlineInputBorder(),
                          hintText: 'e.g. Base Set',
                        ),
                      ),
                    ),
                    const SizedBox(width: 16),
                    Expanded(
                      child: TextFormField(
                        controller: _numberController,
                        decoration: const InputDecoration(
                          labelText: 'Number',
                          border: OutlineInputBorder(),
                          hintText: 'e.g. 4/102',
                        ),
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 16),
                TextFormField(
                  controller: _yearController,
                  decoration: const InputDecoration(
                    labelText: 'Release Year',
                    border: OutlineInputBorder(),
                  ),
                  keyboardType: TextInputType.number,
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
              : const Text('Create'),
        ),
      ],
    );
  }
}
