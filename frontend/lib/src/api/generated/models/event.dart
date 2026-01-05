// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

part 'event.g.dart';

@JsonSerializable()
class Event {
  const Event({
    required this.createdAt,
    required this.date,
    required this.entryFee,
    required this.eventType,
    required this.eventUuid,
    required this.name,
    this.maxParticipants,
  });
  
  factory Event.fromJson(Map<String, Object?> json) => _$EventFromJson(json);
  
  @JsonKey(name: 'created_at')
  final DateTime createdAt;
  final DateTime date;
  @JsonKey(name: 'entry_fee')
  final double entryFee;
  @JsonKey(name: 'event_type')
  final String eventType;
  @JsonKey(name: 'event_uuid')
  final String eventUuid;
  @JsonKey(name: 'max_participants')
  final int? maxParticipants;
  final String name;

  Map<String, Object?> toJson() => _$EventToJson(this);
}
