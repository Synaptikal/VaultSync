// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'audit_discrepancy.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

AuditDiscrepancy _$AuditDiscrepancyFromJson(Map<String, dynamic> json) =>
    AuditDiscrepancy(
      productUuid: json['product_uuid'] as String,
      productName: json['product_name'] as String?,
      condition: json['condition'] as String,
      expectedQuantity: (json['expected_quantity'] as num).toInt(),
      actualQuantity: (json['actual_quantity'] as num).toInt(),
      variance: (json['variance'] as num).toInt(),
      locationTag: json['location_tag'] as String?,
    );

Map<String, dynamic> _$AuditDiscrepancyToJson(AuditDiscrepancy instance) =>
    <String, dynamic>{
      'product_uuid': instance.productUuid,
      'product_name': instance.productName,
      'condition': instance.condition,
      'expected_quantity': instance.expectedQuantity,
      'actual_quantity': instance.actualQuantity,
      'variance': instance.variance,
      'location_tag': instance.locationTag,
    };
