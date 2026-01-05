/// Returns the list of schema migrations to be applied
pub fn get_schema_migrations() -> Vec<(i32, &'static str, Vec<&'static str>)> {
    vec![
        (1, "Initial Schema", vec![
            "CREATE TABLE IF NOT EXISTS Global_Catalog (
                product_uuid TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                category TEXT NOT NULL,
                set_code TEXT,
                collector_number TEXT,
                barcode TEXT,
                release_year INTEGER,
                metadata TEXT
            )",
            "CREATE TABLE IF NOT EXISTS Local_Inventory (
                inventory_uuid TEXT PRIMARY KEY,
                product_uuid TEXT NOT NULL,
                variant_type TEXT,
                condition TEXT NOT NULL,
                quantity_on_hand INTEGER NOT NULL,
                location_tag TEXT NOT NULL,
                FOREIGN KEY (product_uuid) REFERENCES Global_Catalog (product_uuid)
            )",
            "CREATE TABLE IF NOT EXISTS Pricing_Matrix (
                price_uuid TEXT PRIMARY KEY,
                product_uuid TEXT NOT NULL,
                market_mid REAL,
                market_low REAL,
                last_sync_timestamp TEXT,
                FOREIGN KEY (product_uuid) REFERENCES Global_Catalog (product_uuid)
            )",
            "CREATE TABLE IF NOT EXISTS Transactions (
                transaction_uuid TEXT PRIMARY KEY,
                customer_uuid TEXT,
                timestamp TEXT NOT NULL,
                transaction_type TEXT NOT NULL
            )",
            "CREATE TABLE IF NOT EXISTS Transaction_Items (
                item_uuid TEXT PRIMARY KEY,
                transaction_uuid TEXT NOT NULL,
                product_uuid TEXT NOT NULL,
                quantity INTEGER NOT NULL,
                unit_price REAL NOT NULL,
                condition TEXT NOT NULL,
                FOREIGN KEY (transaction_uuid) REFERENCES Transactions (transaction_uuid),
                FOREIGN KEY (product_uuid) REFERENCES Global_Catalog (product_uuid)
            )",
            "CREATE TABLE IF NOT EXISTS Customers (
                customer_uuid TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT,
                phone TEXT,
                store_credit REAL NOT NULL DEFAULT 0.0,
                created_at TEXT NOT NULL
            )",
            "CREATE TABLE IF NOT EXISTS Users (
                user_uuid TEXT PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                role TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
        ]),
        (2, "Sync Log", vec![
            "CREATE TABLE IF NOT EXISTS Sync_Log (
                record_id TEXT PRIMARY KEY,
                record_type TEXT NOT NULL,
                operation TEXT NOT NULL,
                data TEXT NOT NULL,
                node_id TEXT NOT NULL,
                vector_clock INTEGER NOT NULL,
                timestamp TEXT NOT NULL
            )"
        ]),
        (3, "Indexes", vec![
            "CREATE INDEX IF NOT EXISTS idx_products_name ON Global_Catalog(name)",
            "CREATE INDEX IF NOT EXISTS idx_products_category ON Global_Catalog(category)",
            "CREATE INDEX IF NOT EXISTS idx_inventory_product ON Local_Inventory(product_uuid)",
            "CREATE INDEX IF NOT EXISTS idx_pricing_product ON Pricing_Matrix(product_uuid)",
            "CREATE INDEX IF NOT EXISTS idx_transactions_customer ON Transactions(customer_uuid)",
            "CREATE INDEX IF NOT EXISTS idx_transactions_timestamp ON Transactions(timestamp)",
            "CREATE INDEX IF NOT EXISTS idx_transaction_items_transaction ON Transaction_Items(transaction_uuid)",
            "CREATE INDEX IF NOT EXISTS idx_transaction_items_product ON Transaction_Items(product_uuid)",
            "CREATE INDEX IF NOT EXISTS idx_sync_log_clock ON Sync_Log(vector_clock)"
        ]),
        (4, "Version Vectors", vec![
            "ALTER TABLE Sync_Log RENAME COLUMN vector_clock TO local_clock",
            "ALTER TABLE Sync_Log ADD COLUMN version_vector TEXT",
            "CREATE TABLE IF NOT EXISTS Version_Vectors (
                entity_uuid TEXT NOT NULL,
                node_id TEXT NOT NULL,
                counter INTEGER NOT NULL,
                PRIMARY KEY (entity_uuid, node_id)
            )"
        ]),
        (5, "Serialized Inventory", vec![
            "ALTER TABLE Local_Inventory ADD COLUMN specific_price REAL",
            "ALTER TABLE Local_Inventory ADD COLUMN serialized_details TEXT"
        ]),
        (6, "Wants List", vec![
            "CREATE TABLE IF NOT EXISTS Wants_Lists (
                wants_list_uuid TEXT PRIMARY KEY,
                customer_uuid TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (customer_uuid) REFERENCES Customers (customer_uuid)
            )",
            "CREATE TABLE IF NOT EXISTS Wants_Items (
                item_uuid TEXT PRIMARY KEY,
                wants_list_uuid TEXT NOT NULL,
                product_uuid TEXT NOT NULL,
                min_condition TEXT NOT NULL,
                max_price REAL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (wants_list_uuid) REFERENCES Wants_Lists (wants_list_uuid),
                FOREIGN KEY (product_uuid) REFERENCES Global_Catalog (product_uuid)
            )"
        ]),
        (7, "Events", vec![
            "CREATE TABLE IF NOT EXISTS Events (
                event_uuid TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                event_type TEXT NOT NULL,
                date TEXT NOT NULL,
                entry_fee REAL,
                max_participants INTEGER,
                created_at TEXT NOT NULL
            )",
            "CREATE TABLE IF NOT EXISTS Event_Participants (
                participant_uuid TEXT PRIMARY KEY,
                event_uuid TEXT NOT NULL,
                customer_uuid TEXT,
                name TEXT NOT NULL,
                paid BOOLEAN NOT NULL DEFAULT 0,
                placement INTEGER,
                created_at TEXT NOT NULL,
                FOREIGN KEY (event_uuid) REFERENCES Events (event_uuid)
            )"
        ]),
         (8, "Audit", vec![
            "CREATE TABLE IF NOT EXISTS Price_Overrides (
                override_uuid TEXT PRIMARY KEY,
                product_uuid TEXT NOT NULL,
                new_price REAL NOT NULL,
                reason TEXT NOT NULL,
                user_uuid TEXT,
                timestamp TEXT NOT NULL
            )"
        ]),
        // HIGH-007 / LOW-005 FIX: Add performance indexes
        (9, "Performance Indexes", vec![
            "CREATE INDEX IF NOT EXISTS idx_wants_items_product ON Wants_Items(product_uuid)",
            "CREATE INDEX IF NOT EXISTS idx_inventory_product ON Local_Inventory(product_uuid)",
            "CREATE INDEX IF NOT EXISTS idx_inventory_condition ON Local_Inventory(condition)",
            "CREATE INDEX IF NOT EXISTS idx_transaction_items_product ON Transaction_Items(product_uuid)",
            "CREATE INDEX IF NOT EXISTS idx_transactions_customer ON Transactions(customer_uuid)",
            "CREATE INDEX IF NOT EXISTS idx_transactions_timestamp ON Transactions(timestamp)",
            "CREATE INDEX IF NOT EXISTS idx_products_name ON Global_Catalog(name)",
            "CREATE INDEX IF NOT EXISTS idx_products_barcode ON Global_Catalog(barcode)"
        ]),
        // MED-010 FIX: Add soft delete columns
        (10, "Soft Deletes", vec![
            "ALTER TABLE Local_Inventory ADD COLUMN deleted_at TEXT DEFAULT NULL",
            "ALTER TABLE Customers ADD COLUMN deleted_at TEXT DEFAULT NULL",
            "ALTER TABLE Global_Catalog ADD COLUMN deleted_at TEXT DEFAULT NULL",
            "CREATE INDEX IF NOT EXISTS idx_inventory_deleted ON Local_Inventory(deleted_at)",
            "CREATE INDEX IF NOT EXISTS idx_customers_deleted ON Customers(deleted_at)",
            "CREATE INDEX IF NOT EXISTS idx_products_deleted ON Global_Catalog(deleted_at)"
        ]),
        // MED-004 FIX: Persistent pricing rules
        (11, "Pricing Rules", vec![
            "CREATE TABLE IF NOT EXISTS Pricing_Rules (
                rule_id TEXT PRIMARY KEY,
                priority INTEGER NOT NULL,
                category TEXT,
                condition TEXT,
                min_market_price REAL,
                max_market_price REAL,
                cash_multiplier REAL NOT NULL,
                credit_multiplier REAL NOT NULL,
                is_active INTEGER DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"
        ]),
        // MED-005 FIX: Refresh Tokens
        (12, "Refresh Tokens", vec![
            "CREATE TABLE IF NOT EXISTS Refresh_Tokens (
                token_hash TEXT PRIMARY KEY,
                user_uuid TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                created_at TEXT NOT NULL,
                is_revoked INTEGER DEFAULT 0
            )",
            "CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user ON Refresh_Tokens(user_uuid)"
        ]),
        // Feature: Reconciliation & Audit
        (13, "Inventory Conflicts", vec![
            "CREATE TABLE IF NOT EXISTS Inventory_Conflicts (
                conflict_uuid TEXT PRIMARY KEY,
                product_uuid TEXT NOT NULL,
                conflict_type TEXT NOT NULL,
                terminal_ids TEXT,
                expected_quantity INTEGER,
                actual_quantity INTEGER,
                resolution_status TEXT NOT NULL DEFAULT 'Pending',
                resolved_by TEXT,
                resolution_notes TEXT,
                created_at TEXT NOT NULL,
                resolved_at TEXT,
                FOREIGN KEY (product_uuid) REFERENCES Global_Catalog (product_uuid)
            )",
            "CREATE INDEX IF NOT EXISTS idx_conflicts_status ON Inventory_Conflicts(resolution_status)",
            "CREATE INDEX IF NOT EXISTS idx_conflicts_product ON Inventory_Conflicts(product_uuid)"
        ]),
        // Phase 1: Tax, Payment, Locations, Suppliers
        (14, "Business Core Tables", vec![
            // Tax Rates table
            "CREATE TABLE IF NOT EXISTS Tax_Rates (
                rate_id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                rate REAL NOT NULL CHECK(rate >= 0 AND rate <= 1),
                applies_to_category TEXT,
                is_default INTEGER DEFAULT 0,
                is_active INTEGER DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            // Payment Methods for transactions
            "CREATE TABLE IF NOT EXISTS Payment_Methods (
                payment_uuid TEXT PRIMARY KEY,
                transaction_uuid TEXT NOT NULL,
                method_type TEXT NOT NULL CHECK(method_type IN ('Cash', 'Card', 'StoreCredit', 'Check', 'Other')),
                amount REAL NOT NULL,
                reference TEXT,
                card_last_four TEXT,
                auth_code TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (transaction_uuid) REFERENCES Transactions(transaction_uuid)
            )",
            "CREATE INDEX IF NOT EXISTS idx_payment_methods_transaction ON Payment_Methods(transaction_uuid)",
            // Store Locations
            "CREATE TABLE IF NOT EXISTS Store_Locations (
                location_uuid TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                address TEXT,
                phone TEXT,
                is_primary INTEGER DEFAULT 0,
                is_active INTEGER DEFAULT 1,
                created_at TEXT NOT NULL
            )",
            // Suppliers
            "CREATE TABLE IF NOT EXISTS Suppliers (
                supplier_uuid TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                contact_name TEXT,
                email TEXT,
                phone TEXT,
                address TEXT,
                payment_terms TEXT,
                notes TEXT,
                is_active INTEGER DEFAULT 1,
                created_at TEXT NOT NULL
            )"
        ]),
        // Phase 1: Transaction and Customer column additions  
        (15, "Transaction Extensions", vec![
            "ALTER TABLE Transactions ADD COLUMN subtotal REAL DEFAULT 0",
            "ALTER TABLE Transactions ADD COLUMN tax_amount REAL DEFAULT 0",
            "ALTER TABLE Transactions ADD COLUMN total REAL DEFAULT 0",
            "ALTER TABLE Transactions ADD COLUMN cash_tendered REAL",
            "ALTER TABLE Transactions ADD COLUMN change_given REAL",
            "ALTER TABLE Transactions ADD COLUMN notes TEXT",
            "ALTER TABLE Transactions ADD COLUMN void_reason TEXT",
            "ALTER TABLE Transactions ADD COLUMN voided_at TEXT",
            "ALTER TABLE Transactions ADD COLUMN location_uuid TEXT",
            // Customer extensions
            "ALTER TABLE Customers ADD COLUMN trade_in_limit REAL DEFAULT 500.0",
            "ALTER TABLE Customers ADD COLUMN is_banned INTEGER DEFAULT 0",
            "ALTER TABLE Customers ADD COLUMN ban_reason TEXT",
            "ALTER TABLE Customers ADD COLUMN notes TEXT",
            "ALTER TABLE Customers ADD COLUMN preferred_contact TEXT",
            "ALTER TABLE Customers ADD COLUMN tax_exempt INTEGER DEFAULT 0",
            "ALTER TABLE Customers ADD COLUMN tax_exempt_id TEXT",
            "ALTER TABLE Customers ADD COLUMN loyalty_points INTEGER DEFAULT 0",
            "ALTER TABLE Customers ADD COLUMN tier TEXT DEFAULT 'Standard'"
        ]),
        // Phase 1: Inventory and Product extensions
        (16, "Inventory Extensions", vec![
            "ALTER TABLE Local_Inventory ADD COLUMN cost_basis REAL",
            "ALTER TABLE Local_Inventory ADD COLUMN supplier_uuid TEXT",
            "ALTER TABLE Local_Inventory ADD COLUMN received_date TEXT",
            "ALTER TABLE Local_Inventory ADD COLUMN min_stock_level INTEGER DEFAULT 0",
            "ALTER TABLE Local_Inventory ADD COLUMN max_stock_level INTEGER",
            "ALTER TABLE Local_Inventory ADD COLUMN reorder_point INTEGER",
            "ALTER TABLE Local_Inventory ADD COLUMN bin_location TEXT",
            "ALTER TABLE Local_Inventory ADD COLUMN last_sold_date TEXT",
            "ALTER TABLE Local_Inventory ADD COLUMN last_counted_date TEXT",
            // Product extensions
            "ALTER TABLE Global_Catalog ADD COLUMN weight_oz REAL",
            "ALTER TABLE Global_Catalog ADD COLUMN length_in REAL",
            "ALTER TABLE Global_Catalog ADD COLUMN width_in REAL",
            "ALTER TABLE Global_Catalog ADD COLUMN height_in REAL",
            "ALTER TABLE Global_Catalog ADD COLUMN upc TEXT",
            "ALTER TABLE Global_Catalog ADD COLUMN isbn TEXT",
            "ALTER TABLE Global_Catalog ADD COLUMN manufacturer TEXT",
            "ALTER TABLE Global_Catalog ADD COLUMN msrp REAL",
            // Event extensions
            "ALTER TABLE Events ADD COLUMN prize_pool REAL DEFAULT 0",
            "ALTER TABLE Events ADD COLUMN format TEXT",
            "ALTER TABLE Events ADD COLUMN description TEXT",
            "ALTER TABLE Events ADD COLUMN results_json TEXT",
            "ALTER TABLE Events ADD COLUMN status TEXT DEFAULT 'Scheduled'"
        ]),
        // Phase 1: Additional indexes
        (17, "Phase1 Indexes", vec![
            "CREATE INDEX IF NOT EXISTS idx_transactions_type ON Transactions(transaction_type)",
            "CREATE INDEX IF NOT EXISTS idx_transaction_items_product_condition ON Transaction_Items(product_uuid, condition)",
            "CREATE INDEX IF NOT EXISTS idx_pricing_matrix_sync ON Pricing_Matrix(last_sync_timestamp)",
            "CREATE INDEX IF NOT EXISTS idx_inventory_location_product ON Local_Inventory(location_tag, product_uuid)",
            "CREATE INDEX IF NOT EXISTS idx_products_upc ON Global_Catalog(upc)",
            "CREATE INDEX IF NOT EXISTS idx_products_isbn ON Global_Catalog(isbn)",
            "CREATE INDEX IF NOT EXISTS idx_inventory_supplier ON Local_Inventory(supplier_uuid)"
        ]),
        // Phase 2: Layaway/Holds system
        (18, "Holds System", vec![
            "CREATE TABLE IF NOT EXISTS Holds (
                hold_uuid TEXT PRIMARY KEY,
                customer_uuid TEXT NOT NULL,
                status TEXT NOT NULL CHECK(status IN ('Active', 'Completed', 'Cancelled', 'Expired')),
                total_amount REAL NOT NULL,
                deposit_amount REAL NOT NULL,
                balance_due REAL NOT NULL,
                expiration_date TEXT NOT NULL,
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (customer_uuid) REFERENCES Customers(customer_uuid)
            )",
            "CREATE TABLE IF NOT EXISTS Hold_Items (
                item_uuid TEXT PRIMARY KEY,
                hold_uuid TEXT NOT NULL,
                inventory_uuid TEXT NOT NULL,
                quantity INTEGER NOT NULL,
                unit_price REAL NOT NULL,
                FOREIGN KEY (hold_uuid) REFERENCES Holds(hold_uuid),
                FOREIGN KEY (inventory_uuid) REFERENCES Local_Inventory(inventory_uuid)
            )",
            "CREATE TABLE IF NOT EXISTS Hold_Payments (
                payment_uuid TEXT PRIMARY KEY,
                hold_uuid TEXT NOT NULL,
                amount REAL NOT NULL,
                payment_method TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (hold_uuid) REFERENCES Holds(hold_uuid)
            )",
            "CREATE INDEX IF NOT EXISTS idx_holds_customer ON Holds(customer_uuid)",
            "CREATE INDEX IF NOT EXISTS idx_holds_status ON Holds(status)",
            "CREATE INDEX IF NOT EXISTS idx_hold_items_hold ON Hold_Items(hold_uuid)"
        ]),
        // Damaged Items and Consignment
        (19, "Damaged and Consignment", vec![
            "CREATE TABLE IF NOT EXISTS Damaged_Items (
                damage_uuid TEXT PRIMARY KEY,
                inventory_uuid TEXT NOT NULL,
                quantity INTEGER NOT NULL,
                damage_type TEXT NOT NULL CHECK(damage_type IN ('Defective', 'Damaged', 'Missing', 'Expired', 'Other')),
                description TEXT,
                disposition TEXT CHECK(disposition IN ('Pending', 'WriteOff', 'ReturnToVendor', 'Discounted', 'Repaired')),
                original_value REAL,
                recovered_value REAL DEFAULT 0,
                reported_by TEXT,
                created_at TEXT NOT NULL,
                resolved_at TEXT,
                FOREIGN KEY (inventory_uuid) REFERENCES Local_Inventory(inventory_uuid)
            )",
            "CREATE TABLE IF NOT EXISTS Consignors (
                consignor_uuid TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT,
                phone TEXT,
                commission_rate REAL NOT NULL DEFAULT 0.4,
                balance_owed REAL NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            )",
            "CREATE TABLE IF NOT EXISTS Consignment_Items (
                consignment_uuid TEXT PRIMARY KEY,
                consignor_uuid TEXT NOT NULL,
                inventory_uuid TEXT NOT NULL,
                asking_price REAL NOT NULL,
                minimum_price REAL,
                commission_rate REAL NOT NULL,
                status TEXT NOT NULL CHECK(status IN ('Active', 'Sold', 'Returned', 'Expired')),
                received_date TEXT NOT NULL,
                sold_date TEXT,
                FOREIGN KEY (consignor_uuid) REFERENCES Consignors(consignor_uuid),
                FOREIGN KEY (inventory_uuid) REFERENCES Local_Inventory(inventory_uuid)
            )",
            "CREATE INDEX IF NOT EXISTS idx_consignment_consignor ON Consignment_Items(consignor_uuid)",
            "CREATE INDEX IF NOT EXISTS idx_consignment_status ON Consignment_Items(status)"
        ]),
        // Advanced Pricing Rules (Tasks 083, 084, 085)
        (20, "Advanced Pricing Rules", vec![
            "ALTER TABLE Pricing_Rules ADD COLUMN start_date TEXT",
            "ALTER TABLE Pricing_Rules ADD COLUMN end_date TEXT",
            "ALTER TABLE Pricing_Rules ADD COLUMN customer_tier TEXT",
            "ALTER TABLE Pricing_Rules ADD COLUMN min_quantity INTEGER"
        ]),
        // Price History (Task 086)
        (21, "Price History", vec![
            "CREATE TABLE IF NOT EXISTS Price_History (
                history_uuid TEXT PRIMARY KEY,
                product_uuid TEXT NOT NULL,
                market_mid REAL NOT NULL,
                market_low REAL NOT NULL,
                source TEXT NOT NULL,
                recorded_at TEXT NOT NULL,
                FOREIGN KEY (product_uuid) REFERENCES Global_Catalog(product_uuid)
            )",
            "CREATE INDEX IF NOT EXISTS idx_price_history_date ON Price_History(recorded_at)"
        ]),
        // Barcode Scan Logging (Task 098)
        (22, "Scan Logs", vec![
            "CREATE TABLE IF NOT EXISTS Scan_Logs (
                scan_uuid TEXT PRIMARY KEY,
                scanned_value TEXT NOT NULL,
                scan_type TEXT NOT NULL, 
                result_type TEXT, 
                result_uuid TEXT,
                user_uuid TEXT,
                scanned_at TEXT NOT NULL
            )",
            "CREATE INDEX IF NOT EXISTS idx_scan_logs_date ON Scan_Logs(scanned_at)",
            "CREATE INDEX IF NOT EXISTS idx_scan_logs_user ON Scan_Logs(user_uuid)"
        ]),
        // Report cache table
        (23, "Report Cache", vec![
            "CREATE TABLE IF NOT EXISTS Report_Cache (
                report_key TEXT PRIMARY KEY,
                report_type TEXT NOT NULL,
                data TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT NOT NULL
            )",
            "CREATE INDEX IF NOT EXISTS idx_report_cache_type ON Report_Cache(report_type)"
        ]),
        // Notification system
        (24, "Notification System", vec![
             "CREATE TABLE IF NOT EXISTS Notifications (
                notification_uuid TEXT PRIMARY KEY,
                user_uuid TEXT,
                title TEXT NOT NULL,
                message TEXT NOT NULL,
                is_read INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                expires_at TEXT
             )",
             "CREATE INDEX IF NOT EXISTS idx_notifications_user ON Notifications(user_uuid)",
             "CREATE INDEX IF NOT EXISTS idx_notifications_unread ON Notifications(user_uuid, is_read)"
        ]),
        // Offline Queue
        (25, "Offline Queue", vec![
            "CREATE TABLE IF NOT EXISTS Offline_Queue (
                queue_id TEXT PRIMARY KEY,
                action_type TEXT NOT NULL,
                payload TEXT NOT NULL,
                status TEXT NOT NULL CHECK(status IN ('pending', 'processing', 'completed', 'failed')),
                created_at TEXT NOT NULL,
                retry_count INTEGER DEFAULT 0,
                last_error TEXT
            )",
            "CREATE INDEX IF NOT EXISTS idx_offline_queue_status ON Offline_Queue(status)"
        ]),
        // Conflict Resolution (Task 121 - UI Support)
        (26, "Conflict Resolution", vec![
            "CREATE TABLE IF NOT EXISTS Sync_Conflicts (
                conflict_uuid TEXT PRIMARY KEY,
                resource_type TEXT NOT NULL,
                resource_uuid TEXT NOT NULL,
                conflict_type TEXT NOT NULL, -- 'Oversold', 'Price_Mismatch', 'Concurrent_Mod'
                resolution_status TEXT NOT NULL DEFAULT 'Pending',
                detected_at TEXT NOT NULL,
                resolved_at TEXT,
                resolved_by_user TEXT,
                resolution_strategy TEXT -- 'LocalWins', 'RemoteWins', 'Manual'
            )",
            "CREATE TABLE IF NOT EXISTS Conflict_Snapshots (
                snapshot_uuid TEXT PRIMARY KEY,
                conflict_uuid TEXT NOT NULL,
                node_id TEXT NOT NULL,
                state_data TEXT NOT NULL,
                vector_clock TEXT,
                FOREIGN KEY (conflict_uuid) REFERENCES Sync_Conflicts(conflict_uuid)
            )",
            "CREATE INDEX IF NOT EXISTS idx_conflicts_status ON Sync_Conflicts(resolution_status)"
        ]),
        // Inventory Audit Conflicts (AuditService support)
        (27, "Inventory Audit Conflicts", vec![
            "CREATE TABLE IF NOT EXISTS Inventory_Conflicts (
                conflict_uuid TEXT PRIMARY KEY,
                product_uuid TEXT NOT NULL,
                conflict_type TEXT NOT NULL,
                terminal_ids TEXT, -- JSON array of terminal IDs
                expected_quantity INTEGER NOT NULL,
                actual_quantity INTEGER NOT NULL,
                resolution_status TEXT NOT NULL,
                resolved_by TEXT,
                resolution_notes TEXT,
                created_at TEXT NOT NULL,
                resolved_at TEXT,
                FOREIGN KEY (product_uuid) REFERENCES Global_Catalog (product_uuid)
            )",
            "CREATE INDEX IF NOT EXISTS idx_inv_conflicts_status ON Inventory_Conflicts(resolution_status)"
        ]),
        // CRITICAL FIX: Add user_uuid to Transactions (Missing column fix)
        (28, "Add User to Transaction", vec![
            "ALTER TABLE Transactions ADD COLUMN user_uuid TEXT",
            "CREATE INDEX IF NOT EXISTS idx_transactions_user ON Transactions(user_uuid)"
        ]),
    ]
}
