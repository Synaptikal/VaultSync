// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

import 'transaction_item.dart';
import 'transaction_type.dart';

part 'transaction.g.dart';

@JsonSerializable()
class Transaction {
  const Transaction({
    required this.items,
    required this.timestamp,
    required this.transactionType,
    required this.transactionUuid,
    this.customerUuid,
    this.userUuid,
  });
  
  factory Transaction.fromJson(Map<String, Object?> json) => _$TransactionFromJson(json);
  
  @JsonKey(name: 'customer_uuid')
  final String? customerUuid;
  final List<TransactionItem> items;
  final DateTime timestamp;
  @JsonKey(name: 'transaction_type')
  final TransactionType transactionType;
  @JsonKey(name: 'transaction_uuid')
  final String transactionUuid;
  @JsonKey(name: 'user_uuid')
  final String? userUuid;

  Map<String, Object?> toJson() => _$TransactionToJson(this);
}
