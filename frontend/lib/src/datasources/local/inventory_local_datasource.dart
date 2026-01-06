import 'dart:convert';

import 'package:sqflite/sqflite.dart';
import '../../api/generated/export.dart';
import '../../services/database_service.dart';

/// Local Data Source for Inventory Items (TASK-AUD-001e)
///
/// Handles all SQLite operations for inventory items.
/// Provides offline-first caching and persistence.
///
/// Responsibilities:
/// - CRUD operations on local SQLite database
/// - Cache management
/// - Sync status tracking (is_synced flag)
/// - Query optimization

class InventoryLocalDataSource {
  final DatabaseService dbService;

  InventoryLocalDataSource({required this.dbService});

  /// Get all inventory items from local DB
  Future<List<InventoryItem>> getAll({int? limit, int? offset}) async {
    final db = await dbService.database;
    final List<Map<String, dynamic>> maps = await db.query(
      'inventory',
      where: 'deleted_at IS NULL',
      orderBy: 'updated_at DESC',
      limit: limit,
      offset: offset,
    );
    return maps.map((map) => _fromMap(map)).toList();
  }

  /// Get a single inventory item by ID
  Future<InventoryItem?> getById(String inventoryUuid) async {
    final db = await dbService.database;
    final List<Map<String, dynamic>> maps = await db.query(
      'inventory',
      where: 'inventory_uuid = ? AND deleted_at IS NULL',
      whereArgs: [inventoryUuid],
      limit: 1,
    );
    if (maps.isEmpty) return null;
    return _fromMap(maps.first);
  }

  /// Get inventory items by product UUID
  Future<List<InventoryItem>> getByProductUuid(String productUuid) async {
    final db = await dbService.database;
    final List<Map<String, dynamic>> maps = await db.query(
      'inventory',
      where: 'product_uuid = ? AND deleted_at IS NULL',
      whereArgs: [productUuid],
      orderBy: 'condition ASC',
    );
    return maps.map((map) => _fromMap(map)).toList();
  }

  /// Insert a new inventory item
  Future<void> insert(InventoryItem item) async {
    final db = await dbService.database;
    await db.insert(
      'inventory',
      _toMap(item, isSynced: false),
      conflictAlgorithm: ConflictAlgorithm.replace,
    );
  }

  /// Update existing inventory item
  Future<void> update(InventoryItem item) async {
    final db = await dbService.database;
    await db.update(
      'inventory',
      _toMap(item, isSynced: false),
      where: 'inventory_uuid = ?',
      whereArgs: [item.inventoryUuid],
    );
  }

  /// Soft delete (mark as deleted)
  Future<void> delete(String inventoryUuid) async {
    final db = await dbService.database;
    await db.update(
      'inventory',
      {
        'deleted_at': DateTime.now().toIso8601String(),
        'is_synced': 0,
      },
      where: 'inventory_uuid = ?',
      whereArgs: [inventoryUuid],
    );
  }

  /// Hard delete (permanent removal)
  Future<void> hardDelete(String inventoryUuid) async {
    final db = await dbService.database;
    await db.delete(
      'inventory',
      where: 'inventory_uuid = ?',
      whereArgs: [inventoryUuid],
    );
  }

  /// Get inventory items that haven't been synced
  Future<List<InventoryItem>> getUnsynced() async {
    final db = await dbService.database;
    final List<Map<String, dynamic>> maps = await db.query(
      'inventory',
      where: 'is_synced = 0',
      orderBy: 'updated_at DESC',
    );
    return maps.map((map) => _fromMap(map)).toList();
  }

  /// Mark inventory item as synced
  Future<void> markSynced(String inventoryUuid) async {
    final db = await dbService.database;
    await db.update(
      'inventory',
      {'is_synced': 1},
      where: 'inventory_uuid = ?',
      whereArgs: [inventoryUuid],
    );
  }

  /// Get UUIDs of unsynced items to prevent overwriting
  Future<Set<String>> getDirtyUuids() async {
    final db = await dbService.database;
    final List<Map<String, dynamic>> maps = await db.query(
      'inventory',
      columns: ['inventory_uuid'],
      where: 'is_synced = 0',
    );
    return maps.map((m) => m['inventory_uuid'] as String).toSet();
  }

  /// Batch insert (for initial sync)
  Future<void> insertBatch(List<InventoryItem> items) async {
    final db = await dbService.database;
    final batch = db.batch();
    for (final item in items) {
      batch.insert(
        'inventory',
        _toMap(item, isSynced: true),
        conflictAlgorithm: ConflictAlgorithm.replace,
      );
    }
    await batch.commit(noResult: true);
  }

  /// Get total count
  Future<int> count() async {
    final db = await dbService.database;
    final result = await db.rawQuery(
      'SELECT COUNT(*) as count FROM inventory WHERE deleted_at IS NULL',
    );
    return Sqflite.firstIntValue(result) ?? 0;
  }

  /// Clear all inventory (use carefully)
  Future<void> clearAll() async {
    final db = await dbService.database;
    await db.delete('inventory');
  }

  /// Get low stock items
  Future<List<InventoryItem>> getLowStock({int threshold = 3}) async {
    final db = await dbService.database;
    final List<Map<String, dynamic>> maps = await db.query(
      'inventory',
      where: 'quantity_on_hand <= ? AND deleted_at IS NULL',
      whereArgs: [threshold],
      orderBy: 'quantity_on_hand ASC',
    );
    return maps.map((map) => _fromMap(map)).toList();
  }

  // === Helper Methods ===

  /// Convert InventoryItem to Map for SQLite
  Map<String, dynamic> _toMap(InventoryItem item, {required bool isSynced}) {
    return {
      'inventory_uuid': item.inventoryUuid,
      'product_uuid': item.productUuid,
      'condition': item.condition.name,
      'quantity_on_hand': item.quantityOnHand,
      'location_tag': item.locationTag,
      'bin_location': item.binLocation,
      'specific_price': item.specificPrice,
      'cost_basis': item.costBasis,
      'min_stock_level': item.minStockLevel,
      'max_stock_level': item.maxStockLevel,
      'reorder_point': item.reorderPoint,
      'supplier_uuid': item.supplierUuid,
      'received_date': item.receivedDate?.toIso8601String(),
      'last_counted_date': item.lastCountedDate?.toIso8601String(),
      'last_sold_date': item.lastSoldDate?.toIso8601String(),
      'variant_type': item.variantType?.name,
      'serialized_details': item.serializedDetails != null
          ? jsonEncode(item.serializedDetails)
          : null,
      'is_synced': isSynced ? 1 : 0,
      'created_at': DateTime.now().toIso8601String(),
      'updated_at': DateTime.now().toIso8601String(),
      'deleted_at': item.deletedAt?.toIso8601String(),
    };
  }

  /// Convert Map to InventoryItem
  InventoryItem _fromMap(Map<String, dynamic> map) {
    return InventoryItem(
      inventoryUuid: map['inventory_uuid'] as String,
      productUuid: map['product_uuid'] as String,
      condition: _parseCondition(map['condition'] as String?),
      quantityOnHand: map['quantity_on_hand'] as int,
      locationTag: map['location_tag'] as String? ?? '',
      binLocation: map['bin_location'] as String?,
      specificPrice: map['specific_price'] as double?,
      costBasis: map['cost_basis'] as double?,
      minStockLevel: map['min_stock_level'] as int? ?? 0,
      maxStockLevel: map['max_stock_level'] as int?,
      reorderPoint: map['reorder_point'] as int?,
      supplierUuid: map['supplier_uuid'] as String?,
      receivedDate: map['received_date'] != null
          ? DateTime.parse(map['received_date'] as String)
          : null,
      lastCountedDate: map['last_counted_date'] != null
          ? DateTime.parse(map['last_counted_date'] as String)
          : null,
      lastSoldDate: map['last_sold_date'] != null
          ? DateTime.parse(map['last_sold_date'] as String)
          : null,
      variantType: _parseVariantType(map['variant_type'] as String?),
      serializedDetails: map['serialized_details'] != null
          ? jsonDecode(map['serialized_details'] as String)
          : null,
      deletedAt: map['deleted_at'] != null
          ? DateTime.parse(map['deleted_at'] as String)
          : null,
    );
  }

  /// Parse condition enum from string
  Condition _parseCondition(String? conditionStr) {
    if (conditionStr == null) return Condition.nm;
    return Condition.values.firstWhere(
      (e) => e.name == conditionStr,
      orElse: () => Condition.nm,
    );
  }

  /// Parse variant type enum from string
  VariantType? _parseVariantType(String? variantStr) {
    if (variantStr == null) return null;
    return VariantType.values.firstWhere(
      (e) => e.name == variantStr,
      orElse: () => VariantType.normal,
    );
  }
}
