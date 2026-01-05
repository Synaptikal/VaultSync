// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'wants_list.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

WantsList _$WantsListFromJson(Map<String, dynamic> json) => WantsList(
      createdAt: DateTime.parse(json['created_at'] as String),
      customerUuid: json['customer_uuid'] as String,
      items: (json['items'] as List<dynamic>)
          .map((e) => WantsItem.fromJson(e as Map<String, dynamic>))
          .toList(),
      wantsListUuid: json['wants_list_uuid'] as String,
    );

Map<String, dynamic> _$WantsListToJson(WantsList instance) => <String, dynamic>{
      'created_at': instance.createdAt.toIso8601String(),
      'customer_uuid': instance.customerUuid,
      'items': instance.items,
      'wants_list_uuid': instance.wantsListUuid,
    };
