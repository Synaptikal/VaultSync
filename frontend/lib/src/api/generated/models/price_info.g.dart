// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'price_info.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

PriceInfo _$PriceInfoFromJson(Map<String, dynamic> json) => PriceInfo(
      lastSyncTimestamp: DateTime.parse(json['last_sync_timestamp'] as String),
      marketLow: (json['market_low'] as num).toDouble(),
      marketMid: (json['market_mid'] as num).toDouble(),
      priceUuid: json['price_uuid'] as String,
      productUuid: json['product_uuid'] as String,
    );

Map<String, dynamic> _$PriceInfoToJson(PriceInfo instance) => <String, dynamic>{
      'last_sync_timestamp': instance.lastSyncTimestamp.toIso8601String(),
      'market_low': instance.marketLow,
      'market_mid': instance.marketMid,
      'price_uuid': instance.priceUuid,
      'product_uuid': instance.productUuid,
    };
