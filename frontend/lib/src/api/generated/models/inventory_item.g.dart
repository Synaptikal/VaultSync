// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'inventory_item.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

InventoryItem _$InventoryItemFromJson(Map<String, dynamic> json) =>
    InventoryItem(
      condition: Condition.fromJson(json['condition'] as String),
      inventoryUuid: json['inventory_uuid'] as String,
      locationTag: json['location_tag'] as String,
      minStockLevel: (json['min_stock_level'] as num).toInt(),
      productUuid: json['product_uuid'] as String,
      quantityOnHand: (json['quantity_on_hand'] as num).toInt(),
      serializedDetails: json['serialized_details'],
      binLocation: json['bin_location'] as String?,
      costBasis: (json['cost_basis'] as num?)?.toDouble(),
      deletedAt: json['deleted_at'] == null
          ? null
          : DateTime.parse(json['deleted_at'] as String),
      lastCountedDate: json['last_counted_date'] == null
          ? null
          : DateTime.parse(json['last_counted_date'] as String),
      lastSoldDate: json['last_sold_date'] == null
          ? null
          : DateTime.parse(json['last_sold_date'] as String),
      maxStockLevel: (json['max_stock_level'] as num?)?.toInt(),
      receivedDate: json['received_date'] == null
          ? null
          : DateTime.parse(json['received_date'] as String),
      reorderPoint: (json['reorder_point'] as num?)?.toInt(),
      specificPrice: (json['specific_price'] as num?)?.toDouble(),
      supplierUuid: json['supplier_uuid'] as String?,
      variantType: json['variant_type'] == null
          ? null
          : VariantType.fromJson(json['variant_type'] as String),
    );

Map<String, dynamic> _$InventoryItemToJson(InventoryItem instance) =>
    <String, dynamic>{
      'bin_location': instance.binLocation,
      'condition': instance.condition,
      'cost_basis': instance.costBasis,
      'deleted_at': instance.deletedAt?.toIso8601String(),
      'inventory_uuid': instance.inventoryUuid,
      'last_counted_date': instance.lastCountedDate?.toIso8601String(),
      'last_sold_date': instance.lastSoldDate?.toIso8601String(),
      'location_tag': instance.locationTag,
      'max_stock_level': instance.maxStockLevel,
      'min_stock_level': instance.minStockLevel,
      'product_uuid': instance.productUuid,
      'quantity_on_hand': instance.quantityOnHand,
      'received_date': instance.receivedDate?.toIso8601String(),
      'reorder_point': instance.reorderPoint,
      'serialized_details': instance.serializedDetails,
      'specific_price': instance.specificPrice,
      'supplier_uuid': instance.supplierUuid,
      'variant_type': instance.variantType,
    };
