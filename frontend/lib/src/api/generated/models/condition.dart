// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

@JsonEnum()
enum Condition {
  @JsonValue('NM')
  nm('NM'),
  @JsonValue('LP')
  lp('LP'),
  @JsonValue('MP')
  mp('MP'),
  @JsonValue('HP')
  hp('HP'),
  @JsonValue('DMG')
  dmg('DMG'),
  /// The name has been replaced because it contains a keyword. Original name: `New`.
  @JsonValue('New')
  valueNew('New'),
  @JsonValue('OpenBox')
  openBox('OpenBox'),
  @JsonValue('Used')
  used('Used'),
  @JsonValue('GemMint')
  gemMint('GemMint'),
  @JsonValue('Mint')
  mint('Mint'),
  @JsonValue('NearMintMint')
  nearMintMint('NearMintMint'),
  @JsonValue('VeryFine')
  veryFine('VeryFine'),
  @JsonValue('Fine')
  fine('Fine'),
  @JsonValue('Good')
  good('Good'),
  @JsonValue('Poor')
  poor('Poor'),
  /// Default value for all unparsed values, allows backward compatibility when adding new values on the backend.
  $unknown(null);

  const Condition(this.json);

  factory Condition.fromJson(String json) => values.firstWhere(
        (e) => e.json == json,
        orElse: () => $unknown,
      );

  final String? json;

  String? toJson() => json;

  @override
  String toString() => json?.toString() ?? super.toString();
  /// Returns all defined enum values excluding the $unknown value.
  static List<Condition> get $valuesDefined => values.where((value) => value != $unknown).toList();
}
