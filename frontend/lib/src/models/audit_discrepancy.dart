import 'package:json_annotation/json_annotation.dart';

part 'audit_discrepancy.g.dart';

/// Audit Discrepancy Model (PHASE 5 - Inventory Audit)
///
/// Represents a variance between expected and actual inventory.
/// Returned from the backend's blind count API.
///
/// Backend Schema:
/// ```sql
/// CREATE TABLE Inventory_Conflicts (
///   conflict_uuid TEXT PRIMARY KEY,
///   product_uuid TEXT NOT NULL,
///   conflict_type TEXT NOT NULL,
///   expected_quantity INTEGER NOT NULL,
///   actual_quantity INTEGER NOT NULL,
///   resolution_status TEXT NOT NULL
/// );
/// ```

@JsonSerializable()
class AuditDiscrepancy {
  @JsonKey(name: 'product_uuid')
  final String productUuid;

  @JsonKey(name: 'product_name')
  final String? productName;

  @JsonKey(name: 'condition')
  final String condition;

  @JsonKey(name: 'expected_quantity')
  final int expectedQuantity;

  @JsonKey(name: 'actual_quantity')
  final int actualQuantity;

  @JsonKey(name: 'variance')
  final int variance;

  @JsonKey(name: 'location_tag')
  final String? locationTag;

  AuditDiscrepancy({
    required this.productUuid,
    this.productName,
    required this.condition,
    required this.expectedQuantity,
    required this.actualQuantity,
    required this.variance,
    this.locationTag,
  });

  factory AuditDiscrepancy.fromJson(Map<String, dynamic> json) =>
      _$AuditDiscrepancyFromJson(json);

  Map<String, dynamic> toJson() => _$AuditDiscrepancyToJson(this);

  /// Get variance severity
  DiscrepancySeverity get severity {
    final percentVariance = (variance.abs() / expectedQuantity * 100).round();

    if (percentVariance >= 50) return DiscrepancySeverity.high;
    if (percentVariance >= 20) return DiscrepancySeverity.medium;
    return DiscrepancySeverity.low;
  }

  /// Check if this is an overage (more than expected)
  bool get isOverage => variance > 0;

  /// Check if this is a shortage (less than expected)
  bool get isShortage => variance < 0;

  /// Get human-readable variance text
  String get varianceText {
    if (variance > 0) return '+$variance (Overage)';
    if (variance < 0) return '$variance (Shortage)';
    return 'No variance';
  }

  /// Get variance percentage
  double get variancePercentage {
    if (expectedQuantity == 0) return 0;
    return (variance.abs() / expectedQuantity * 100);
  }

  /// Get display name
  String get displayName {
    return productName ?? 'Product $productUuid';
  }
}

/// Discrepancy severity levels
enum DiscrepancySeverity {
  low, // < 20% variance
  medium, // 20-50% variance
  high, // > 50% variance
}

/// Blind count item (for scanning)
class BlindCountItem {
  final String productUuid;
  final String productName;
  final String condition;
  int quantity;

  BlindCountItem({
    required this.productUuid,
    required this.productName,
    required this.condition,
    this.quantity = 1,
  });

  Map<String, dynamic> toJson() {
    return {
      'product_uuid': productUuid,
      'condition': condition,
      'quantity': quantity,
    };
  }
}

/// Audit session (tracks a blind count)
class AuditSession {
  final String sessionId;
  final String locationTag;
  final DateTime startedAt;
  final List<BlindCountItem> items;
  List<AuditDiscrepancy>? discrepancies;
  DateTime? completedAt;

  AuditSession({
    required this.sessionId,
    required this.locationTag,
    DateTime? startedAt,
    List<BlindCountItem>? items,
    this.discrepancies,
    this.completedAt,
  })  : startedAt = startedAt ?? DateTime.now(),
        items = items ?? [];

  /// Add scanned item
  void addItem(BlindCountItem item) {
    // Check if already scanned
    final existing = items.firstWhere(
      (i) => i.productUuid == item.productUuid && i.condition == item.condition,
      orElse: () => item,
    );

    if (existing == item) {
      // New item
      items.add(item);
    } else {
      // Increment existing
      existing.quantity++;
    }
  }

  /// Get total items scanned
  int get totalItemsScanned {
    return items.fold(0, (sum, item) => sum + item.quantity);
  }

  /// Check if session is complete
  bool get isComplete => completedAt != null;

  /// Duration of audit
  Duration get duration {
    final end = completedAt ?? DateTime.now();
    return end.difference(startedAt);
  }

  /// Get duration as string
  String get durationText {
    final minutes = duration.inMinutes;
    final seconds = duration.inSeconds % 60;
    return '${minutes}m ${seconds}s';
  }
}
