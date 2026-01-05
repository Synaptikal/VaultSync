// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

import 'condition.dart';
import 'variant_type.dart';

part 'inventory_item.g.dart';

@JsonSerializable()
class InventoryItem {
  const InventoryItem({
    required this.condition,
    required this.inventoryUuid,
    required this.locationTag,
    required this.minStockLevel,
    required this.productUuid,
    required this.quantityOnHand,
    required this.serializedDetails,
    this.binLocation,
    this.costBasis,
    this.deletedAt,
    this.lastCountedDate,
    this.lastSoldDate,
    this.maxStockLevel,
    this.receivedDate,
    this.reorderPoint,
    this.specificPrice,
    this.supplierUuid,
    this.variantType,
  });
  
  factory InventoryItem.fromJson(Map<String, Object?> json) => _$InventoryItemFromJson(json);
  
  @JsonKey(name: 'bin_location')
  final String? binLocation;
  final Condition condition;
  @JsonKey(name: 'cost_basis')
  final double? costBasis;
  @JsonKey(name: 'deleted_at')
  final DateTime? deletedAt;
  @JsonKey(name: 'inventory_uuid')
  final String inventoryUuid;
  @JsonKey(name: 'last_counted_date')
  final DateTime? lastCountedDate;
  @JsonKey(name: 'last_sold_date')
  final DateTime? lastSoldDate;
  @JsonKey(name: 'location_tag')
  final String locationTag;
  @JsonKey(name: 'max_stock_level')
  final int? maxStockLevel;
  @JsonKey(name: 'min_stock_level')
  final int minStockLevel;
  @JsonKey(name: 'product_uuid')
  final String productUuid;
  @JsonKey(name: 'quantity_on_hand')
  final int quantityOnHand;
  @JsonKey(name: 'received_date')
  final DateTime? receivedDate;
  @JsonKey(name: 'reorder_point')
  final int? reorderPoint;
  @JsonKey(name: 'serialized_details')
  final dynamic serializedDetails;
  @JsonKey(name: 'specific_price')
  final double? specificPrice;
  @JsonKey(name: 'supplier_uuid')
  final String? supplierUuid;
  @JsonKey(name: 'variant_type')
  final VariantType? variantType;

  Map<String, Object?> toJson() => _$InventoryItemToJson(this);
}
