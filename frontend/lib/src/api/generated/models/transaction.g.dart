// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'transaction.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

Transaction _$TransactionFromJson(Map<String, dynamic> json) => Transaction(
      items: (json['items'] as List<dynamic>)
          .map((e) => TransactionItem.fromJson(e as Map<String, dynamic>))
          .toList(),
      timestamp: DateTime.parse(json['timestamp'] as String),
      transactionType:
          TransactionType.fromJson(json['transaction_type'] as String),
      transactionUuid: json['transaction_uuid'] as String,
      customerUuid: json['customer_uuid'] as String?,
      userUuid: json['user_uuid'] as String?,
    );

Map<String, dynamic> _$TransactionToJson(Transaction instance) =>
    <String, dynamic>{
      'customer_uuid': instance.customerUuid,
      'items': instance.items,
      'timestamp': instance.timestamp.toIso8601String(),
      'transaction_type': instance.transactionType,
      'transaction_uuid': instance.transactionUuid,
      'user_uuid': instance.userUuid,
    };
