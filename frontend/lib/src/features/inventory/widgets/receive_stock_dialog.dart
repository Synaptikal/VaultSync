import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../../services/api_service.dart';
import '../../../api/generated/models/product.dart';
import 'add_serialized_item_dialog.dart';
import 'bulk_add_dialog.dart';
import 'create_product_dialog.dart';
import '../../../providers/product_provider.dart';

class ReceiveStockDialog extends StatefulWidget {
  const ReceiveStockDialog({super.key});

  @override
  State<ReceiveStockDialog> createState() => _ReceiveStockDialogState();
}

class _ReceiveStockDialogState extends State<ReceiveStockDialog> {
  final _searchController = TextEditingController();
  List<Product> _searchResults = [];
  bool _isLoading = false;
  Product? _selectedProduct;

  Future<void> _search() async {
    if (_searchController.text.isEmpty) return;
    setState(() => _isLoading = true);
    try {
      final results = await context
          .read<ApiService>()
          .searchProducts(query: _searchController.text);
      setState(() => _searchResults = results);
    } catch (e) {
      // Handle error
    } finally {
      setState(() => _isLoading = false);
    }
  }

  Future<void> _createNewProduct() async {
    final newProduct = await showDialog<Product>(
      context: context,
      builder: (_) => const CreateProductDialog(),
    );
    if (newProduct != null) {
      if (mounted) context.read<ProductProvider>().loadProducts();
      setState(() {
        _selectedProduct = newProduct;
        _searchResults = [newProduct];
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Dialog(
      child: Container(
        width: 600,
        height: 500,
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('Receive Stock',
                style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold)),
            const SizedBox(height: 16),
            if (_selectedProduct == null) ...[
              Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: _searchController,
                      decoration: const InputDecoration(
                        labelText: 'Search Product',
                        hintText: 'Name, Set Code...',
                        prefixIcon: Icon(Icons.search),
                        border: OutlineInputBorder(),
                      ),
                      onSubmitted: (_) => _search(),
                    ),
                  ),
                  const SizedBox(width: 8),
                  FilledButton(onPressed: _search, child: const Text('Search')),
                  const SizedBox(width: 8),
                  OutlinedButton.icon(
                    onPressed: _createNewProduct,
                    icon: const Icon(Icons.add),
                    label: const Text('New'),
                  ),
                ],
              ),
              const SizedBox(height: 16),
              Expanded(
                child: _isLoading
                    ? const Center(child: CircularProgressIndicator())
                    : ListView.builder(
                        itemCount: _searchResults.length,
                        itemBuilder: (context, index) {
                          final product = _searchResults[index];
                          return ListTile(
                            leading: const Icon(Icons.image),
                            title: Text(product.name),
                            subtitle: Text(
                                '${product.setCode} #${product.collectorNumber}'),
                            trailing: const Icon(Icons.chevron_right),
                            onTap: () =>
                                setState(() => _selectedProduct = product),
                          );
                        },
                      ),
              ),
            ] else ...[
              // Product Selected
              ListTile(
                contentPadding: EdgeInsets.zero,
                leading: IconButton(
                  icon: const Icon(Icons.arrow_back),
                  onPressed: () => setState(() => _selectedProduct = null),
                ),
                title: Text(_selectedProduct!.name,
                    style: const TextStyle(fontWeight: FontWeight.bold)),
                subtitle: const Text('Select how you want to add this item'),
              ),
              const Divider(),
              const SizedBox(height: 16),
              Row(
                children: [
                  _OptionCard(
                    title: 'Bulk / Standard',
                    icon: Icons.copy,
                    description:
                        'Add quantity of non-serialized items (e.g. Near Mint, Light Play)',
                    onTap: () async {
                      Navigator.pop(context);
                      await showDialog(
                        context: context,
                        builder: (_) =>
                            BulkAddDialog(product: _selectedProduct!),
                      );
                    },
                  ),
                  const SizedBox(width: 16),
                  _OptionCard(
                    title: 'Graded / Serialized',
                    icon: Icons.diamond,
                    description:
                        'Add a specific unique item with photos, certification #, and grading.',
                    color: Colors.amber.shade100,
                    iconColor: Colors.amber.shade800,
                    onTap: () async {
                      Navigator.pop(context);
                      await showDialog(
                        context: context,
                        builder: (_) =>
                            AddSerializedItemDialog(product: _selectedProduct!),
                      );
                    },
                  ),
                ],
              ),
            ],
          ],
        ),
      ),
    );
  }
}

class _OptionCard extends StatelessWidget {
  final String title;
  final IconData icon;
  final String description;
  final VoidCallback onTap;
  final Color? color;
  final Color? iconColor;

  const _OptionCard({
    required this.title,
    required this.icon,
    required this.description,
    required this.onTap,
    this.color,
    this.iconColor,
  });

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: GestureDetector(
        onTap: onTap,
        child: Container(
          height: 200,
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: color ??
                Theme.of(context)
                    .colorScheme
                    .surfaceContainerHighest
                    .withAlpha(77),
            borderRadius: BorderRadius.circular(12),
            border: Border.all(color: Colors.grey.shade300),
          ),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(icon,
                  size: 48,
                  color: iconColor ?? Theme.of(context).colorScheme.primary),
              const SizedBox(height: 12),
              Text(title,
                  style: const TextStyle(
                      fontSize: 18, fontWeight: FontWeight.bold)),
              const SizedBox(height: 8),
              Text(description,
                  textAlign: TextAlign.center,
                  style: TextStyle(color: Colors.grey.shade600)),
            ],
          ),
        ),
      ),
    );
  }
}
