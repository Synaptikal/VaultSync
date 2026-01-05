// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

import 'condition.dart';

part 'transaction_item.g.dart';

@JsonSerializable()
class TransactionItem {
  const TransactionItem({
    required this.condition,
    required this.itemUuid,
    required this.productUuid,
    required this.quantity,
    required this.unitPrice,
  });
  
  factory TransactionItem.fromJson(Map<String, Object?> json) => _$TransactionItemFromJson(json);
  
  final Condition condition;
  @JsonKey(name: 'item_uuid')
  final String itemUuid;
  @JsonKey(name: 'product_uuid')
  final String productUuid;
  final int quantity;
  @JsonKey(name: 'unit_price')
  final double unitPrice;

  Map<String, Object?> toJson() => _$TransactionItemToJson(this);
}
