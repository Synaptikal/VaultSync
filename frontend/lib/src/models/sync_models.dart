import 'dart:convert';

enum RecordType {
  product,
  inventoryItem,
  priceInfo,
  transaction,
  customer,
}

enum SyncOperation {
  insert,
  update,
  delete,
}

class VectorTimestamp {
  final Map<String, int> entries;

  VectorTimestamp({required this.entries});

  factory VectorTimestamp.fromJson(Map<String, dynamic> json) {
    Map<String, int> entries = {};
    if (json['entries'] != null) {
      Map<String, dynamic> rawEntries = json['entries'];
      rawEntries.forEach((key, value) {
        if (value is int) {
          entries[key] = value;
        }
      });
    }
    return VectorTimestamp(entries: entries);
  }

  Map<String, dynamic> toJson() {
    return {
      'entries': entries,
    };
  }
}

class ChangeRecord {
  final String recordId;
  final RecordType recordType;
  final SyncOperation operation;
  final Map<String, dynamic> data;
  final VectorTimestamp vectorTimestamp;
  final DateTime timestamp;
  final int? sequenceNumber;

  ChangeRecord({
    required this.recordId,
    required this.recordType,
    required this.operation,
    required this.data,
    required this.vectorTimestamp,
    required this.timestamp,
    this.sequenceNumber,
  });

  factory ChangeRecord.fromJson(Map<String, dynamic> json) {
    return ChangeRecord(
      recordId: json['record_id'],
      recordType: RecordType.values.firstWhere(
        (e) =>
            e.name.toLowerCase() ==
            (json['record_type'] as String).toLowerCase(),
        orElse: () => RecordType.product,
      ),
      operation: SyncOperation.values.firstWhere(
        (e) =>
            e.name.toLowerCase() == (json['operation'] as String).toLowerCase(),
        orElse: () => SyncOperation.update,
      ),
      data: json['data'] is String ? jsonDecode(json['data']) : json['data'],
      vectorTimestamp: VectorTimestamp.fromJson(json['vector_timestamp']),
      timestamp: DateTime.parse(json['timestamp']),
      sequenceNumber: json['sequence_number'],
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'record_id': recordId,
      'record_type': recordType.name,
      'operation': operation.name,
      'data': data,
      'vector_timestamp': vectorTimestamp.toJson(),
      'timestamp': timestamp.toIso8601String(),
      'sequence_number': sequenceNumber,
    };
  }
}
