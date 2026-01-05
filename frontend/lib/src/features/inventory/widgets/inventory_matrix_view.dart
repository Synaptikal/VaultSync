import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../../api/generated/models/product.dart';
import '../../../api/generated/models/category.dart';
import '../../../providers/product_provider.dart';
import '../../../services/api_service.dart';

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
      final items = await context
          .read<ApiService>()
          .getInventory(limit: 1000); // Fetch mostly all

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

      setState(() {
        _matrix = matrix;
        _isLoading = false;
      });
    } catch (e) {
      setState(() {
        _error = e.toString();
        _isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading) return const Center(child: CircularProgressIndicator());
    if (_error != null) return Center(child: Text('Error: $_error'));

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
              IconButton(icon: const Icon(Icons.refresh), onPressed: _loadData),
            ],
          ),
        ),
        Expanded(
          child: SingleChildScrollView(
            scrollDirection: Axis.vertical,
            child: SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: DataTable(
                columns: [
                  const DataColumn(label: Text('Product')),
                  const DataColumn(label: Text('Set/Code')),
                  ...conditions.map((c) => DataColumn(
                      label: Text(c,
                          style:
                              const TextStyle(fontWeight: FontWeight.bold)))),
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
                      name: 'Loading...', // Or Unknown
                      category: Category.other,
                      metadata: {},
                    ),
                  );

                  int rowTotal = 0;

                  final cells = conditions.map((cond) {
                    // Normalize map keys? Our keys are likely UPPERCASE from earlier logic
                    // But let's check exact match.
                    // Actually condition.toString() typically gives "Condition.nm" or similar?
                    // My logic above used split('.').last.toUpperCase().
                    // Assuming 'NM', 'LP' etc match.

                    // Need to map our standard conditions to whatever we put in map
                    // Condition enum values: nm, lp, mp, hp, dmg.
                    // My previous step: split('.').last.toUpperCase() -> NM, LP...

                    final qty = quantities[cond] ?? 0;
                    rowTotal += qty;

                    return DataCell(
                      Container(
                        alignment: Alignment.center,
                        child: Text(
                          qty > 0 ? qty.toString() : '-',
                          style: TextStyle(
                            color:
                                qty > 0 ? Colors.black : Colors.grey.shade300,
                            fontWeight:
                                qty > 0 ? FontWeight.bold : FontWeight.normal,
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
                        style: const TextStyle(fontWeight: FontWeight.bold))),
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
