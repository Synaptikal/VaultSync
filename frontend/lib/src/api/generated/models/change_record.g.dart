// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'change_record.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

ChangeRecord _$ChangeRecordFromJson(Map<String, dynamic> json) => ChangeRecord(
      data: json['data'],
      operation: SyncOperation.fromJson(json['operation'] as String),
      recordId: json['record_id'] as String,
      recordType: RecordType.fromJson(json['record_type'] as String),
      timestamp: DateTime.parse(json['timestamp'] as String),
      vectorTimestamp: VectorTimestamp.fromJson(
          json['vector_timestamp'] as Map<String, dynamic>),
      checksum: json['checksum'] as String?,
      sequenceNumber: (json['sequence_number'] as num?)?.toInt(),
    );

Map<String, dynamic> _$ChangeRecordToJson(ChangeRecord instance) =>
    <String, dynamic>{
      'checksum': instance.checksum,
      'data': instance.data,
      'operation': instance.operation,
      'record_id': instance.recordId,
      'record_type': instance.recordType,
      'sequence_number': instance.sequenceNumber,
      'timestamp': instance.timestamp.toIso8601String(),
      'vector_timestamp': instance.vectorTimestamp,
    };
