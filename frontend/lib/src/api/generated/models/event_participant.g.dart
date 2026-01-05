// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'event_participant.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

EventParticipant _$EventParticipantFromJson(Map<String, dynamic> json) =>
    EventParticipant(
      createdAt: DateTime.parse(json['created_at'] as String),
      eventUuid: json['event_uuid'] as String,
      name: json['name'] as String,
      paid: json['paid'] as bool,
      participantUuid: json['participant_uuid'] as String,
      customerUuid: json['customer_uuid'] as String?,
      placement: (json['placement'] as num?)?.toInt(),
    );

Map<String, dynamic> _$EventParticipantToJson(EventParticipant instance) =>
    <String, dynamic>{
      'created_at': instance.createdAt.toIso8601String(),
      'customer_uuid': instance.customerUuid,
      'event_uuid': instance.eventUuid,
      'name': instance.name,
      'paid': instance.paid,
      'participant_uuid': instance.participantUuid,
      'placement': instance.placement,
    };
