import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:uuid/uuid.dart';
import 'package:intl/intl.dart';
import '../../providers/wants_provider.dart';
import '../../providers/customer_provider.dart';
import '../../services/api_service.dart';
import '../../api/generated/models/wants_list.dart';
import '../../api/generated/models/wants_item.dart';
import '../../api/generated/models/condition.dart';
import '../../api/generated/models/customer.dart';
import '../../api/generated/models/product.dart';

class WantsScreen extends StatefulWidget {
  const WantsScreen({super.key});

  @override
  State<WantsScreen> createState() => _WantsScreenState();
}

class _WantsScreenState extends State<WantsScreen> {
  Customer? _selectedCustomer;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      context.read<CustomerProvider>().loadCustomers();
    });
  }

  void _onCustomerSelected(Customer? customer) {
    setState(() => _selectedCustomer = customer);
    if (customer != null) {
      context.read<WantsProvider>().loadWantsLists(customer.customerUuid);
    } else {
      context.read<WantsProvider>().clear();
    }
  }

  Future<void> _showCreateWantsListDialog() async {
    if (_selectedCustomer == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Please select a customer first')),
      );
      return;
    }

    final List<_PendingWantsItem> pendingItems = [];

    await showDialog(
      context: context,
      builder: (dialogContext) => StatefulBuilder(
        builder: (sbContext, setDialogState) => AlertDialog(
          title: Text('Create Wants List for ${_selectedCustomer!.name}'),
          content: SizedBox(
            width: 500,
            height: 400,
            child: Column(
              children: [
                // Add item section
                Row(
                  children: [
                    Expanded(
                      child: ElevatedButton.icon(
                        icon: const Icon(Icons.add),
                        label: const Text('Add Item'),
                        onPressed: () async {
                          final item = await _showAddItemDialog(dialogContext);
                          if (item != null) {
                            setDialogState(() => pendingItems.add(item));
                          }
                        },
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 16),
                const Divider(),
                // Items list
                Expanded(
                  child: pendingItems.isEmpty
                      ? const Center(
                          child: Text(
                              'No items added yet.\nTap "Add Item" to add wanted products.',
                              textAlign: TextAlign.center))
                      : ListView.builder(
                          itemCount: pendingItems.length,
                          itemBuilder: (context, index) {
                            final item = pendingItems[index];
                            return ListTile(
                              title: Text(item.productName),
                              subtitle: Text(
                                  'Min: ${item.minCondition.name}${item.maxPrice != null ? ' • Max: \$${item.maxPrice!.toStringAsFixed(2)}' : ''}'),
                              trailing: IconButton(
                                icon:
                                    const Icon(Icons.delete, color: Colors.red),
                                onPressed: () => setDialogState(
                                    () => pendingItems.removeAt(index)),
                              ),
                            );
                          },
                        ),
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(dialogContext),
              child: const Text('Cancel'),
            ),
            FilledButton(
              onPressed: pendingItems.isEmpty
                  ? null
                  : () async {
                      try {
                        final wantsList = WantsList(
                          wantsListUuid: const Uuid().v4(),
                          customerUuid: _selectedCustomer!.customerUuid,
                          createdAt: DateTime.now(),
                          items: pendingItems
                              .map((p) => WantsItem(
                                    itemUuid: const Uuid().v4(),
                                    productUuid: p.productUuid,
                                    minCondition: p.minCondition,
                                    maxPrice: p.maxPrice,
                                    createdAt: DateTime.now(),
                                  ))
                              .toList(),
                        );

                        await context
                            .read<WantsProvider>()
                            .createWantsList(wantsList);
                        if (dialogContext.mounted) {
                          Navigator.pop(dialogContext);
                          if (mounted) {
                            ScaffoldMessenger.of(context).showSnackBar(
                              const SnackBar(
                                  content:
                                      Text('Wants list created successfully!')),
                            );
                          }
                        }
                      } catch (e) {
                        if (mounted) {
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(content: Text('Error: $e')),
                          );
                        }
                      }
                    },
              child: const Text('Create List'),
            ),
          ],
        ),
      ),
    );
  }

  Future<_PendingWantsItem?> _showAddItemDialog(
      BuildContext parentContext) async {
    final searchController = TextEditingController();
    List<Product> searchResults = [];
    Product? selectedProduct;
    Condition selectedCondition = Condition.nm;
    final maxPriceController = TextEditingController();

    return await showDialog<_PendingWantsItem>(
      context: parentContext,
      builder: (context) => StatefulBuilder(
        builder: (context, setState) => AlertDialog(
          title: const Text('Add Wanted Item'),
          content: SizedBox(
            width: 400,
            height: 350,
            child: Column(
              children: [
                // Search field
                TextField(
                  controller: searchController,
                  decoration: InputDecoration(
                    labelText: 'Search Products',
                    prefixIcon: const Icon(Icons.search),
                    suffixIcon: IconButton(
                      icon: const Icon(Icons.search),
                      onPressed: () async {
                        if (searchController.text.isNotEmpty) {
                          try {
                            final results = await context
                                .read<ApiService>()
                                .searchProducts(query: searchController.text);
                            setState(() => searchResults = results);
                          } catch (e) {
                            // Handle error
                          }
                        }
                      },
                    ),
                  ),
                  onSubmitted: (value) async {
                    if (value.isNotEmpty) {
                      try {
                        final results = await context
                            .read<ApiService>()
                            .searchProducts(query: value);
                        setState(() => searchResults = results);
                      } catch (e) {
                        // Handle error
                      }
                    }
                  },
                ),
                const SizedBox(height: 8),
                // Search results
                Expanded(
                  child: searchResults.isEmpty
                      ? const Center(child: Text('Search for a product'))
                      : ListView.builder(
                          itemCount: searchResults.length,
                          itemBuilder: (context, index) {
                            final product = searchResults[index];
                            final isSelected = selectedProduct?.productUuid ==
                                product.productUuid;
                            return ListTile(
                              title: Text(product.name),
                              subtitle: Text(
                                  product.setCode ?? product.category.name),
                              selected: isSelected,
                              selectedTileColor: Theme.of(context)
                                  .colorScheme
                                  .primaryContainer,
                              onTap: () =>
                                  setState(() => selectedProduct = product),
                            );
                          },
                        ),
                ),
                const Divider(),
                // Condition selector
                if (selectedProduct != null) ...[
                  DropdownButtonFormField<Condition>(
                    initialValue: selectedCondition,
                    decoration:
                        const InputDecoration(labelText: 'Minimum Condition'),
                    items: Condition.values
                        .map((c) =>
                            DropdownMenuItem(value: c, child: Text(c.name)))
                        .toList(),
                    onChanged: (v) => setState(() => selectedCondition = v!),
                  ),
                  const SizedBox(height: 8),
                  TextField(
                    controller: maxPriceController,
                    decoration: const InputDecoration(
                      labelText: 'Max Price (optional)',
                      prefixText: '\$',
                    ),
                    keyboardType:
                        const TextInputType.numberWithOptions(decimal: true),
                  ),
                ],
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('Cancel'),
            ),
            FilledButton(
              onPressed: selectedProduct == null
                  ? null
                  : () {
                      Navigator.pop(
                        context,
                        _PendingWantsItem(
                          productUuid: selectedProduct!.productUuid,
                          productName: selectedProduct!.name,
                          minCondition: selectedCondition,
                          maxPrice: double.tryParse(maxPriceController.text),
                        ),
                      );
                    },
              child: const Text('Add'),
            ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Wants Lists')),
      body: Column(
        children: [
          // Customer selector
          Padding(
            padding: const EdgeInsets.all(16.0),
            child: Consumer<CustomerProvider>(
              builder: (context, customerProvider, child) {
                if (customerProvider.isLoading) {
                  return const LinearProgressIndicator();
                }
                return DropdownButtonFormField<Customer>(
                  initialValue: _selectedCustomer,
                  decoration: const InputDecoration(
                    labelText: 'Select Customer',
                    border: OutlineInputBorder(),
                    prefixIcon: Icon(Icons.person),
                  ),
                  items: customerProvider.customers
                      .map((c) => DropdownMenuItem(
                          value: c,
                          child: Text(
                              '${c.name} (${c.email ?? c.phone ?? 'No contact'})')))
                      .toList(),
                  onChanged: _onCustomerSelected,
                );
              },
            ),
          ),
          const Divider(height: 1),
          // Wants lists
          Expanded(
            child: Consumer<WantsProvider>(
              builder: (context, wantsProvider, child) {
                if (_selectedCustomer == null) {
                  return const Center(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Icon(Icons.list_alt, size: 64, color: Colors.grey),
                        SizedBox(height: 16),
                        Text('Select a customer to view their wants lists'),
                      ],
                    ),
                  );
                }

                if (wantsProvider.isLoading) {
                  return const Center(child: CircularProgressIndicator());
                }

                if (wantsProvider.error != null) {
                  return Center(child: Text('Error: ${wantsProvider.error}'));
                }

                if (wantsProvider.wantsLists.isEmpty) {
                  return Center(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        const Icon(Icons.checklist,
                            size: 64, color: Colors.grey),
                        const SizedBox(height: 16),
                        Text(
                            '${_selectedCustomer!.name} has no wants lists yet'),
                        const SizedBox(height: 8),
                        FilledButton.icon(
                          onPressed: _showCreateWantsListDialog,
                          icon: const Icon(Icons.add),
                          label: const Text('Create First List'),
                        ),
                      ],
                    ),
                  );
                }

                return ListView.builder(
                  padding: const EdgeInsets.all(16),
                  itemCount: wantsProvider.wantsLists.length,
                  itemBuilder: (context, index) {
                    final wantsList = wantsProvider.wantsLists[index];
                    return Card(
                      margin: const EdgeInsets.only(bottom: 12),
                      child: ExpansionTile(
                        leading: CircleAvatar(
                          backgroundColor:
                              Theme.of(context).colorScheme.primaryContainer,
                          child: Text('${wantsList.items.length}'),
                        ),
                        title: Text('Wants List #${index + 1}'),
                        subtitle: Text(
                            'Created: ${DateFormat.yMMMd().format(wantsList.createdAt.toLocal())}'),
                        children: wantsList.items.map((item) {
                          return ListTile(
                            contentPadding:
                                const EdgeInsets.symmetric(horizontal: 24),
                            leading: const Icon(Icons.check_box_outline_blank),
                            title: FutureBuilder<Product>(
                              future: context
                                  .read<ApiService>()
                                  .getProductById(item.productUuid),
                              builder: (context, snapshot) {
                                return Text(
                                    snapshot.data?.name ?? 'Loading...');
                              },
                            ),
                            subtitle: Text(
                              'Min: ${item.minCondition.name}${item.maxPrice != null ? ' • Max: \$${item.maxPrice!.toStringAsFixed(2)}' : ''}',
                            ),
                          );
                        }).toList(),
                      ),
                    );
                  },
                );
              },
            ),
          ),
        ],
      ),
      floatingActionButton: _selectedCustomer != null
          ? FloatingActionButton(
              onPressed: _showCreateWantsListDialog,
              child: const Icon(Icons.add),
            )
          : null,
    );
  }
}

class _PendingWantsItem {
  final String productUuid;
  final String productName;
  final Condition minCondition;
  final double? maxPrice;

  _PendingWantsItem({
    required this.productUuid,
    required this.productName,
    required this.minCondition,
    this.maxPrice,
  });
}
