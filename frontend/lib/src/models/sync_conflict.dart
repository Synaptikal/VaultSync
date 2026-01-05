import 'package:json_annotation/json_annotation.dart';

part 'sync_conflict.g.dart';

/// Sync Conflict Model (PHASE 4 - Conflict Resolution)
///
/// Represents a CRDT conflict detected by the backend.
/// Contains both local and remote state for side-by-side comparison.
///
/// Backend Schema (from v0.2.0):
/// ```sql
/// CREATE TABLE Sync_Conflicts (
///   conflict_uuid TEXT PRIMARY KEY,
///   resource_type TEXT NOT NULL,
///   resource_uuid TEXT NOT NULL,
///   conflict_type TEXT NOT NULL,
///   resolution_status TEXT NOT NULL,
///   detected_at TEXT NOT NULL,
///   resolved_at TEXT,
///   resolved_by_user TEXT,
///   resolution_strategy TEXT
/// );
/// ```

@JsonSerializable()
class SyncConflict {
  @JsonKey(name: 'conflict_uuid')
  final String conflictUuid;

  @JsonKey(name: 'resource_type')
  final String resourceType; // 'Product', 'Inventory', 'Transaction'

  @JsonKey(name: 'resource_uuid')
  final String resourceUuid;

  @JsonKey(name: 'conflict_type')
  final String conflictType; // 'Concurrent_Mod', 'Oversold', 'PhysicalMiscount'

  @JsonKey(name: 'status')
  final String status; // 'Pending', 'Resolved'

  @JsonKey(name: 'detected_at')
  final String detectedAt;

  @JsonKey(name: 'remote_node_id')
  final String remoteNodeId;

  @JsonKey(name: 'remote_state')
  final Map<String, dynamic> remoteState;

  @JsonKey(name: 'local_state')
  final Map<String, dynamic> localState;

  SyncConflict({
    required this.conflictUuid,
    required this.resourceType,
    required this.resourceUuid,
    required this.conflictType,
    required this.status,
    required this.detectedAt,
    required this.remoteNodeId,
    required this.remoteState,
    required this.localState,
  });

  factory SyncConflict.fromJson(Map<String, dynamic> json) =>
      _$SyncConflictFromJson(json);

  Map<String, dynamic> toJson() => _$SyncConflictToJson(this);

  /// Get human-readable conflict type
  String get conflictTypeName {
    switch (conflictType) {
      case 'Concurrent_Mod':
        return 'Concurrent Modification';
      case 'Oversold':
        return 'Oversold Inventory';
      case 'PhysicalMiscount':
        return 'Physical Miscount';
      default:
        return conflictType;
    }
  }

  /// Get conflict severity (for UI color coding)
  ConflictSeverity get severity {
    switch (conflictType) {
      case 'Oversold':
        return ConflictSeverity.high;
      case 'PhysicalMiscount':
        return ConflictSeverity.medium;
      case 'Concurrent_Mod':
        return ConflictSeverity.low;
      default:
        return ConflictSeverity.medium;
    }
  }

  /// Check if local state was deleted
  bool get isLocalDeleted {
    return localState['status'] == 'deleted_or_missing';
  }

  /// Check if remote state was deleted
  bool get isRemoteDeleted {
    return remoteState.isEmpty || remoteState['deleted_at'] != null;
  }

  /// Get detected time as DateTime
  DateTime get detectedDateTime {
    return DateTime.parse(detectedAt);
  }

  /// Get time ago string
  String get timeAgo {
    final now = DateTime.now();
    final diff = now.difference(detectedDateTime);

    if (diff.inMinutes < 1) return 'Just now';
    if (diff.inMinutes < 60) return '${diff.inMinutes}m ago';
    if (diff.inHours < 24) return '${diff.inHours}h ago';
    if (diff.inDays < 7) return '${diff.inDays}d ago';
    return '${(diff.inDays / 7).floor()}w ago';
  }

  /// Get field differences between local and remote
  Map<String, FieldDifference> getFieldDifferences() {
    final differences = <String, FieldDifference>{};

    // Get all keys from both states
    final allKeys = {...localState.keys, ...remoteState.keys};

    for (final key in allKeys) {
      // Skip internal/metadata fields
      if (key == 'deleted_at' || key == 'status') continue;

      final localValue = localState[key];
      final remoteValue = remoteState[key];

      if (localValue != remoteValue) {
        differences[key] = FieldDifference(
          fieldName: key,
          localValue: localValue,
          remoteValue: remoteValue,
        );
      }
    }

    return differences;
  }
}

/// Field difference for side-by-side comparison
class FieldDifference {
  final String fieldName;
  final dynamic localValue;
  final dynamic remoteValue;

  FieldDifference({
    required this.fieldName,
    required this.localValue,
    required this.remoteValue,
  });

  /// Get human-readable field name
  String get displayName {
    // Convert snake_case to Title Case
    return fieldName
        .split('_')
        .map((word) => word[0].toUpperCase() + word.substring(1))
        .join(' ');
  }

  /// Format value for display
  String formatValue(dynamic value) {
    if (value == null) return 'Not set';
    if (value is bool) return value ? 'Yes' : 'No';
    if (value is Map) return 'Complex data';
    if (value is List) return '${value.length} items';
    return value.toString();
  }
}

/// Conflict severity levels
enum ConflictSeverity {
  low, // Can be auto-resolved, minor differences
  medium, // Requires attention, data quality issue
  high, // Critical, potential data loss
}

/// Resolution strategy
enum ResolutionStrategy {
  localWins, // Keep local version
  remoteWins, // Keep remote version
  manual, // Requires manual merge
}

extension ResolutionStrategyExtension on ResolutionStrategy {
  String get apiValue {
    switch (this) {
      case ResolutionStrategy.localWins:
        return 'LocalWins';
      case ResolutionStrategy.remoteWins:
        return 'RemoteWins';
      case ResolutionStrategy.manual:
        return 'Manual';
    }
  }

  String get displayName {
    switch (this) {
      case ResolutionStrategy.localWins:
        return 'Keep Local Version';
      case ResolutionStrategy.remoteWins:
        return 'Keep Remote Version';
      case ResolutionStrategy.manual:
        return 'Merge Manually';
    }
  }
}
