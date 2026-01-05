// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

import 'category.dart';

part 'product.g.dart';

@JsonSerializable()
class Product {
  const Product({
    required this.category,
    required this.metadata,
    required this.name,
    required this.productUuid,
    this.barcode,
    this.collectorNumber,
    this.deletedAt,
    this.heightIn,
    this.isbn,
    this.lengthIn,
    this.manufacturer,
    this.msrp,
    this.releaseYear,
    this.setCode,
    this.upc,
    this.weightOz,
    this.widthIn,
  });
  
  factory Product.fromJson(Map<String, Object?> json) => _$ProductFromJson(json);
  
  final String? barcode;
  final Category category;
  @JsonKey(name: 'collector_number')
  final String? collectorNumber;
  @JsonKey(name: 'deleted_at')
  final DateTime? deletedAt;
  @JsonKey(name: 'height_in')
  final double? heightIn;
  final String? isbn;
  @JsonKey(name: 'length_in')
  final double? lengthIn;
  final String? manufacturer;
  final dynamic metadata;
  final double? msrp;
  final String name;
  @JsonKey(name: 'product_uuid')
  final String productUuid;
  @JsonKey(name: 'release_year')
  final int? releaseYear;
  @JsonKey(name: 'set_code')
  final String? setCode;
  final String? upc;
  @JsonKey(name: 'weight_oz')
  final double? weightOz;
  @JsonKey(name: 'width_in')
  final double? widthIn;

  Map<String, Object?> toJson() => _$ProductToJson(this);
}
