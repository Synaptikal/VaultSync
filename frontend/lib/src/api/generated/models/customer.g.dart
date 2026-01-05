// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'customer.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

Customer _$CustomerFromJson(Map<String, dynamic> json) => Customer(
      createdAt: DateTime.parse(json['created_at'] as String),
      customerUuid: json['customer_uuid'] as String,
      name: json['name'] as String,
      storeCredit: (json['store_credit'] as num).toDouble(),
      email: json['email'] as String?,
      phone: json['phone'] as String?,
      tier: json['tier'] as String?,
    );

Map<String, dynamic> _$CustomerToJson(Customer instance) => <String, dynamic>{
      'created_at': instance.createdAt.toIso8601String(),
      'customer_uuid': instance.customerUuid,
      'email': instance.email,
      'name': instance.name,
      'phone': instance.phone,
      'store_credit': instance.storeCredit,
      'tier': instance.tier,
    };
