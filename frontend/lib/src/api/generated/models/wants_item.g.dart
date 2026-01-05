// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'wants_item.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

WantsItem _$WantsItemFromJson(Map<String, dynamic> json) => WantsItem(
      createdAt: DateTime.parse(json['created_at'] as String),
      itemUuid: json['item_uuid'] as String,
      minCondition: Condition.fromJson(json['min_condition'] as String),
      productUuid: json['product_uuid'] as String,
      maxPrice: (json['max_price'] as num?)?.toDouble(),
    );

Map<String, dynamic> _$WantsItemToJson(WantsItem instance) => <String, dynamic>{
      'created_at': instance.createdAt.toIso8601String(),
      'item_uuid': instance.itemUuid,
      'max_price': instance.maxPrice,
      'min_condition': instance.minCondition,
      'product_uuid': instance.productUuid,
    };
