// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

import 'condition.dart';

part 'wants_item.g.dart';

@JsonSerializable()
class WantsItem {
  const WantsItem({
    required this.createdAt,
    required this.itemUuid,
    required this.minCondition,
    required this.productUuid,
    this.maxPrice,
  });
  
  factory WantsItem.fromJson(Map<String, Object?> json) => _$WantsItemFromJson(json);
  
  @JsonKey(name: 'created_at')
  final DateTime createdAt;
  @JsonKey(name: 'item_uuid')
  final String itemUuid;
  @JsonKey(name: 'max_price')
  final double? maxPrice;
  @JsonKey(name: 'min_condition')
  final Condition minCondition;
  @JsonKey(name: 'product_uuid')
  final String productUuid;

  Map<String, Object?> toJson() => _$WantsItemToJson(this);
}
