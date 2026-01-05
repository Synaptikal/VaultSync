// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

part 'price_info.g.dart';

@JsonSerializable()
class PriceInfo {
  const PriceInfo({
    required this.lastSyncTimestamp,
    required this.marketLow,
    required this.marketMid,
    required this.priceUuid,
    required this.productUuid,
  });
  
  factory PriceInfo.fromJson(Map<String, Object?> json) => _$PriceInfoFromJson(json);
  
  @JsonKey(name: 'last_sync_timestamp')
  final DateTime lastSyncTimestamp;
  @JsonKey(name: 'market_low')
  final double marketLow;
  @JsonKey(name: 'market_mid')
  final double marketMid;
  @JsonKey(name: 'price_uuid')
  final String priceUuid;
  @JsonKey(name: 'product_uuid')
  final String productUuid;

  Map<String, Object?> toJson() => _$PriceInfoToJson(this);
}
