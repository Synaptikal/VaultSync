import 'package:sqflite/sqflite.dart';
import 'package:path/path.dart';

/// Database Service (PHASE 2 - Local Storage)
///
/// Manages SQLite database initialization and migrations.
/// Provides a singleton instance for consistent access.
///
/// Usage:
/// ```dart
/// final db = await DatabaseService().database;
/// await db.query('products');
/// ```

class DatabaseService {
  static final DatabaseService _instance = DatabaseService._internal();
  static Database? _database;

  factory DatabaseService() => _instance;

  DatabaseService._internal();

  Future<Database> get database async {
    if (_database != null) return _database!;
    _database = await _initDatabase();
    return _database!;
  }

  Future<Database> _initDatabase() async {
    final dbPath = await getDatabasesPath();
    final path = join(dbPath, 'vaultsync.db');

    return await openDatabase(
      path,
      version: 4, // Increment when schema changes (v4: inventory columns)
      onCreate: _onCreate,
      onUpgrade: _onUpgrade,
    );
  }

  /// Create initial schema
  Future<void> _onCreate(Database db, int version) async {
    // Products table
    await db.execute('''
      CREATE TABLE products (
        product_uuid TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        category TEXT,
        set_code TEXT,
        collector_number TEXT,
        barcode TEXT,
        release_year INTEGER,
        metadata TEXT,
        is_synced INTEGER DEFAULT 0,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        deleted_at TEXT
      )
    ''');

    await db.execute('CREATE INDEX idx_products_name ON products(name)');
    await db
        .execute('CREATE INDEX idx_products_category ON products(category)');
    await db.execute('CREATE INDEX idx_products_barcode ON products(barcode)');
    await db.execute('CREATE INDEX idx_products_synced ON products(is_synced)');

    // Inventory table
    await db.execute('''
      CREATE TABLE inventory (
        inventory_uuid TEXT PRIMARY KEY,
        product_uuid TEXT NOT NULL,
        condition TEXT NOT NULL,
        quantity_on_hand INTEGER NOT NULL,
        location_tag TEXT,
        specific_price REAL,
        is_synced INTEGER DEFAULT 0,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        deleted_at TEXT,
        FOREIGN KEY (product_uuid) REFERENCES products(product_uuid)
      )
    ''');

    await db.execute(
        'CREATE INDEX idx_inventory_product ON inventory(product_uuid)');
    await db.execute(
        'CREATE INDEX idx_inventory_location ON inventory(location_tag)');

    // Sync queue table (PHASE 3)
    await db.execute('''
      CREATE TABLE sync_queue (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        entity_type TEXT NOT NULL,
        entity_uuid TEXT NOT NULL,
        operation TEXT NOT NULL,
        payload TEXT NOT NULL,
        attempts INTEGER DEFAULT 0,
        last_error TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
      )
    ''');

    await db.execute(
        'CREATE INDEX idx_sync_queue_entity ON sync_queue(entity_type, entity_uuid)');

    // Sync conflicts table (PHASE 4)
    await db.execute('''
      CREATE TABLE sync_conflicts (
        conflict_uuid TEXT PRIMARY KEY,
        resource_type TEXT NOT NULL,
        resource_uuid TEXT NOT NULL,
        local_state TEXT NOT NULL,
        remote_state TEXT NOT NULL,
        detected_at TEXT NOT NULL,
        resolved_at TEXT
      )
    ''');
  }

  /// Handle database upgrades
  Future<void> _onUpgrade(Database db, int oldVersion, int newVersion) async {
    if (oldVersion < 2) {
      // Migration to version 2
      await db.execute('ALTER TABLE products ADD COLUMN barcode TEXT');
      await db
          .execute('CREATE INDEX idx_products_barcode ON products(barcode)');
    }

    if (oldVersion < 3) {
      // Migration to version 3 - Add sync queue
      await db.execute('''
        CREATE TABLE IF NOT EXISTS sync_queue (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          entity_type TEXT NOT NULL,
          entity_uuid TEXT NOT NULL,
          operation TEXT NOT NULL,
          payload TEXT NOT NULL,
          attempts INTEGER DEFAULT 0,
          last_error TEXT,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        )
      ''');
    }

    if (oldVersion < 4) {
      // Migration to version 4 - Extend inventory table (TASK-AUD-001e)
      // Add missing columns for full InventoryItem support
      await db.execute('ALTER TABLE inventory ADD COLUMN bin_location TEXT');
      await db.execute('ALTER TABLE inventory ADD COLUMN cost_basis REAL');
      await db.execute(
          'ALTER TABLE inventory ADD COLUMN min_stock_level INTEGER DEFAULT 0');
      await db
          .execute('ALTER TABLE inventory ADD COLUMN max_stock_level INTEGER');
      await db
          .execute('ALTER TABLE inventory ADD COLUMN reorder_point INTEGER');
      await db.execute('ALTER TABLE inventory ADD COLUMN supplier_uuid TEXT');
      await db.execute('ALTER TABLE inventory ADD COLUMN received_date TEXT');
      await db
          .execute('ALTER TABLE inventory ADD COLUMN last_counted_date TEXT');
      await db.execute('ALTER TABLE inventory ADD COLUMN last_sold_date TEXT');
      await db.execute('ALTER TABLE inventory ADD COLUMN variant_type TEXT');
      await db
          .execute('ALTER TABLE inventory ADD COLUMN serialized_details TEXT');
    }
  }

  /// Close database connection
  Future<void> close() async {
    if (_database != null) {
      await _database!.close();
      _database = null;
    }
  }

  /// Clear all data (use carefully - for logout/reset only)
  Future<void> clearAllData() async {
    final db = await database;
    await db.delete('products');
    await db.delete('inventory');
    await db.delete('sync_queue');
    await db.delete('sync_conflicts');
  }
}
