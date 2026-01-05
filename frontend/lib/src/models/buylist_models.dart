import '../api/generated/models/condition.dart';

enum PaymentMethod {
  cash,
  storeCredit,
}

class BuylistItem {
  final String productUuid;
  final Condition condition;
  final int quantity;

  BuylistItem({
    required this.productUuid,
    required this.condition,
    required this.quantity,
  });

  Map<String, dynamic> toJson() => {
    'product_uuid': productUuid,
    'condition': condition.json, // Using the json getter from the generated enum
    'quantity': quantity,
  };
}

class QuoteResult {
  final double cashPrice;
  final double creditPrice;
  final double marketPrice;

  QuoteResult({
    required this.cashPrice,
    required this.creditPrice,
    required this.marketPrice,
  });

  factory QuoteResult.fromJson(Map<String, dynamic> json) {
    return QuoteResult(
      cashPrice: (json['cash_price'] as num).toDouble(),
      creditPrice: (json['credit_price'] as num).toDouble(),
      marketPrice: (json['market_price'] as num).toDouble(),
    );
  }
}
