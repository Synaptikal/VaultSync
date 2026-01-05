// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, unused_import, invalid_annotation_target, unnecessary_import

import 'package:json_annotation/json_annotation.dart';

import 'record_type.dart';
import 'sync_operation.dart';
import 'vector_timestamp.dart';

part 'change_record.g.dart';

@JsonSerializable()
class ChangeRecord {
  const ChangeRecord({
    required this.data,
    required this.operation,
    required this.recordId,
    required this.recordType,
    required this.timestamp,
    required this.vectorTimestamp,
    this.checksum,
    this.sequenceNumber,
  });
  
  factory ChangeRecord.fromJson(Map<String, Object?> json) => _$ChangeRecordFromJson(json);
  
  /// TASK-123: Checksum for data integrity verification
  final String? checksum;
  final dynamic data;
  final SyncOperation operation;
  @JsonKey(name: 'record_id')
  final String recordId;
  @JsonKey(name: 'record_type')
  final RecordType recordType;

  /// Server-local sequence number for delta sync
  @JsonKey(name: 'sequence_number')
  final int? sequenceNumber;
  final DateTime timestamp;
  @JsonKey(name: 'vector_timestamp')
  final VectorTimestamp vectorTimestamp;

  Map<String, Object?> toJson() => _$ChangeRecordToJson(this);
}
