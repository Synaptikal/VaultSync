// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

@JsonEnum()
enum Category {
  @JsonValue('TCG')
  tcg('TCG'),
  @JsonValue('SportsCard')
  sportsCard('SportsCard'),
  @JsonValue('Comic')
  comic('Comic'),
  @JsonValue('Bobblehead')
  bobblehead('Bobblehead'),
  @JsonValue('Apparel')
  apparel('Apparel'),
  @JsonValue('Figure')
  figure('Figure'),
  @JsonValue('Accessory')
  accessory('Accessory'),
  @JsonValue('Other')
  other('Other'),
  /// Default value for all unparsed values, allows backward compatibility when adding new values on the backend.
  $unknown(null);

  const Category(this.json);

  factory Category.fromJson(String json) => values.firstWhere(
        (e) => e.json == json,
        orElse: () => $unknown,
      );

  final String? json;

  String? toJson() => json;

  @override
  String toString() => json?.toString() ?? super.toString();
  /// Returns all defined enum values excluding the $unknown value.
  static List<Category> get $valuesDefined => values.where((value) => value != $unknown).toList();
}
