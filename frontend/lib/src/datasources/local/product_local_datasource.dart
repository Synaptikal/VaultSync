import 'package:sqflite/sqflite.dart';
import '../../api/generated/export.dart';
import '../../services/database_service.dart';

/// Local Data Source for Products (PHASE 2)
///
/// Handles all SQLite operations for products.
/// Provides offline-first caching and persistence.
///
/// Responsibilities:
/// - CRUD operations on local SQLite database
/// - Cache management
/// - Sync status tracking (is_synced flag)
/// - Query optimization
///
/// Schema:
/// ```sql
/// CREATE TABLE products (
///   product_uuid TEXT PRIMARY KEY,
///   name TEXT NOT NULL,
///   category TEXT,
///   set_code TEXT,
///   collector_number TEXT,
///   metadata TEXT,  -- JSON
///   is_synced INTEGER DEFAULT 0,
///   created_at TEXT,
///   updated_at TEXT,
///   deleted_at TEXT
/// );
/// ```

class ProductLocalDataSource {
  final DatabaseService dbService;

  ProductLocalDataSource({required this.dbService});

  /// Get all products from local DB
  Future<List<Product>> getAll({int? limit, int? offset}) async {
    final db = await dbService.database;
    final List<Map<String, dynamic>> maps = await db.query(
      'products',
      where: 'deleted_at IS NULL',
      orderBy: 'name ASC',
      limit: limit,
      offset: offset,
    );
    return maps.map((map) => _fromMap(map)).toList();
  }

  /// Get a single product by ID
  Future<Product?> getById(String productUuid) async {
    final db = await dbService.database;
    final List<Map<String, dynamic>> maps = await db.query(
      'products',
      where: 'product_uuid = ? AND deleted_at IS NULL',
      whereArgs: [productUuid],
      limit: 1,
    );
    if (maps.isEmpty) return null;
    return _fromMap(maps.first);
  }

  /// Insert a new product
  Future<void> insert(Product product) async {
    final db = await dbService.database;
    await db.insert(
      'products',
      _toMap(product, isSynced: false),
      conflictAlgorithm: ConflictAlgorithm.replace,
    );
  }

  /// Update existing product
  Future<void> update(Product product) async {
    final db = await dbService.database;
    await db.update(
      'products',
      _toMap(product, isSynced: false),
      where: 'product_uuid = ?',
      whereArgs: [product.productUuid],
    );
  }

  /// Soft delete (mark as deleted)
  Future<void> delete(String productUuid) async {
    final db = await dbService.database;
    await db.update(
      'products',
      {
        'deleted_at': DateTime.now().toIso8601String(),
        'is_synced': 0, // Needs to sync deletion
      },
      where: 'product_uuid = ?',
      whereArgs: [productUuid],
    );
  }

  /// Hard delete (permanent removal)
  Future<void> hardDelete(String productUuid) async {
    final db = await dbService.database;
    await db.delete(
      'products',
      where: 'product_uuid = ?',
      whereArgs: [productUuid],
    );
  }

  /// Search products by name
  Future<List<Product>> search(String query) async {
    final db = await dbService.database;
    final List<Map<String, dynamic>> maps = await db.query(
      'products',
      where: 'name LIKE ? AND deleted_at IS NULL',
      whereArgs: ['%$query%'],
      orderBy: 'name ASC',
    );
    return maps.map((map) => _fromMap(map)).toList();
  }

  /// Get products by category
  Future<List<Product>> getByCategory(String category) async {
    final db = await dbService.database;
    final List<Map<String, dynamic>> maps = await db.query(
      'products',
      where: 'category = ? AND deleted_at IS NULL',
      whereArgs: [category],
      orderBy: 'name ASC',
    );
    return maps.map((map) => _fromMap(map)).toList();
  }

  /// Get products that haven't been synced
  Future<List<Product>> getUnsynced() async {
    final db = await dbService.database;
    final List<Map<String, dynamic>> maps = await db.query(
      'products',
      where: 'is_synced = 0',
      orderBy: 'updated_at DESC',
    );
    return maps.map((map) => _fromMap(map)).toList();
  }

  /// Mark product as synced
  Future<void> markSynced(String productUuid) async {
    final db = await dbService.database;
    await db.update(
      'products',
      {'is_synced': 1},
      where: 'product_uuid = ?',
      whereArgs: [productUuid],
    );
  }

  /// Clear all products (use carefully)
  Future<void> clearAll() async {
    final db = await dbService.database;
    await db.delete('products');
  }

  /// Get total count
  Future<int> count() async {
    final db = await dbService.database;
    final result = await db.rawQuery(
      'SELECT COUNT(*) as count FROM products WHERE deleted_at IS NULL',
    );
    return Sqflite.firstIntValue(result) ?? 0;
  }

  /// Batch insert (for initial sync)
  Future<void> insertBatch(List<Product> products) async {
    final db = await dbService.database;
    final batch = db.batch();
    for (final product in products) {
      batch.insert(
        'products',
        _toMap(product, isSynced: true), // Coming from server = synced
        conflictAlgorithm: ConflictAlgorithm.replace,
      );
    }
    await batch.commit(noResult: true);
  }

  /// Get UUIDs of unsynced products to prevent overwriting
  Future<Set<String>> getDirtyUuids() async {
    final db = await dbService.database;
    final List<Map<String, dynamic>> maps = await db.query(
      'products',
      columns: ['product_uuid'],
      where: 'is_synced = 0',
    );
    return maps.map((m) => m['product_uuid'] as String).toSet();
  }

  // === Helper Methods ===

  /// Convert Product to Map for SQLite
  Map<String, dynamic> _toMap(Product product, {required bool isSynced}) {
    return {
      'product_uuid': product.productUuid,
      'name': product.name,
      'category': product.category.toString().split('.').last,
      'set_code': product.setCode,
      'collector_number': product.collectorNumber,
      'barcode': product.barcode,
      'release_year': product.releaseYear,
      'metadata': product.metadata != null ? product.metadata.toString() : null,
      'is_synced': isSynced ? 1 : 0,
      'created_at': DateTime.now().toIso8601String(),
      'updated_at': DateTime.now().toIso8601String(),
      'deleted_at': product.deletedAt?.toIso8601String(),
    };
  }

  /// Convert Map to Product
  Product _fromMap(Map<String, dynamic> map) {
    return Product(
      productUuid: map['product_uuid'] as String,
      name: map['name'] as String,
      category: _parseCategory(map['category'] as String?),
      setCode: map['set_code'] as String?,
      collectorNumber: map['collector_number'] as String?,
      barcode: map['barcode'] as String?,
      releaseYear: map['release_year'] as int?,
      metadata: map['metadata'] != null
          ? map['metadata'] as Map<String,
              dynamic> // Warning: needs jsonDecode if storing as string
          : null,
      deletedAt: map['deleted_at'] != null
          ? DateTime.parse(map['deleted_at'] as String)
          : null,
    );
  }

  /// Parse category enum from string
  Category _parseCategory(String? categoryStr) {
    if (categoryStr == null) return Category.other;
    return Category.values.firstWhere(
      (e) => e.toString().split('.').last == categoryStr,
      orElse: () => Category.other,
    );
  }
}
