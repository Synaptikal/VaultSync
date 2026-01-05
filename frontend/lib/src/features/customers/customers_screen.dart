import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:uuid/uuid.dart';
import 'package:intl/intl.dart';
import '../../providers/customer_provider.dart';
import '../../services/api_service.dart';
import '../../api/generated/models/customer.dart';

class CustomersScreen extends StatefulWidget {
  const CustomersScreen({super.key});

  @override
  State<CustomersScreen> createState() => _CustomersScreenState();
}

class _CustomersScreenState extends State<CustomersScreen> {
  final TextEditingController _searchController = TextEditingController();

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      context.read<CustomerProvider>().loadCustomers();
    });
  }

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  // ... _showAddCustomerDialog and _showCustomerDetails methods remain filtered ...

  // (Assuming _showAddCustomerDialog and _showCustomerDetails are preserved by matching context, but I need to be careful with range)
  // I will just replace the build method and the controller parts if possible, but the earlier tool call targeted lines 81-140.
  // I need to add the controller field at the top of the class.

  // Let's do this in two chunks or replace the whole start of class.

  // Chunk 1: Add controller field

  Future<void> _showAddCustomerDialog() async {
    final nameController = TextEditingController();
    final emailController = TextEditingController();
    final phoneController = TextEditingController();

    await showDialog(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('Add Customer'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
                controller: nameController,
                decoration: const InputDecoration(labelText: 'Name')),
            const SizedBox(height: 8),
            TextField(
                controller: emailController,
                decoration: const InputDecoration(labelText: 'Email')),
            const SizedBox(height: 8),
            TextField(
                controller: phoneController,
                decoration: const InputDecoration(labelText: 'Phone')),
          ],
        ),
        actions: [
          TextButton(
              onPressed: () => Navigator.pop(dialogContext),
              child: const Text('Cancel')),
          FilledButton(
              onPressed: () async {
                try {
                  // Use screen context for Provider to be safe, or dialogContext is also fine as it's below
                  await context
                      .read<CustomerProvider>()
                      .addCustomer(<String, dynamic>{
                    'customer_uuid': const Uuid().v4(),
                    'name': nameController.text,
                    'email': emailController.text.isNotEmpty
                        ? emailController.text
                        : null,
                    'phone': phoneController.text.isNotEmpty
                        ? phoneController.text
                        : null,
                    'store_credit': 0.0,
                    'created_at': DateTime.now().toIso8601String(),
                  });

                  // Use dialogContext to pop
                  if (dialogContext.mounted) {
                    Navigator.pop(dialogContext);
                  }

                  // Use screen context for SnackBar
                  if (mounted) {
                    ScaffoldMessenger.of(context).showSnackBar(const SnackBar(
                        content: Text('Customer Added Successfully')));
                  }
                } catch (e) {
                  if (mounted) {
                    ScaffoldMessenger.of(context)
                        .showSnackBar(SnackBar(content: Text('Error: $e')));
                  }
                }
              },
              child: const Text('Add')),
        ],
      ),
    );
  }

  Future<void> _showCustomerDetails(Customer customer) async {
    showDialog(
      context: context,
      builder: (context) => _CustomerDetailsDialog(customer: customer),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Customers')),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.all(16.0),
            child: TextField(
              controller: _searchController,
              decoration: const InputDecoration(
                hintText: 'Search customers...',
                prefixIcon: Icon(Icons.search),
                border: OutlineInputBorder(),
              ),
              onChanged: (value) {
                setState(() {});
              },
            ),
          ),
          Expanded(
            child: Consumer<CustomerProvider>(
              builder: (context, provider, child) {
                if (provider.isLoading) {
                  return const Center(child: CircularProgressIndicator());
                } else if (provider.error != null) {
                  return Center(child: Text('Error: ${provider.error}'));
                }

                // Filter customers
                final query = _searchController.text.toLowerCase();
                final customers = provider.customers.where((c) {
                  final name = c.name.toLowerCase();
                  final email = c.email?.toLowerCase() ?? '';
                  final phone = c.phone ?? '';
                  return name.contains(query) ||
                      email.contains(query) ||
                      phone.contains(query);
                }).toList();

                if (customers.isEmpty) {
                  return const Center(
                      child: Text('No customers found matching query'));
                }

                return ListView.builder(
                  padding: const EdgeInsets.all(16),
                  itemCount: customers.length,
                  itemBuilder: (context, index) {
                    final customer = customers[index];
                    return Card(
                      margin: const EdgeInsets.only(bottom: 8),
                      child: ListTile(
                        leading: CircleAvatar(
                            child: Text(customer.name.isNotEmpty
                                ? customer.name[0].toUpperCase()
                                : '?')),
                        title: Text(customer.name),
                        subtitle: Text(customer.email ??
                            customer.phone ??
                            'No contact info'),
                        trailing: Text(
                          '\$${customer.storeCredit.toStringAsFixed(2)}',
                          style: const TextStyle(
                              fontWeight: FontWeight.bold, color: Colors.green),
                        ),
                        onTap: () => _showCustomerDetails(customer),
                      ),
                    );
                  },
                );
              },
            ),
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _showAddCustomerDialog,
        child: const Icon(Icons.person_add),
      ),
    );
  }
}

class _CustomerDetailsDialog extends StatefulWidget {
  final Customer customer;

  const _CustomerDetailsDialog({required this.customer});

  @override
  State<_CustomerDetailsDialog> createState() => _CustomerDetailsDialogState();
}

class _CustomerDetailsDialogState extends State<_CustomerDetailsDialog> {
  late Future<List<Map<String, dynamic>>> _historyFuture;

  @override
  void initState() {
    super.initState();
    _historyFuture = context
        .read<ApiService>()
        .getCustomerHistory(widget.customer.customerUuid);
  }

  Future<void> _addCredit() async {
    final amountController = TextEditingController();
    await showDialog(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('Add Store Credit'),
        content: TextField(
          controller: amountController,
          decoration:
              const InputDecoration(labelText: 'Amount', prefixText: '\$'),
          keyboardType: const TextInputType.numberWithOptions(decimal: true),
        ),
        actions: [
          TextButton(
              onPressed: () => Navigator.pop(dialogContext),
              child: const Text('Cancel')),
          FilledButton(
            onPressed: () async {
              final amount = double.tryParse(amountController.text);
              if (amount != null) {
                try {
                  await context
                      .read<ApiService>()
                      .updateStoreCredit(widget.customer.customerUuid, amount);
                  if (dialogContext.mounted) {
                    Navigator.pop(dialogContext); // Close add credit dialog
                    if (mounted) {
                      context
                          .read<CustomerProvider>()
                          .loadCustomers(); // Refresh list
                      setState(() {
                        // Refresh history
                        _historyFuture = context
                            .read<ApiService>()
                            .getCustomerHistory(widget.customer.customerUuid);
                      });
                    }
                  }
                } catch (e) {
                  if (mounted) {
                    ScaffoldMessenger.of(context)
                        .showSnackBar(SnackBar(content: Text('Error: $e')));
                  }
                }
              }
            },
            child: const Text('Add Credit'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Dialog(
      child: Container(
        width: 500,
        height: 600,
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                CircleAvatar(
                    radius: 30,
                    child: Text(widget.customer.name[0].toUpperCase(),
                        style: const TextStyle(fontSize: 24))),
                const SizedBox(width: 16),
                Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(widget.customer.name,
                        style: Theme.of(context).textTheme.headlineSmall),
                    Text(widget.customer.email ??
                        widget.customer.phone ??
                        'No contact info'),
                  ],
                ),
                const Spacer(),
                Column(
                  crossAxisAlignment: CrossAxisAlignment.end,
                  children: [
                    const Text('Store Credit',
                        style: TextStyle(color: Colors.grey)),
                    Text('\$${widget.customer.storeCredit.toStringAsFixed(2)}',
                        style: const TextStyle(
                            fontSize: 20,
                            fontWeight: FontWeight.bold,
                            color: Colors.green)),
                  ],
                ),
              ],
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                FilledButton.icon(
                  onPressed: _addCredit,
                  icon: const Icon(Icons.add),
                  label: const Text('Add Credit'),
                ),
                const SizedBox(width: 8),
                OutlinedButton.icon(
                  onPressed: () {
                    // TODO: Implement Edit Profile
                  },
                  icon: const Icon(Icons.edit),
                  label: const Text('Edit Profile'),
                ),
              ],
            ),
            const Divider(height: 32),
            const Text('Transaction History',
                style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
            const SizedBox(height: 8),
            Expanded(
              child: FutureBuilder<List<Map<String, dynamic>>>(
                future: _historyFuture,
                builder: (context, snapshot) {
                  if (snapshot.connectionState == ConnectionState.waiting) {
                    return const Center(child: CircularProgressIndicator());
                  }
                  if (snapshot.hasError) {
                    return Center(child: Text('Error: ${snapshot.error}'));
                  }
                  final history = snapshot.data ?? [];
                  if (history.isEmpty) {
                    return const Center(child: Text('No transaction history.'));
                  }

                  return ListView.separated(
                    itemCount: history.length,
                    separatorBuilder: (context, index) => const Divider(),
                    itemBuilder: (context, index) {
                      final tx = history[index];
                      final date = DateTime.parse(tx['timestamp']);
                      final type = tx['transaction_type'];

                      Color typeColor = Colors.grey;
                      IconData typeIcon = Icons.receipt;

                      switch (type) {
                        case 'Sale':
                          typeColor = Colors.green;
                          typeIcon = Icons.shopping_cart;
                          break;
                        case 'Buy':
                          typeColor = Colors.orange;
                          typeIcon = Icons.store;
                          break;
                        case 'Trade':
                          typeColor = Colors.blue;
                          typeIcon = Icons.swap_horiz;
                          break;
                        case 'Return':
                          typeColor = Colors.red;
                          typeIcon = Icons.assignment_return;
                          break;
                      }

                      return ListTile(
                        leading: Icon(typeIcon, color: typeColor),
                        title: Text(type),
                        subtitle: Text(
                            DateFormat.yMMMd().add_jm().format(date.toLocal())),
                        // We could show total amount if we summed items, but for now just ID
                        trailing: Text(
                            tx['transaction_uuid'].toString().substring(0, 8),
                            style: const TextStyle(fontFamily: 'monospace')),
                      );
                    },
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }
}
