// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'product.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

Product _$ProductFromJson(Map<String, dynamic> json) => Product(
      category: Category.fromJson(json['category'] as String),
      metadata: json['metadata'],
      name: json['name'] as String,
      productUuid: json['product_uuid'] as String,
      barcode: json['barcode'] as String?,
      collectorNumber: json['collector_number'] as String?,
      deletedAt: json['deleted_at'] == null
          ? null
          : DateTime.parse(json['deleted_at'] as String),
      heightIn: (json['height_in'] as num?)?.toDouble(),
      isbn: json['isbn'] as String?,
      lengthIn: (json['length_in'] as num?)?.toDouble(),
      manufacturer: json['manufacturer'] as String?,
      msrp: (json['msrp'] as num?)?.toDouble(),
      releaseYear: (json['release_year'] as num?)?.toInt(),
      setCode: json['set_code'] as String?,
      upc: json['upc'] as String?,
      weightOz: (json['weight_oz'] as num?)?.toDouble(),
      widthIn: (json['width_in'] as num?)?.toDouble(),
    );

Map<String, dynamic> _$ProductToJson(Product instance) => <String, dynamic>{
      'barcode': instance.barcode,
      'category': instance.category,
      'collector_number': instance.collectorNumber,
      'deleted_at': instance.deletedAt?.toIso8601String(),
      'height_in': instance.heightIn,
      'isbn': instance.isbn,
      'length_in': instance.lengthIn,
      'manufacturer': instance.manufacturer,
      'metadata': instance.metadata,
      'msrp': instance.msrp,
      'name': instance.name,
      'product_uuid': instance.productUuid,
      'release_year': instance.releaseYear,
      'set_code': instance.setCode,
      'upc': instance.upc,
      'weight_oz': instance.weightOz,
      'width_in': instance.widthIn,
    };
