import 'package:flutter/foundation.dart';
import 'package:uuid/uuid.dart';
import '../api/generated/models/product.dart';
import '../api/generated/models/condition.dart';
import '../api/generated/models/customer.dart';
import '../models/buylist_models.dart';
import '../services/api_service.dart';

/// Discount types
enum DiscountType { percentage, fixed }

/// Represents a discount applied to cart or item
class Discount {
  final String id;
  final String name;
  final DiscountType type;
  final double value;
  final String? reason;

  Discount({
    String? id,
    required this.name,
    required this.type,
    required this.value,
    this.reason,
  }) : id = id ?? const Uuid().v4();

  double apply(double subtotal) {
    if (type == DiscountType.percentage) {
      return subtotal * (value / 100);
    }
    return value.clamp(0, subtotal);
  }
}

/// Tax zone configuration
class TaxZone {
  final String id;
  final String name;
  final double rate; // e.g., 0.0825 for 8.25%
  final bool isDefault;

  const TaxZone({
    required this.id,
    required this.name,
    required this.rate,
    this.isDefault = false,
  });

  static const TaxZone noTax = TaxZone(id: 'none', name: 'Tax Exempt', rate: 0.0);
  static const TaxZone standard = TaxZone(id: 'standard', name: 'Standard (8.25%)', rate: 0.0825, isDefault: true);
  static const TaxZone reduced = TaxZone(id: 'reduced', name: 'Reduced (5%)', rate: 0.05);
  static const TaxZone high = TaxZone(id: 'high', name: 'High (10%)', rate: 0.10);

  static List<TaxZone> get allZones => [noTax, standard, reduced, high];
}

/// Held transaction for later recall
class HeldTransaction {
  final String id;
  final String name;
  final DateTime heldAt;
  final Customer? customer;
  final List<CartItem> saleItems;
  final List<CartItem> tradeInItems;
  final Discount? cartDiscount;
  final TaxZone taxZone;

  HeldTransaction({
    String? id,
    required this.name,
    required this.heldAt,
    this.customer,
    required this.saleItems,
    required this.tradeInItems,
    this.cartDiscount,
    required this.taxZone,
  }) : id = id ?? const Uuid().v4();
}

class CartItem {
  final String itemUuid;
  final Product product;
  int quantity;
  double price;
  String condition;
  Discount? discount;

  CartItem({
    required this.itemUuid,
    required this.product,
    required this.quantity,
    required this.price,
    required this.condition,
    this.discount,
  });

  double get subtotal => quantity * price;
  
  double get discountAmount => discount?.apply(subtotal) ?? 0.0;
  
  double get total => subtotal - discountAmount;

  Map<String, dynamic> toTransactionItemJson() {
    return {
      'item_uuid': itemUuid,
      'product_uuid': product.productUuid,
      'quantity': quantity,
      'unit_price': price,
      'condition': condition,
      'discount_amount': discountAmount,
    };
  }
  
  CartItem copyWith({
    String? itemUuid,
    Product? product,
    int? quantity,
    double? price,
    String? condition,
    Discount? discount,
  }) {
    return CartItem(
      itemUuid: itemUuid ?? this.itemUuid,
      product: product ?? this.product,
      quantity: quantity ?? this.quantity,
      price: price ?? this.price,
      condition: condition ?? this.condition,
      discount: discount ?? this.discount,
    );
  }
}

class CartProvider with ChangeNotifier {
  final ApiService _apiService;
  final List<CartItem> _saleItems = [];
  final List<CartItem> _tradeInItems = [];
  final List<HeldTransaction> _heldTransactions = [];
  Customer? _customer;
  Discount? _cartDiscount;
  TaxZone _taxZone = TaxZone.standard;

  CartProvider(this._apiService);

  // Getters
  List<CartItem> get saleItems => List.unmodifiable(_saleItems);
  List<CartItem> get tradeInItems => List.unmodifiable(_tradeInItems);
  List<HeldTransaction> get heldTransactions => List.unmodifiable(_heldTransactions);
  Customer? get customer => _customer;
  Discount? get cartDiscount => _cartDiscount;
  TaxZone get taxZone => _taxZone;
  
  // Backward compatibility
  List<CartItem> get items => saleItems;

  // Sale calculations
  double get saleSubtotal => _saleItems.fold(0.0, (sum, item) => sum + item.subtotal);
  double get saleItemDiscounts => _saleItems.fold(0.0, (sum, item) => sum + item.discountAmount);
  double get saleAfterItemDiscounts => saleSubtotal - saleItemDiscounts;
  double get cartDiscountAmount => _cartDiscount?.apply(saleAfterItemDiscounts) ?? 0.0;
  double get saleTotal => saleAfterItemDiscounts - cartDiscountAmount;
  
  // Tax calculations
  double get taxAmount => saleTotal * _taxZone.rate;
  double get saleTotalWithTax => saleTotal + taxAmount;
  
  // Trade-in calculations
  double get tradeInTotal => _tradeInItems.fold(0.0, (sum, item) => sum + item.total);
  
  // Net total (customer pays or store owes)
  double get netTotal => saleTotalWithTax - tradeInTotal;
  double get customerPays => netTotal > 0 ? netTotal : 0;
  double get storeOwes => netTotal < 0 ? netTotal.abs() : 0;
  
  // Legacy aliases
  double get totalSaleAmount => saleTotalWithTax;
  double get totalTradeInAmount => tradeInTotal;
  
  bool get isEmpty => _saleItems.isEmpty && _tradeInItems.isEmpty;
  bool get hasDiscount => _cartDiscount != null || _saleItems.any((i) => i.discount != null);

  bool _isProcessing = false;
  bool get isProcessing => _isProcessing;

  // ============================================
  // Customer Management
  // ============================================
  
  void setCustomer(Customer? customer) {
    _customer = customer;
    notifyListeners();
  }

  // ============================================
  // Tax Zone Management
  // ============================================
  
  void setTaxZone(TaxZone zone) {
    _taxZone = zone;
    notifyListeners();
  }

  // ============================================
  // Discount Management
  // ============================================
  
  void applyCartDiscount(Discount discount) {
    _cartDiscount = discount;
    notifyListeners();
  }

  void removeCartDiscount() {
    _cartDiscount = null;
    notifyListeners();
  }

  void applyItemDiscount(int index, Discount discount, {bool isSale = true}) {
    final items = isSale ? _saleItems : _tradeInItems;
    if (index >= 0 && index < items.length) {
      items[index].discount = discount;
      notifyListeners();
    }
  }

  void removeItemDiscount(int index, {bool isSale = true}) {
    final items = isSale ? _saleItems : _tradeInItems;
    if (index >= 0 && index < items.length) {
      items[index].discount = null;
      notifyListeners();
    }
  }

  // ============================================
  // Hold Transaction Management
  // ============================================
  
  void holdCurrentTransaction({String? name}) {
    if (isEmpty) return;
    
    final holdName = name ?? 'Hold ${_heldTransactions.length + 1}';
    
    final held = HeldTransaction(
      name: holdName,
      heldAt: DateTime.now(),
      customer: _customer,
      saleItems: _saleItems.map((i) => i.copyWith()).toList(),
      tradeInItems: _tradeInItems.map((i) => i.copyWith()).toList(),
      cartDiscount: _cartDiscount,
      taxZone: _taxZone,
    );
    
    _heldTransactions.add(held);
    clear();
  }

  void recallTransaction(String holdId) {
    final index = _heldTransactions.indexWhere((h) => h.id == holdId);
    if (index == -1) return;
    
    final held = _heldTransactions[index];
    
    // Save current cart if not empty
    if (!isEmpty) {
      holdCurrentTransaction(name: 'Auto-Hold ${DateTime.now().millisecond}');
    }
    
    // Restore held transaction
    _saleItems.clear();
    _saleItems.addAll(held.saleItems);
    _tradeInItems.clear();
    _tradeInItems.addAll(held.tradeInItems);
    _customer = held.customer;
    _cartDiscount = held.cartDiscount;
    _taxZone = held.taxZone;
    
    // Remove from held list
    _heldTransactions.removeAt(index);
    
    notifyListeners();
  }

  void deleteHeldTransaction(String holdId) {
    _heldTransactions.removeWhere((h) => h.id == holdId);
    notifyListeners();
  }

  // ============================================
  // Sale Item Management
  // ============================================

  void addToSale(Product product, {double price = 10.0, String condition = 'NM'}) {
    // Check if product already in cart with same condition
    final existingIndex = _saleItems.indexWhere(
      (item) => item.product.productUuid == product.productUuid && item.condition == condition
    );
    
    if (existingIndex != -1) {
      _saleItems[existingIndex].quantity++;
    } else {
      final newItem = CartItem(
        itemUuid: const Uuid().v4(),
        product: product,
        quantity: 1,
        price: price,
        condition: condition,
      );
      _saleItems.add(newItem);
    }
    notifyListeners();
  }

  // ============================================
  // Trade-In Item Management
  // ============================================

  Future<void> addToTradeIn(Product product, {String condition = 'NM', double? offeredPrice}) async {
    double price = offeredPrice ?? 0.0;
    
    if (offeredPrice == null) {
      try {
        Condition condEnum = Condition.values.firstWhere(
          (e) => e.json == condition,
          orElse: () => Condition.nm,
        );
        
        final item = BuylistItem(
          productUuid: product.productUuid,
          condition: condEnum,
          quantity: 1
        );
        
        final quote = await _apiService.getBuylistQuote(item);
        price = quote.cashPrice;
      } catch (e) {
        if (kDebugMode) print('Failed to get quote: $e');
      }
    }

    final newItem = CartItem(
      itemUuid: const Uuid().v4(),
      product: product,
      quantity: 1,
      price: price,
      condition: condition,
    );
    _tradeInItems.add(newItem);
    notifyListeners();
  }

  // ============================================
  // Item Operations
  // ============================================

  void addToCart(Product product, {double price = 10.0, String condition = 'NM'}) {
    addToSale(product, price: price, condition: condition);
  }

  void removeFromSale(String itemUuid) {
    _saleItems.removeWhere((item) => item.itemUuid == itemUuid);
    notifyListeners();
  }

  void removeFromTradeIn(String itemUuid) {
    _tradeInItems.removeWhere((item) => item.itemUuid == itemUuid);
    notifyListeners();
  }
  
  void removeSaleItem(int index) {
    if (index >= 0 && index < _saleItems.length) {
      _saleItems.removeAt(index);
      notifyListeners();
    }
  }

  void removeTradeInItem(int index) {
    if (index >= 0 && index < _tradeInItems.length) {
      _tradeInItems.removeAt(index);
      notifyListeners();
    }
  }
  
  void updateSaleQuantity(int index, int quantity) {
    if (index >= 0 && index < _saleItems.length) {
      if (quantity <= 0) {
        _saleItems.removeAt(index);
      } else {
        _saleItems[index].quantity = quantity;
      }
      notifyListeners();
    }
  }

  void updateTradeInQuantity(int index, int quantity) {
    if (index >= 0 && index < _tradeInItems.length) {
      if (quantity <= 0) {
        _tradeInItems.removeAt(index);
      } else {
        _tradeInItems[index].quantity = quantity;
      }
      notifyListeners();
    }
  }

  void updateSaleItem(String itemUuid, {int? quantity, double? price}) {
    final index = _saleItems.indexWhere((item) => item.itemUuid == itemUuid);
    if (index != -1) {
      final item = _saleItems[index];
      item.quantity = quantity ?? item.quantity;
      item.price = price ?? item.price;
      notifyListeners();
    }
  }

  void updateTradeInItem(String itemUuid, {int? quantity, double? price, String? condition}) {
    final index = _tradeInItems.indexWhere((item) => item.itemUuid == itemUuid);
    if (index != -1) {
      final item = _tradeInItems[index];
      item.quantity = quantity ?? item.quantity;
      item.price = price ?? item.price;
      item.condition = condition ?? item.condition;
      notifyListeners();
    }
  }

  void removeFromCart(String itemUuid) {
    removeFromSale(itemUuid);
  }

  void clear() {
    _saleItems.clear();
    _tradeInItems.clear();
    _customer = null;
    _cartDiscount = null;
    notifyListeners();
  }

  void clearCart() => clear();

  // ============================================
  // Checkout
  // ============================================

  Future<void> checkout(String? customerUuid) async {
    if (_saleItems.isEmpty && _tradeInItems.isEmpty) return;

    _isProcessing = true;
    notifyListeners();

    try {
      if (_saleItems.isNotEmpty && _tradeInItems.isEmpty) {
        final saleItemsJson = _saleItems.map((item) => item.toTransactionItemJson()).toList();
        await _apiService.createTransaction(
          customerUuid: customerUuid, 
          items: saleItemsJson,
          transactionType: 'Sale'
        );
      } else if (_saleItems.isEmpty && _tradeInItems.isNotEmpty) {
        final buylistItems = _tradeInItems.map((item) {
           Condition condEnum = Condition.values.firstWhere(
            (e) => e.json == item.condition,
            orElse: () => Condition.nm,
          );
          return BuylistItem(
            productUuid: item.product.productUuid,
            condition: condEnum,
            quantity: item.quantity
          );
        }).toList();

        await _apiService.processBuylist(
          items: buylistItems,
          customerUuid: customerUuid,
          paymentMethod: PaymentMethod.cash,
        );
      } else {
        final buylistItems = _tradeInItems.map((item) {
           Condition condEnum = Condition.values.firstWhere(
            (e) => e.json == item.condition,
            orElse: () => Condition.nm,
          );
          return BuylistItem(
            productUuid: item.product.productUuid,
            condition: condEnum,
            quantity: item.quantity
          );
        }).toList();
        
        final purchaseItems = _saleItems.map((item) => item.toTransactionItemJson()).toList();
        
        await _apiService.processTradeIn(
          tradeInItems: buylistItems,
          purchaseItems: purchaseItems,
          customerUuid: customerUuid,
        );
      }
      
      clear();
    } catch (e) {
      rethrow;
    } finally {
      _isProcessing = false;
      notifyListeners();
    }
  }
}
