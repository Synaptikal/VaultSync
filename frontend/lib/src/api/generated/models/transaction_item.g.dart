// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'transaction_item.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

TransactionItem _$TransactionItemFromJson(Map<String, dynamic> json) =>
    TransactionItem(
      condition: Condition.fromJson(json['condition'] as String),
      itemUuid: json['item_uuid'] as String,
      productUuid: json['product_uuid'] as String,
      quantity: (json['quantity'] as num).toInt(),
      unitPrice: (json['unit_price'] as num).toDouble(),
    );

Map<String, dynamic> _$TransactionItemToJson(TransactionItem instance) =>
    <String, dynamic>{
      'condition': instance.condition,
      'item_uuid': instance.itemUuid,
      'product_uuid': instance.productUuid,
      'quantity': instance.quantity,
      'unit_price': instance.unitPrice,
    };
