// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'sync_conflict.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

SyncConflict _$SyncConflictFromJson(Map<String, dynamic> json) => SyncConflict(
      conflictUuid: json['conflict_uuid'] as String,
      resourceType: json['resource_type'] as String,
      resourceUuid: json['resource_uuid'] as String,
      conflictType: json['conflict_type'] as String,
      status: json['status'] as String,
      detectedAt: json['detected_at'] as String,
      remoteNodeId: json['remote_node_id'] as String,
      remoteState: json['remote_state'] as Map<String, dynamic>,
      localState: json['local_state'] as Map<String, dynamic>,
    );

Map<String, dynamic> _$SyncConflictToJson(SyncConflict instance) =>
    <String, dynamic>{
      'conflict_uuid': instance.conflictUuid,
      'resource_type': instance.resourceType,
      'resource_uuid': instance.resourceUuid,
      'conflict_type': instance.conflictType,
      'status': instance.status,
      'detected_at': instance.detectedAt,
      'remote_node_id': instance.remoteNodeId,
      'remote_state': instance.remoteState,
      'local_state': instance.localState,
    };
