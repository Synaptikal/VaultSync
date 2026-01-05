import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';
import 'dart:async';
import '../../api/generated/models/customer.dart';
import '../../api/generated/models/product.dart';
import '../../api/generated/models/category.dart';
import '../../providers/product_provider.dart';
import '../../providers/customer_provider.dart';
import '../../providers/cart_provider.dart';
import '../../services/api_service.dart';
import 'widgets/condition_grading_dialog.dart';
import 'widgets/payment_dialog.dart';
import 'widgets/quick_actions_bar.dart';
import 'widgets/discount_dialog.dart';
import 'widgets/hold_transaction_dialog.dart';
import 'widgets/tax_zone_dialog.dart';

/// Enhanced POS Screen with split-screen layout for sales and trade-ins
class POSScreen extends StatefulWidget {
  const POSScreen({super.key});

  @override
  State<POSScreen> createState() => _POSScreenState();
}

class _POSScreenState extends State<POSScreen>
    with SingleTickerProviderStateMixin {
  final _searchController = TextEditingController();
  final _barcodeController = TextEditingController();
  final _barcodeFocusNode = FocusNode();
  Timer? _debounce;
  Customer? _selectedCustomer;
  late TabController _tabController;

  // Barcode scanner buffer
  String _barcodeBuffer = '';
  DateTime _lastKeyTime = DateTime.now();

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      context.read<ProductProvider>().loadProducts();
      context.read<CustomerProvider>().loadCustomers();
    });
  }

  @override
  void dispose() {
    _searchController.dispose();
    _barcodeController.dispose();
    _barcodeFocusNode.dispose();
    _debounce?.cancel();
    _tabController.dispose();
    super.dispose();
  }

  void _onSearchChanged(String query) {
    if (_debounce?.isActive ?? false) _debounce!.cancel();
    _debounce = Timer(const Duration(milliseconds: 300), () {
      context.read<ProductProvider>().loadProducts(query: query);
    });
  }

  /// Handle barcode scanner input (rapid key presses ending with Enter)
  void _handleBarcodeInput(KeyEvent event) {
    if (event is KeyDownEvent) {
      final now = DateTime.now();
      final timeDiff = now.difference(_lastKeyTime).inMilliseconds;

      // Reset buffer if too much time has passed (manual typing)
      if (timeDiff > 100) {
        _barcodeBuffer = '';
      }
      _lastKeyTime = now;

      if (event.logicalKey == LogicalKeyboardKey.enter &&
          _barcodeBuffer.isNotEmpty) {
        _processBarcodeSearch(_barcodeBuffer);
        _barcodeBuffer = '';
      } else if (event.character != null) {
        _barcodeBuffer += event.character!;
      }
    }
  }

  Future<void> _processBarcodeSearch(String barcode) async {
    // Search for product by barcode
    final provider = context.read<ProductProvider>();
    await provider.loadProducts(query: barcode);

    if (provider.products.length == 1) {
      // Auto-add if exactly one match
      _addToCart(provider.products.first);
    }
  }

  void _addToCart(Product product) {
    final cart = context.read<CartProvider>();
    if (_tabController.index == 0) {
      // Sale mode
      cart.addToSale(product);
    } else {
      // Trade-in mode - show condition dialog
      _showConditionGradingDialog(product);
    }
  }

  Future<void> _showConditionGradingDialog(Product product) async {
    final result = await showDialog<Map<String, dynamic>>(
      context: context,
      builder: (context) => ConditionGradingDialog(product: product),
    );

    if (result != null && mounted) {
      context.read<CartProvider>().addToTradeIn(
            product,
            condition: result['condition'] as String,
            offeredPrice: result['price'] as double?,
          );
    }
  }

  Future<void> _selectCustomer() async {
    final customer = await showDialog<Customer>(
      context: context,
      builder: (context) => _CustomerSelectionDialog(),
    );

    if (customer != null) {
      setState(() => _selectedCustomer = customer);
      context.read<CartProvider>().setCustomer(customer);
    }
  }

  Future<void> _processTransaction() async {
    final cart = context.read<CartProvider>();

    if (cart.saleItems.isEmpty && cart.tradeInItems.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Cart is empty')),
      );
      return;
    }

    final result = await showDialog<Map<String, dynamic>>(
      context: context,
      builder: (context) => PaymentDialog(
        saleTotal: cart.saleTotal,
        tradeInTotal: cart.tradeInTotal,
        customer: _selectedCustomer,
      ),
    );

    if (result != null && result['confirmed'] == true) {
      try {
        await _submitTransaction(result);
        cart.clear();
        setState(() => _selectedCustomer = null);
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(
              content: Text('Transaction completed successfully!'),
              backgroundColor: Colors.green,
            ),
          );
        }
      } catch (e) {
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('Error: $e'), backgroundColor: Colors.red),
          );
        }
      }
    }
  }

  Future<void> _submitTransaction(Map<String, dynamic> paymentDetails) async {
    final cart = context.read<CartProvider>();
    final api = context.read<ApiService>();

    // Submit sale items
    if (cart.saleItems.isNotEmpty) {
      await api.createTransaction(
        customerUuid: _selectedCustomer?.customerUuid,
        items:
            cart.saleItems.map((item) => item.toTransactionItemJson()).toList(),
        transactionType: 'Sale',
      );
    }

    // Submit trade-in items
    if (cart.tradeInItems.isNotEmpty) {
      await api.createTransaction(
        customerUuid: _selectedCustomer?.customerUuid,
        items: cart.tradeInItems
            .map((item) => item.toTransactionItemJson())
            .toList(),
        transactionType: 'Buy',
      );
    }
  }

  Future<void> _showDiscountDialog(CartProvider cart) async {
    if (cart.saleItems.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Add items to cart first')),
      );
      return;
    }

    final discount = await showDialog<Discount>(
      context: context,
      builder: (context) => DiscountDialog(
        isCartDiscount: true,
        currentSubtotal: cart.saleAfterItemDiscounts,
      ),
    );

    if (discount != null && mounted) {
      cart.applyCartDiscount(discount);
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Applied ${discount.name}'),
          backgroundColor: Colors.green,
        ),
      );
    }
  }

  Future<void> _showHoldDialog() async {
    final cart = context.read<CartProvider>();

    if (cart.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Cart is empty - nothing to hold')),
      );
      return;
    }

    final result = await showDialog<bool>(
      context: context,
      builder: (context) => const HoldTransactionDialog(),
    );

    if (result == true && mounted) {
      setState(() => _selectedCustomer = null);
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Transaction held'),
          backgroundColor: Colors.orange,
        ),
      );
    }
  }

  void _showHeldTransactionsSheet() {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      builder: (context) => DraggableScrollableSheet(
        initialChildSize: 0.5,
        minChildSize: 0.25,
        maxChildSize: 0.85,
        expand: false,
        builder: (context, scrollController) => Column(
          children: [
            Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: Colors.orange.shade50,
                borderRadius:
                    const BorderRadius.vertical(top: Radius.circular(16)),
              ),
              child: Row(
                children: [
                  Icon(Icons.pause_circle_outline,
                      color: Colors.orange.shade700),
                  const SizedBox(width: 8),
                  Text('Held Transactions',
                      style: TextStyle(
                          fontSize: 18,
                          fontWeight: FontWeight.bold,
                          color: Colors.orange.shade700)),
                  const Spacer(),
                  IconButton(
                    icon: const Icon(Icons.close),
                    onPressed: () => Navigator.pop(context),
                  ),
                ],
              ),
            ),
            const Expanded(child: HeldTransactionsPanel()),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final isWideScreen = MediaQuery.of(context).size.width > 1200;

    return KeyboardListener(
      focusNode: FocusNode(),
      onKeyEvent: _handleBarcodeInput,
      child: Scaffold(
        appBar: _buildAppBar(),
        body: isWideScreen ? _buildWideLayout() : _buildNarrowLayout(),
        bottomNavigationBar: _buildCheckoutBar(),
      ),
    );
  }

  PreferredSizeWidget _buildAppBar() {
    return AppBar(
      title: Row(
        children: [
          const Icon(Icons.point_of_sale),
          const SizedBox(width: 8),
          const Text('Point of Sale'),
          const SizedBox(width: 24),
          // Customer display
          if (_selectedCustomer != null) ...[
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
              decoration: BoxDecoration(
                color: Colors.white.withValues(alpha: 0.2),
                borderRadius: BorderRadius.circular(20),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Icon(Icons.person, size: 16),
                  const SizedBox(width: 4),
                  Text(_selectedCustomer!.name),
                  const SizedBox(width: 8),
                  InkWell(
                    onTap: () => setState(() => _selectedCustomer = null),
                    child: const Icon(Icons.close, size: 16),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
      actions: [
        // Quick scan button
        IconButton(
          icon: const Icon(Icons.qr_code_scanner),
          tooltip: 'Scan Barcode',
          onPressed: () => _barcodeFocusNode.requestFocus(),
        ),
        // Select customer
        IconButton(
          icon: const Icon(Icons.person_add),
          tooltip: 'Select Customer',
          onPressed: _selectCustomer,
        ),
        // Void transaction
        Consumer<CartProvider>(
          builder: (context, cart, _) => IconButton(
            icon: const Icon(Icons.delete_outline),
            tooltip: 'Clear Cart',
            onPressed: cart.isEmpty
                ? null
                : () {
                    showDialog(
                      context: context,
                      builder: (context) => AlertDialog(
                        title: const Text('Clear Cart?'),
                        content: const Text(
                            'This will remove all items from the cart.'),
                        actions: [
                          TextButton(
                            onPressed: () => Navigator.pop(context),
                            child: const Text('Cancel'),
                          ),
                          ElevatedButton(
                            onPressed: () {
                              cart.clear();
                              Navigator.pop(context);
                            },
                            style: ElevatedButton.styleFrom(
                                backgroundColor: Colors.red),
                            child: const Text('Clear'),
                          ),
                        ],
                      ),
                    );
                  },
          ),
        ),
      ],
      bottom: TabBar(
        controller: _tabController,
        tabs: const [
          Tab(
            icon: Icon(Icons.shopping_cart),
            text: 'SELL TO CUSTOMER',
          ),
          Tab(
            icon: Icon(Icons.swap_horiz),
            text: 'BUY FROM CUSTOMER',
          ),
        ],
      ),
    );
  }

  Widget _buildWideLayout() {
    return Row(
      children: [
        // Left side - Product search and grid
        Expanded(
          flex: 3,
          child: _buildProductPanel(),
        ),
        const VerticalDivider(width: 1),
        // Right side - Cart
        Expanded(
          flex: 2,
          child: _buildCartPanel(),
        ),
      ],
    );
  }

  Widget _buildNarrowLayout() {
    return TabBarView(
      controller: _tabController,
      children: [
        // Sale tab
        Column(
          children: [
            Expanded(child: _buildProductPanel()),
            _buildMiniCart(isSale: true),
          ],
        ),
        // Trade-in tab
        Column(
          children: [
            Expanded(child: _buildProductPanel()),
            _buildMiniCart(isSale: false),
          ],
        ),
      ],
    );
  }

  Widget _buildProductPanel() {
    return Column(
      children: [
        // Search bar with barcode support
        Padding(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              Expanded(
                child: TextField(
                  controller: _searchController,
                  decoration: InputDecoration(
                    hintText: 'Search products or scan barcode...',
                    prefixIcon: const Icon(Icons.search),
                    suffixIcon: _searchController.text.isNotEmpty
                        ? IconButton(
                            icon: const Icon(Icons.clear),
                            onPressed: () {
                              _searchController.clear();
                              context.read<ProductProvider>().loadProducts();
                            },
                          )
                        : null,
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(12),
                    ),
                    filled: true,
                  ),
                  onChanged: _onSearchChanged,
                ),
              ),
              const SizedBox(width: 8),
              // Category filter
              PopupMenuButton<Category?>(
                icon: const Icon(Icons.filter_list),
                tooltip: 'Filter by Category',
                onSelected: (category) {
                  // Filter products by category
                  context.read<ProductProvider>().filterByCategory(category);
                },
                itemBuilder: (context) => [
                  const PopupMenuItem(
                      value: null, child: Text('All Categories')),
                  ...Category.$valuesDefined.map(
                    (cat) =>
                        PopupMenuItem(value: cat, child: Text(cat.toString())),
                  ),
                ],
              ),
            ],
          ),
        ),
        // Product grid
        Expanded(
          child: Consumer<ProductProvider>(
            builder: (context, provider, _) {
              if (provider.isLoading) {
                return const Center(child: CircularProgressIndicator());
              }
              if (provider.error != null) {
                return Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      const Icon(Icons.error_outline,
                          size: 48, color: Colors.red),
                      const SizedBox(height: 16),
                      Text('Error: ${provider.error}'),
                      ElevatedButton(
                        onPressed: () => provider.loadProducts(),
                        child: const Text('Retry'),
                      ),
                    ],
                  ),
                );
              }
              if (provider.products.isEmpty) {
                return const Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.inventory_2_outlined,
                          size: 64, color: Colors.grey),
                      SizedBox(height: 16),
                      Text('No products found'),
                      Text('Try adjusting your search',
                          style: TextStyle(color: Colors.grey)),
                    ],
                  ),
                );
              }

              return GridView.builder(
                padding: const EdgeInsets.all(16),
                gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
                  maxCrossAxisExtent: 180,
                  childAspectRatio: 0.75,
                  crossAxisSpacing: 12,
                  mainAxisSpacing: 12,
                ),
                itemCount: provider.products.length,
                itemBuilder: (context, index) {
                  final product = provider.products[index];
                  return _ProductCard(
                    product: product,
                    onTap: () => _addToCart(product),
                    isTradeMode: _tabController.index == 1,
                  );
                },
              );
            },
          ),
        ),
      ],
    );
  }

  Widget _buildCartPanel() {
    return Consumer<CartProvider>(
      builder: (context, cart, _) {
        return Column(
          children: [
            // Cart header with toggle
            Container(
              padding: const EdgeInsets.all(16),
              color: Theme.of(context).colorScheme.surfaceContainerHighest,
              child: Row(
                children: [
                  Icon(
                    _tabController.index == 0
                        ? Icons.shopping_cart
                        : Icons.swap_horiz,
                    color: _tabController.index == 0
                        ? Colors.green
                        : Colors.orange,
                  ),
                  const SizedBox(width: 8),
                  Text(
                    _tabController.index == 0 ? 'Sale Items' : 'Trade-In Items',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                  const Spacer(),
                  Text(
                    '${_tabController.index == 0 ? cart.saleItems.length : cart.tradeInItems.length} items',
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                ],
              ),
            ),
            // Cart items
            Expanded(
              child: _tabController.index == 0
                  ? _buildSaleItemsList(cart)
                  : _buildTradeInItemsList(cart),
            ),
            // Totals section
            _buildTotalsSummary(cart),
          ],
        );
      },
    );
  }

  Widget _buildSaleItemsList(CartProvider cart) {
    if (cart.saleItems.isEmpty) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.add_shopping_cart, size: 48, color: Colors.grey),
            SizedBox(height: 8),
            Text('No items in cart'),
            Text('Tap products to add them',
                style: TextStyle(color: Colors.grey)),
          ],
        ),
      );
    }

    return ListView.builder(
      itemCount: cart.saleItems.length,
      itemBuilder: (context, index) {
        final item = cart.saleItems[index];
        return _CartItemTile(
          name: item.product.name,
          condition: item.condition,
          quantity: item.quantity,
          price: item.price,
          onIncrement: () => cart.updateSaleQuantity(index, item.quantity + 1),
          onDecrement: () {
            if (item.quantity > 1) {
              cart.updateSaleQuantity(index, item.quantity - 1);
            } else {
              cart.removeSaleItem(index);
            }
          },
          onRemove: () => cart.removeSaleItem(index),
        );
      },
    );
  }

  Widget _buildTradeInItemsList(CartProvider cart) {
    if (cart.tradeInItems.isEmpty) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.swap_horiz, size: 48, color: Colors.grey),
            SizedBox(height: 8),
            Text('No trade-in items'),
            Text('Tap products to add them',
                style: TextStyle(color: Colors.grey)),
          ],
        ),
      );
    }

    return ListView.builder(
      itemCount: cart.tradeInItems.length,
      itemBuilder: (context, index) {
        final item = cart.tradeInItems[index];
        return _CartItemTile(
          name: item.product.name,
          condition: item.condition,
          quantity: item.quantity,
          price: item.price,
          isTradeIn: true,
          onIncrement: () =>
              cart.updateTradeInQuantity(index, item.quantity + 1),
          onDecrement: () {
            if (item.quantity > 1) {
              cart.updateTradeInQuantity(index, item.quantity - 1);
            } else {
              cart.removeTradeInItem(index);
            }
          },
          onRemove: () => cart.removeTradeInItem(index),
        );
      },
    );
  }

  Widget _buildMiniCart({required bool isSale}) {
    return Consumer<CartProvider>(
      builder: (context, cart, _) {
        final items = isSale ? cart.saleItems : cart.tradeInItems;
        final total = isSale ? cart.saleTotal : cart.tradeInTotal;

        return Container(
          height: 120,
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            boxShadow: [
              BoxShadow(
                color: Colors.black.withValues(alpha: 0.1),
                blurRadius: 8,
                offset: const Offset(0, -4),
              ),
            ],
          ),
          child: Row(
            children: [
              Expanded(
                child: items.isEmpty
                    ? const Center(child: Text('Cart empty'))
                    : ListView.builder(
                        scrollDirection: Axis.horizontal,
                        padding: const EdgeInsets.all(8),
                        itemCount: items.length,
                        itemBuilder: (context, index) {
                          final item = items[index];
                          return Container(
                            width: 80,
                            margin: const EdgeInsets.only(right: 8),
                            decoration: BoxDecoration(
                              color: Colors.white,
                              borderRadius: BorderRadius.circular(8),
                            ),
                            child: Column(
                              mainAxisAlignment: MainAxisAlignment.center,
                              children: [
                                Text(
                                  item.product.name,
                                  maxLines: 2,
                                  overflow: TextOverflow.ellipsis,
                                  textAlign: TextAlign.center,
                                  style: const TextStyle(fontSize: 10),
                                ),
                                Text(
                                  'x${item.quantity}',
                                  style: const TextStyle(
                                      fontWeight: FontWeight.bold),
                                ),
                              ],
                            ),
                          );
                        },
                      ),
              ),
              Container(
                width: 100,
                padding: const EdgeInsets.all(8),
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Text(
                      isSale ? 'TOTAL' : 'CREDIT',
                      style: const TextStyle(
                          fontSize: 10, fontWeight: FontWeight.bold),
                    ),
                    Text(
                      '\$${total.toStringAsFixed(2)}',
                      style: TextStyle(
                        fontSize: 20,
                        fontWeight: FontWeight.bold,
                        color: isSale ? Colors.green : Colors.orange,
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildTotalsSummary(CartProvider cart) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        boxShadow: [
          BoxShadow(
            color: Colors.black.withValues(alpha: 0.1),
            blurRadius: 8,
            offset: const Offset(0, -4),
          ),
        ],
      ),
      child: Column(
        children: [
          // Subtotal
          _buildTotalRow('Subtotal', cart.saleSubtotal, Colors.grey.shade700),

          // Item discounts
          if (cart.saleItemDiscounts > 0) ...[
            const SizedBox(height: 4),
            _buildTotalRow('Item Discounts', -cart.saleItemDiscounts,
                Colors.green.shade600),
          ],

          // Cart discount
          if (cart.cartDiscount != null) ...[
            const SizedBox(height: 4),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Row(
                  children: [
                    Text(cart.cartDiscount!.name,
                        style: TextStyle(color: Colors.green.shade600)),
                    const SizedBox(width: 4),
                    InkWell(
                      onTap: () => cart.removeCartDiscount(),
                      child: Icon(Icons.close,
                          size: 14, color: Colors.red.shade400),
                    ),
                  ],
                ),
                Text('-\$${cart.cartDiscountAmount.toStringAsFixed(2)}',
                    style: TextStyle(
                        color: Colors.green.shade600,
                        fontWeight: FontWeight.bold)),
              ],
            ),
          ],

          // Tax
          const SizedBox(height: 4),
          InkWell(
            onTap: () => showDialog(
              context: context,
              builder: (context) => const TaxZoneDialog(),
            ),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Row(
                  children: [
                    Text('Tax (${cart.taxZone.name})',
                        style: const TextStyle(fontSize: 13)),
                    const SizedBox(width: 4),
                    const Icon(Icons.edit, size: 12, color: Colors.grey),
                  ],
                ),
                Text('+\$${cart.taxAmount.toStringAsFixed(2)}'),
              ],
            ),
          ),

          const SizedBox(height: 8),
          _buildTotalRow('Sale Total', cart.saleTotalWithTax, Colors.green),

          // Trade-in credit
          if (cart.tradeInTotal > 0) ...[
            const SizedBox(height: 8),
            _buildTotalRow(
                'Trade-In Credit', -cart.tradeInTotal, Colors.orange),
          ],

          const Divider(height: 24),
          _buildTotalRow(
            cart.netTotal >= 0 ? 'Customer Pays' : 'Store Owes',
            cart.netTotal.abs(),
            cart.netTotal >= 0 ? Colors.green.shade700 : Colors.orange.shade700,
            isLarge: true,
          ),
        ],
      ),
    );
  }

  Widget _buildTotalRow(String label, double amount, Color color,
      {bool isLarge = false}) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        Text(
          label,
          style: TextStyle(
            fontSize: isLarge ? 18 : 14,
            fontWeight: isLarge ? FontWeight.bold : FontWeight.normal,
          ),
        ),
        Text(
          '\$${amount.toStringAsFixed(2)}',
          style: TextStyle(
            fontSize: isLarge ? 24 : 16,
            fontWeight: FontWeight.bold,
            color: color,
          ),
        ),
      ],
    );
  }

  Widget _buildCheckoutBar() {
    return Consumer<CartProvider>(
      builder: (context, cart, _) {
        return Container(
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.primaryContainer,
          ),
          child: SafeArea(
            child: Row(
              children: [
                // Quick actions
                QuickActionsBar(
                  onAddCustomer: _selectCustomer,
                  onApplyDiscount: () => _showDiscountDialog(cart),
                  onHoldTransaction: () => _showHoldDialog(),
                ),
                // Held transactions badge
                if (cart.heldTransactions.isNotEmpty)
                  Badge(
                    label: Text('${cart.heldTransactions.length}'),
                    child: IconButton(
                      icon: const Icon(Icons.pause_circle_outline),
                      tooltip: 'Held Transactions',
                      onPressed: () => _showHeldTransactionsSheet(),
                    ),
                  ),
                // Tax zone chip
                const SizedBox(width: 8),
                const TaxZoneChip(),
                const Spacer(),
                // Checkout button
                SizedBox(
                  height: 56,
                  child: ElevatedButton.icon(
                    onPressed: cart.isEmpty ? null : _processTransaction,
                    icon: const Icon(Icons.payment, size: 28),
                    label: Text(
                      'CHECKOUT  \$${(cart.saleTotal - cart.tradeInTotal).abs().toStringAsFixed(2)}',
                      style: const TextStyle(
                          fontSize: 18, fontWeight: FontWeight.bold),
                    ),
                    style: ElevatedButton.styleFrom(
                      backgroundColor: Colors.green,
                      foregroundColor: Colors.white,
                      padding: const EdgeInsets.symmetric(horizontal: 32),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(12),
                      ),
                    ),
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}

// Product Card Widget
class _ProductCard extends StatelessWidget {
  final Product product;
  final VoidCallback onTap;
  final bool isTradeMode;

  const _ProductCard({
    required this.product,
    required this.onTap,
    this.isTradeMode = false,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      clipBehavior: Clip.antiAlias,
      child: InkWell(
        onTap: onTap,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // Product image placeholder
            Expanded(
              flex: 3,
              child: Container(
                color: Colors.grey.shade100,
                child: const Center(
                  child: Icon(Icons.image, size: 40, color: Colors.grey),
                ),
              ),
            ),
            // Product info
            Expanded(
              flex: 2,
              child: Padding(
                padding: const EdgeInsets.all(8),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      product.name,
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(
                        fontWeight: FontWeight.bold,
                        fontSize: 12,
                      ),
                    ),
                    const Spacer(),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Container(
                          padding: const EdgeInsets.symmetric(
                              horizontal: 6, vertical: 2),
                          decoration: BoxDecoration(
                            color: _getCategoryColor(product.category),
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(
                            product.category.toString(),
                            style: const TextStyle(
                                fontSize: 8, color: Colors.white),
                          ),
                        ),
                        Icon(
                          isTradeMode
                              ? Icons.swap_horiz
                              : Icons.add_shopping_cart,
                          size: 16,
                          color: isTradeMode ? Colors.orange : Colors.green,
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Color _getCategoryColor(Category category) {
    return switch (category) {
      Category.tcg => Colors.purple,
      Category.sportsCard => Colors.blue,
      Category.comic => Colors.red,
      Category.bobblehead => Colors.orange,
      Category.apparel => Colors.teal,
      Category.figure => Colors.indigo,
      Category.accessory => Colors.brown,
      _ => Colors.grey,
    };
  }
}

// Cart Item Tile Widget
class _CartItemTile extends StatelessWidget {
  final String name;
  final String? condition;
  final int quantity;
  final double price;
  final bool isTradeIn;
  final VoidCallback onIncrement;
  final VoidCallback onDecrement;
  final VoidCallback onRemove;

  const _CartItemTile({
    required this.name,
    this.condition,
    required this.quantity,
    required this.price,
    this.isTradeIn = false,
    required this.onIncrement,
    required this.onDecrement,
    required this.onRemove,
  });

  @override
  Widget build(BuildContext context) {
    return Dismissible(
      key: UniqueKey(),
      direction: DismissDirection.endToStart,
      onDismissed: (_) => onRemove(),
      background: Container(
        color: Colors.red,
        alignment: Alignment.centerRight,
        padding: const EdgeInsets.only(right: 16),
        child: const Icon(Icons.delete, color: Colors.white),
      ),
      child: ListTile(
        leading: Container(
          width: 40,
          height: 40,
          decoration: BoxDecoration(
            color: isTradeIn ? Colors.orange.shade100 : Colors.green.shade100,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Center(
            child: Text(
              condition ?? 'NM',
              style: TextStyle(
                fontWeight: FontWeight.bold,
                fontSize: 12,
                color:
                    isTradeIn ? Colors.orange.shade800 : Colors.green.shade800,
              ),
            ),
          ),
        ),
        title: Text(name, maxLines: 1, overflow: TextOverflow.ellipsis),
        subtitle: Text('\$${price.toStringAsFixed(2)} each'),
        trailing: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            IconButton(
              icon: const Icon(Icons.remove_circle_outline),
              onPressed: onDecrement,
              iconSize: 20,
            ),
            Text(
              '$quantity',
              style: const TextStyle(fontWeight: FontWeight.bold, fontSize: 16),
            ),
            IconButton(
              icon: const Icon(Icons.add_circle_outline),
              onPressed: onIncrement,
              iconSize: 20,
            ),
            const SizedBox(width: 8),
            Text(
              '\$${(price * quantity).toStringAsFixed(2)}',
              style: TextStyle(
                fontWeight: FontWeight.bold,
                color: isTradeIn ? Colors.orange : Colors.green,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

// Customer Selection Dialog
class _CustomerSelectionDialog extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Row(
        children: [
          const Text('Select Customer'),
          const Spacer(),
          IconButton(
            icon: const Icon(Icons.person_add),
            tooltip: 'Add New Customer',
            onPressed: () {
              // TODO: Navigate to add customer
              Navigator.pop(context);
            },
          ),
        ],
      ),
      content: SizedBox(
        width: 400,
        height: 400,
        child: Column(
          children: [
            TextField(
              decoration: const InputDecoration(
                hintText: 'Search customers...',
                prefixIcon: Icon(Icons.search),
                border: OutlineInputBorder(),
              ),
              onChanged: (query) {
                // TODO: Filter customers
              },
            ),
            const SizedBox(height: 16),
            Expanded(
              child: Consumer<CustomerProvider>(
                builder: (context, provider, child) {
                  if (provider.isLoading) {
                    return const Center(child: CircularProgressIndicator());
                  }
                  if (provider.error != null) {
                    return Center(child: Text('Error: ${provider.error}'));
                  }
                  if (provider.customers.isEmpty) {
                    return const Center(child: Text('No customers found'));
                  }

                  return ListView.builder(
                    itemCount: provider.customers.length,
                    itemBuilder: (context, index) {
                      final customer = provider.customers[index];
                      return ListTile(
                        leading: CircleAvatar(
                          child: Text(customer.name[0].toUpperCase()),
                        ),
                        title: Text(customer.name),
                        subtitle: Text(customer.email ?? customer.phone ?? ''),
                        trailing: customer.storeCredit > 0
                            ? Chip(
                                label: Text(
                                    '\$${customer.storeCredit.toStringAsFixed(2)}'),
                                backgroundColor: Colors.green.shade100,
                              )
                            : null,
                        onTap: () => Navigator.pop(context, customer),
                      );
                    },
                  );
                },
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
        TextButton(
          onPressed: () => Navigator.pop(context), // Guest checkout
          child: const Text('Guest Checkout'),
        ),
      ],
    );
  }
}
