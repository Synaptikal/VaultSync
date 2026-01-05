// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

part 'customer.g.dart';

@JsonSerializable()
class Customer {
  const Customer({
    required this.createdAt,
    required this.customerUuid,
    required this.name,
    required this.storeCredit,
    this.email,
    this.phone,
    this.tier,
  });
  
  factory Customer.fromJson(Map<String, Object?> json) => _$CustomerFromJson(json);
  
  @JsonKey(name: 'created_at')
  final DateTime createdAt;
  @JsonKey(name: 'customer_uuid')
  final String customerUuid;
  final String? email;
  final String name;
  final String? phone;
  @JsonKey(name: 'store_credit')
  final double storeCredit;
  final String? tier;

  Map<String, Object?> toJson() => _$CustomerToJson(this);
}
