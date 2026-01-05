import 'package:sqflite_common_ffi/sqflite_ffi.dart';
import 'package:path/path.dart';
import 'dart:io';
import 'dart:convert';
import '../api/generated/models/product.dart';
import '../api/generated/models/customer.dart';
import '../api/generated/models/inventory_item.dart';
import '../api/generated/models/category.dart';
import '../api/generated/models/condition.dart';
import '../api/generated/models/variant_type.dart';
import '../models/sync_models.dart';
import 'package:flutter/foundation.dart' hide Category;

class LocalStorageService {
  static final LocalStorageService _instance = LocalStorageService._internal();
  static Database? _database;

  factory LocalStorageService() => _instance;

  LocalStorageService._internal();

  Future<Database> get database async {
    if (_database != null) return _database!;
    _database = await _initDatabase();
    return _database!;
  }

  Future<Database> _initDatabase() async {
    if (Platform.isWindows || Platform.isLinux) {
      sqfliteFfiInit();
      databaseFactory = databaseFactoryFfi;
    }

    String path = join(await getDatabasesPath(), 'vaultsync_local.db');

    return await openDatabase(
      path,
      version: 3,
      onCreate: _onCreate,
      onUpgrade: _onUpgrade,
    );
  }

  Future<void> _onCreate(Database db, int version) async {
    await db.execute('''
      CREATE TABLE products(
        product_uuid TEXT PRIMARY KEY,
        name TEXT,
        category TEXT,
        set_code TEXT,
        collector_number TEXT,
        metadata TEXT,
        is_synced INTEGER DEFAULT 1
      )
    ''');

    await db.execute('''
      CREATE TABLE customers(
        customer_uuid TEXT PRIMARY KEY,
        name TEXT,
        email TEXT,
        phone TEXT,
        store_credit REAL,
        created_at TEXT,
        is_synced INTEGER DEFAULT 1
      )
    ''');

    await db.execute('''
      CREATE TABLE local_inventory(
        inventory_uuid TEXT PRIMARY KEY,
        product_uuid TEXT,
        variant_type TEXT,
        condition TEXT,
        quantity_on_hand INTEGER,
        location_tag TEXT,
        is_synced INTEGER DEFAULT 1
      )
    ''');

    await db.execute('''
      CREATE TABLE transactions(
        transaction_uuid TEXT PRIMARY KEY,
        customer_uuid TEXT,
        timestamp TEXT,
        transaction_type TEXT,
        is_synced INTEGER DEFAULT 1
      )
    ''');

    await db.execute('''
      CREATE TABLE transaction_items(
        item_uuid TEXT PRIMARY KEY,
        transaction_uuid TEXT,
        product_uuid TEXT,
        quantity INTEGER,
        unit_price REAL,
        condition TEXT
      )
    ''');

    await db.execute('''
      CREATE TABLE sync_log(
        record_id TEXT PRIMARY KEY,
        record_type TEXT,
        operation TEXT,
        data TEXT,
        timestamp TEXT
      )
    ''');
  }

  Future<void> _onUpgrade(Database db, int oldVersion, int newVersion) async {
    if (oldVersion < 2) {
      await db.execute(
          'ALTER TABLE products ADD COLUMN is_synced INTEGER DEFAULT 1');
      await db.execute(
          'ALTER TABLE customers ADD COLUMN is_synced INTEGER DEFAULT 1');
    }
    if (oldVersion < 3) {
      await db.execute('''
        CREATE TABLE IF NOT EXISTS local_inventory(
          inventory_uuid TEXT PRIMARY KEY,
          product_uuid TEXT,
          variant_type TEXT,
          condition TEXT,
          quantity_on_hand INTEGER,
          location_tag TEXT,
          is_synced INTEGER DEFAULT 1
        )
      ''');
      await db.execute('''
        CREATE TABLE IF NOT EXISTS transactions(
          transaction_uuid TEXT PRIMARY KEY,
          customer_uuid TEXT,
          timestamp TEXT,
          transaction_type TEXT,
          is_synced INTEGER DEFAULT 1
        )
      ''');
      await db.execute('''
        CREATE TABLE IF NOT EXISTS transaction_items(
          item_uuid TEXT PRIMARY KEY,
          transaction_uuid TEXT,
          product_uuid TEXT,
          quantity INTEGER,
          unit_price REAL,
          condition TEXT
        )
      ''');
      await db.execute('''
        CREATE TABLE IF NOT EXISTS sync_log(
          record_id TEXT PRIMARY KEY,
          record_type TEXT,
          operation TEXT,
          data TEXT,
          timestamp TEXT
        )
      ''');
    }
  }

  // --- Product Methods ---

  Future<void> saveProduct(Product product, {bool isSynced = true}) async {
    final db = await database;
    await db.insert(
      'products',
      {
        'product_uuid': product.productUuid,
        'name': product.name,
        'category': product.category.toJson(),
        'set_code': product.setCode,
        'collector_number': product.collectorNumber,
        'metadata': jsonEncode(product.metadata),
        'is_synced': isSynced ? 1 : 0,
      },
      conflictAlgorithm: ConflictAlgorithm.replace,
    );

    if (!isSynced) {
      await logSyncChange(product.productUuid, 'Product', 'Update',
          product.toJson() as Map<String, dynamic>);
    }
  }

  Future<List<Product>> getProducts() async {
    final db = await database;
    final List<Map<String, dynamic>> maps = await db.query('products');
    return List.generate(maps.length, (i) {
      final categoryStr = maps[i]['category'] as String;
      final category = Category.fromJson(categoryStr);

      return Product(
        productUuid: maps[i]['product_uuid'],
        name: maps[i]['name'],
        category: category,
        setCode: maps[i]['set_code'],
        collectorNumber: maps[i]['collector_number'],
        metadata:
            maps[i]['metadata'] != null ? jsonDecode(maps[i]['metadata']) : {},
      );
    });
  }

  Future<List<Product>> getUnsyncedProducts() async {
    final db = await database;
    final List<Map<String, dynamic>> maps =
        await db.query('products', where: 'is_synced = ?', whereArgs: [0]);
    return List.generate(maps.length, (i) {
      return Product(
        productUuid: maps[i]['product_uuid'],
        name: maps[i]['name'],
        category: Category.fromJson(maps[i]['category']),
        setCode: maps[i]['set_code'],
        collectorNumber: maps[i]['collector_number'],
        metadata:
            maps[i]['metadata'] != null ? jsonDecode(maps[i]['metadata']) : {},
      );
    });
  }

  // --- Inventory Methods ---

  Future<void> saveInventoryItem(InventoryItem item,
      {bool isSynced = true}) async {
    final db = await database;
    await db.insert(
      'local_inventory',
      {
        'inventory_uuid': item.inventoryUuid,
        'product_uuid': item.productUuid,
        'variant_type': item.variantType?.toJson(),
        'condition': item.condition.toJson(),
        'quantity_on_hand': item.quantityOnHand,
        'location_tag': item.locationTag,
        'is_synced': isSynced ? 1 : 0,
      },
      conflictAlgorithm: ConflictAlgorithm.replace,
    );

    if (!isSynced) {
      await logSyncChange(item.inventoryUuid, 'InventoryItem', 'Update',
          item.toJson() as Map<String, dynamic>);
    }
  }

  Future<List<InventoryItem>> getInventoryItems() async {
    final db = await database;
    final List<Map<String, dynamic>> maps = await db.query('local_inventory');
    return List.generate(maps.length, (i) {
      return InventoryItem(
        inventoryUuid: maps[i]['inventory_uuid'],
        productUuid: maps[i]['product_uuid'],
        variantType: maps[i]['variant_type'] != null
            ? VariantType.fromJson(maps[i]['variant_type'])
            : null,
        condition: Condition.fromJson(maps[i]['condition']),
        quantityOnHand: maps[i]['quantity_on_hand'],
        locationTag: maps[i]['location_tag'],
        minStockLevel: 0,
        serializedDetails: null,
      );
    });
  }

  // --- Customer Methods ---

  Future<void> saveCustomer(Customer customer, {bool isSynced = true}) async {
    final db = await database;
    await db.insert(
      'customers',
      {
        'customer_uuid': customer.customerUuid,
        'name': customer.name,
        'email': customer.email,
        'phone': customer.phone,
        'store_credit': customer.storeCredit,
        'created_at': customer.createdAt.toIso8601String(),
        'is_synced': isSynced ? 1 : 0,
      },
      conflictAlgorithm: ConflictAlgorithm.replace,
    );

    if (!isSynced) {
      await logSyncChange(customer.customerUuid, 'Customer', 'Update',
          customer.toJson() as Map<String, dynamic>);
    }
  }

  Future<List<Customer>> getCustomers() async {
    final db = await database;
    final List<Map<String, dynamic>> maps = await db.query('customers');
    return List.generate(maps.length, (i) {
      return Customer(
        customerUuid: maps[i]['customer_uuid'],
        name: maps[i]['name'],
        email: maps[i]['email'],
        phone: maps[i]['phone'],
        storeCredit: maps[i]['store_credit'],
        createdAt: DateTime.parse(maps[i]['created_at']),
      );
    });
  }

  Future<List<Customer>> getUnsyncedCustomers() async {
    final db = await database;
    final List<Map<String, dynamic>> maps =
        await db.query('customers', where: 'is_synced = ?', whereArgs: [0]);
    return List.generate(maps.length, (i) {
      return Customer(
        customerUuid: maps[i]['customer_uuid'],
        name: maps[i]['name'],
        email: maps[i]['email'],
        phone: maps[i]['phone'],
        storeCredit: maps[i]['store_credit'],
        createdAt: DateTime.parse(maps[i]['created_at']),
      );
    });
  }

  // --- Sync Log Methods ---

  Future<void> logSyncChange(String recordId, String recordType,
      String operation, Map<String, dynamic> data) async {
    final db = await database;
    await db.insert(
      'sync_log',
      {
        'record_id': recordId,
        'record_type': recordType,
        'operation': operation,
        'data': jsonEncode(data),
        'timestamp': DateTime.now().toIso8601String(),
      },
      conflictAlgorithm: ConflictAlgorithm.replace,
    );
  }

  Future<List<Map<String, dynamic>>> getPendingSyncChanges() async {
    final db = await database;
    final List<Map<String, dynamic>> maps =
        await db.query('sync_log', orderBy: 'timestamp ASC');
    return maps.map((map) {
      return {
        'record_id': map['record_id'],
        'record_type': map['record_type'],
        'operation': map['operation'],
        'data': jsonDecode(map['data']),
        'timestamp': map['timestamp'],
      };
    }).toList();
  }

  Future<void> clearSyncLog(String recordId) async {
    final db = await database;
    await db.delete('sync_log', where: 'record_id = ?', whereArgs: [recordId]);
  }

  Future<void> applyRemoteChanges(List<ChangeRecord> changes) async {
    final db = await database;
    final batch = db.batch();

    for (var change in changes) {
      switch (change.operation) {
        case SyncOperation.delete:
          _applyDelete(batch, change);
          break;
        case SyncOperation.insert:
        case SyncOperation.update:
          _applyUpsert(batch, change);
          break;
      }
    }

    await batch.commit(noResult: true);
  }

  void _applyDelete(Batch batch, ChangeRecord change) {
    String table;
    String idColumn;

    switch (change.recordType) {
      case RecordType.product:
        table = 'products';
        idColumn = 'product_uuid';
        break;
      case RecordType.inventoryItem:
        table = 'local_inventory';
        idColumn = 'inventory_uuid';
        break;
      case RecordType.customer:
        table = 'customers';
        idColumn = 'customer_uuid';
        break;
      case RecordType.transaction:
        table = 'transactions';
        idColumn = 'transaction_uuid';
        break;
      default:
        return;
    }

    batch.delete(table, where: '$idColumn = ?', whereArgs: [change.recordId]);
  }

  void _applyUpsert(Batch batch, ChangeRecord change) {
    try {
      switch (change.recordType) {
        case RecordType.product:
          final product = Product.fromJson(change.data);
          batch.insert(
            'products',
            {
              'product_uuid': product.productUuid,
              'name': product.name,
              'category': product.category.toJson(),
              'set_code': product.setCode,
              'collector_number': product.collectorNumber,
              'metadata': jsonEncode(product.metadata),
              'is_synced': 1,
            },
            conflictAlgorithm: ConflictAlgorithm.replace,
          );
          break;

        case RecordType.inventoryItem:
          final item = InventoryItem.fromJson(change.data);
          batch.insert(
            'local_inventory',
            {
              'inventory_uuid': item.inventoryUuid,
              'product_uuid': item.productUuid,
              'variant_type': item.variantType?.toJson(),
              'condition': item.condition.toJson(),
              'quantity_on_hand': item.quantityOnHand,
              'location_tag': item.locationTag,
              'is_synced': 1,
            },
            conflictAlgorithm: ConflictAlgorithm.replace,
          );
          break;

        case RecordType.customer:
          final customer = Customer.fromJson(change.data);
          // Manually construct map to ensure consistent types
          batch.insert(
            'customers',
            {
              'customer_uuid': customer.customerUuid,
              'name': customer.name,
              'email': customer.email,
              'phone': customer.phone,
              'store_credit': customer.storeCredit,
              'created_at': customer.createdAt.toIso8601String(),
              'is_synced': 1,
            },
            conflictAlgorithm: ConflictAlgorithm.replace,
          );
          break;

        default:
          // Handle other types or ignore
          break;
      }
    } catch (e) {
      debugPrint('Error applying remote change for ${change.recordId}: $e');
    }
  }
}
