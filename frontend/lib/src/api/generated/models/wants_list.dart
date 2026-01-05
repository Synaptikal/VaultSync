// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

import 'wants_item.dart';

part 'wants_list.g.dart';

@JsonSerializable()
class WantsList {
  const WantsList({
    required this.createdAt,
    required this.customerUuid,
    required this.items,
    required this.wantsListUuid,
  });
  
  factory WantsList.fromJson(Map<String, Object?> json) => _$WantsListFromJson(json);
  
  @JsonKey(name: 'created_at')
  final DateTime createdAt;
  @JsonKey(name: 'customer_uuid')
  final String customerUuid;
  final List<WantsItem> items;
  @JsonKey(name: 'wants_list_uuid')
  final String wantsListUuid;

  Map<String, Object?> toJson() => _$WantsListToJson(this);
}
