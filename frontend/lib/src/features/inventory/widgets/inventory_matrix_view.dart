import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../../api/generated/models/product.dart';
import '../../../api/generated/models/category.dart';
import '../../../providers/product_provider.dart';
import '../../../providers/inventory_provider.dart';

/// Inventory Matrix View (Refactored to use Providers)
///
/// Now uses InventoryProvider for inventory data instead of ApiService.
/// Enables offline inventory matrix viewing.

class InventoryMatrixView extends StatefulWidget {
  const InventoryMatrixView({super.key});

  @override
  State<InventoryMatrixView> createState() => _InventoryMatrixViewState();
}

class _InventoryMatrixViewState extends State<InventoryMatrixView> {
  bool _isLoading = true;
  String? _error;
  Map<String, Map<String, int>> _matrix = {};

  @override
  void initState() {
    super.initState();
    _loadData();
  }

  Future<void> _loadData() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });

    try {
      // Use InventoryProvider instead of ApiService
      final inventoryProvider = context.read<InventoryProvider>();
      await inventoryProvider.loadInventory();

      final items = inventoryProvider.items;

      // Pivot data: ProductUUID -> Condition -> Quantity
      final Map<String, Map<String, int>> matrix = {};

      for (var item in items) {
        if (!matrix.containsKey(item.productUuid)) {
          matrix[item.productUuid] = {};
        }
        final productEntry = matrix[item.productUuid]!;
        final conditionKey = item.condition
            .toString()
            .split('.')
            .last
            .toUpperCase(); // Enum to string key

        productEntry[conditionKey] =
            (productEntry[conditionKey] ?? 0) + item.quantityOnHand;
      }

      if (mounted) {
        setState(() {
          _matrix = matrix;
          _isLoading = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _error = e.toString();
          _isLoading = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final inventoryProvider = context.watch<InventoryProvider>();

    // Show loading or error state
    if (_isLoading || inventoryProvider.isLoading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_error != null && inventoryProvider.items.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.error_outline, size: 48, color: Colors.red),
            const SizedBox(height: 16),
            Text('Error: $_error'),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: _loadData,
              child: const Text('Retry'),
            ),
          ],
        ),
      );
    }

    // We need products to display names
    final productProvider = context.watch<ProductProvider>();
    // Ensure products are loaded (or load if empty?)
    if (productProvider.products.isEmpty && !productProvider.isLoading) {
      productProvider.loadProducts();
    }

    final conditions = ['NM', 'LP', 'MP', 'HP', 'DMG'];

    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              const Icon(Icons.grid_on),
              const SizedBox(width: 8),
              const Text('Inventory Matrix',
                  style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
              const Spacer(),
              // Offline indicator
              if (inventoryProvider.isOffline)
                const Chip(
                  avatar: Icon(Icons.cloud_off, size: 16),
                  label: Text('Offline'),
                  backgroundColor: Colors.orange,
                ),
              const SizedBox(width: 8),
              IconButton(icon: const Icon(Icons.refresh), onPressed: _loadData),
            ],
          ),
        ),
        Expanded(
          child: _matrix.isEmpty
              ? const Center(child: Text('No inventory data available'))
              : SingleChildScrollView(
                  scrollDirection: Axis.vertical,
                  child: SingleChildScrollView(
                    scrollDirection: Axis.horizontal,
                    child: DataTable(
                      columns: [
                        const DataColumn(label: Text('Product')),
                        const DataColumn(label: Text('Set/Code')),
                        ...conditions.map((c) => DataColumn(
                            label: Text(c,
                                style: const TextStyle(
                                    fontWeight: FontWeight.bold)))),
                        const DataColumn(
                            label: Text('Total',
                                style: TextStyle(fontWeight: FontWeight.bold))),
                      ],
                      rows: _matrix.entries.map((entry) {
                        final productUuid = entry.key;
                        final quantities = entry.value;

                        // Find product details
                        final product = productProvider.products.firstWhere(
                          (p) => p.productUuid == productUuid,
                          orElse: () => Product(
                            productUuid: productUuid,
                            name: 'Unknown Product',
                            category: Category.other,
                            metadata: {},
                          ),
                        );

                        int rowTotal = 0;

                        final cells = conditions.map((cond) {
                          final qty = quantities[cond] ?? 0;
                          rowTotal += qty;

                          return DataCell(
                            Container(
                              alignment: Alignment.center,
                              child: Text(
                                qty > 0 ? qty.toString() : '-',
                                style: TextStyle(
                                  color: qty > 0
                                      ? Colors.black
                                      : Colors.grey.shade300,
                                  fontWeight: qty > 0
                                      ? FontWeight.bold
                                      : FontWeight.normal,
                                ),
                              ),
                            ),
                          );
                        }).toList();

                        return DataRow(cells: [
                          DataCell(SizedBox(
                              width: 200,
                              child: Text(product.name,
                                  overflow: TextOverflow.ellipsis))),
                          DataCell(Text(product.setCode ?? '-')),
                          ...cells,
                          DataCell(Text(rowTotal.toString(),
                              style: const TextStyle(
                                  fontWeight: FontWeight.bold))),
                        ]);
                      }).toList(),
                    ),
                  ),
                ),
        ),
      ],
    );
  }
}
