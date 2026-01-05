// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

@JsonEnum()
enum VariantType {
  @JsonValue('Normal')
  normal('Normal'),
  @JsonValue('Foil')
  foil('Foil'),
  @JsonValue('ReverseHolo')
  reverseHolo('ReverseHolo'),
  @JsonValue('FirstEdition')
  firstEdition('FirstEdition'),
  @JsonValue('Stamped')
  stamped('Stamped'),
  @JsonValue('Signed')
  signed('Signed'),
  @JsonValue('Graded')
  graded('Graded'),
  @JsonValue('Refractor')
  refractor('Refractor'),
  @JsonValue('Patch')
  patch('Patch'),
  @JsonValue('Auto')
  auto('Auto'),
  /// Default value for all unparsed values, allows backward compatibility when adding new values on the backend.
  $unknown(null);

  const VariantType(this.json);

  factory VariantType.fromJson(String json) => values.firstWhere(
        (e) => e.json == json,
        orElse: () => $unknown,
      );

  final String? json;

  String? toJson() => json;

  @override
  String toString() => json?.toString() ?? super.toString();
  /// Returns all defined enum values excluding the $unknown value.
  static List<VariantType> get $valuesDefined => values.where((value) => value != $unknown).toList();
}
