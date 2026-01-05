// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'event.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

Event _$EventFromJson(Map<String, dynamic> json) => Event(
      createdAt: DateTime.parse(json['created_at'] as String),
      date: DateTime.parse(json['date'] as String),
      entryFee: (json['entry_fee'] as num).toDouble(),
      eventType: json['event_type'] as String,
      eventUuid: json['event_uuid'] as String,
      name: json['name'] as String,
      maxParticipants: (json['max_participants'] as num?)?.toInt(),
    );

Map<String, dynamic> _$EventToJson(Event instance) => <String, dynamic>{
      'created_at': instance.createdAt.toIso8601String(),
      'date': instance.date.toIso8601String(),
      'entry_fee': instance.entryFee,
      'event_type': instance.eventType,
      'event_uuid': instance.eventUuid,
      'max_participants': instance.maxParticipants,
      'name': instance.name,
    };
