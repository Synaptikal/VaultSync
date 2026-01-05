// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

@JsonEnum()
enum RecordType {
  @JsonValue('Product')
  product('Product'),
  @JsonValue('InventoryItem')
  inventoryItem('InventoryItem'),
  @JsonValue('PriceInfo')
  priceInfo('PriceInfo'),
  @JsonValue('Transaction')
  transaction('Transaction'),
  @JsonValue('Customer')
  customer('Customer'),
  @JsonValue('WantsList')
  wantsList('WantsList'),
  @JsonValue('Event')
  event('Event'),
  @JsonValue('EventParticipant')
  eventParticipant('EventParticipant'),
  /// Default value for all unparsed values, allows backward compatibility when adding new values on the backend.
  $unknown(null);

  const RecordType(this.json);

  factory RecordType.fromJson(String json) => values.firstWhere(
        (e) => e.json == json,
        orElse: () => $unknown,
      );

  final String? json;

  String? toJson() => json;

  @override
  String toString() => json?.toString() ?? super.toString();
  /// Returns all defined enum values excluding the $unknown value.
  static List<RecordType> get $valuesDefined => values.where((value) => value != $unknown).toList();
}
