// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

part 'vector_timestamp.g.dart';

@JsonSerializable()
class VectorTimestamp {
  const VectorTimestamp({
    required this.entries,
  });
  
  factory VectorTimestamp.fromJson(Map<String, Object?> json) => _$VectorTimestampFromJson(json);
  
  final Map<String, int> entries;

  Map<String, Object?> toJson() => _$VectorTimestampToJson(this);
}
