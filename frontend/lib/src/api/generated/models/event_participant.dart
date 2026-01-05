// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

part 'event_participant.g.dart';

@JsonSerializable()
class EventParticipant {
  const EventParticipant({
    required this.createdAt,
    required this.eventUuid,
    required this.name,
    required this.paid,
    required this.participantUuid,
    this.customerUuid,
    this.placement,
  });
  
  factory EventParticipant.fromJson(Map<String, Object?> json) => _$EventParticipantFromJson(json);
  
  @JsonKey(name: 'created_at')
  final DateTime createdAt;
  @JsonKey(name: 'customer_uuid')
  final String? customerUuid;
  @JsonKey(name: 'event_uuid')
  final String eventUuid;
  final String name;
  final bool paid;
  @JsonKey(name: 'participant_uuid')
  final String participantUuid;
  final int? placement;

  Map<String, Object?> toJson() => _$EventParticipantToJson(this);
}
