# Phase 6: Cash Drawer & Hardware Integration

**Priority:** P2 - Medium (Enhancement)
**Status:** COMPLETE (13/13 tasks)
**Duration:** Weeks 11-12

---

## 6.1 Cash Drawer Integration

### TASK-137: Create cash drawer service interface
- **Status:** [x] Complete
- **Implementation:** `CashDrawerService` in `src/services/cash_drawer.rs`

### TASK-138: Implement `POST /api/cash-drawer/open` endpoint
- **Status:** [x] Complete
- **Endpoint:** Returns ESC/POS command bytes as base64

### TASK-139: Implement `POST /api/cash-drawer/count` endpoint
- **Status:** [x] Complete
- **Endpoint:** Records denomination counts with automatic total calculation

### TASK-140: Add cash drawer kick on sale completion
- **Status:** [x] Complete
- **Implementation:** `get_open_drawer_command()` returns ESC/POS kick command

### TASK-141: Create cash counting workflow
- **Status:** [x] Complete
- **Implementation:** `CashCount` struct with denomination breakdown, `calculate_total()` method

### TASK-142: Implement till reconciliation
- **Status:** [x] Complete
- **Implementation:** `close_shift()` calculates expected vs actual cash

### TASK-143: Add shift open/close cash counts
- **Status:** [x] Complete
- **Endpoints:** `POST /api/shifts`, `POST /api/shifts/:shift_uuid/close`
- **Implementation:** `open_shift()`, `close_shift()` with cash count linking

### TASK-144: Create cash variance reporting
- **Status:** [x] Complete
- **Endpoint:** `GET /api/reports/cash-variance`
- **Implementation:** `get_variance_report()` returns `CashVarianceReport`

---

## 6.2 Printer Integration

### TASK-145: Create printer service interface
- **Status:** [x] Complete
- **Implementation:** `PrinterService` in `src/services/printer.rs`

### TASK-146: Implement ESC/POS command generation
- **Status:** [x] Complete
- **Implementation:** `EscPosBuilder` with fluent API for receipt formatting

### TASK-147: Add thermal printer discovery
- **Status:** [x] Complete
- **Endpoint:** `GET /api/printers`
- **Implementation:** `discover_printers()`, `register_printer()`

### TASK-148: Implement label printer support
- **Status:** [x] Complete
- **Implementation:** `generate_label_escpos()` for thermal label printers

### TASK-149: Add print queue management
- **Status:** [x] Complete
- **Endpoint:** `GET /api/printers/queue`
- **Implementation:** `queue_print_job()`, `get_pending_jobs()`, `process_next_job()`

---

## Database Schema

### Migration 24: Shifts Table
```sql
CREATE TABLE Shifts (
    shift_uuid TEXT PRIMARY KEY,
    user_uuid TEXT NOT NULL,
    terminal_id TEXT NOT NULL,
    opened_at TEXT NOT NULL,
    closed_at TEXT,
    opening_count_uuid TEXT,
    closing_count_uuid TEXT,
    expected_cash REAL NOT NULL DEFAULT 0,
    actual_cash REAL,
    variance REAL,
    status TEXT NOT NULL DEFAULT 'open'
);
```

### Migration 25: Cash_Counts Table
```sql
CREATE TABLE Cash_Counts (
    count_uuid TEXT PRIMARY KEY,
    shift_uuid TEXT,
    count_type TEXT NOT NULL,
    pennies INTEGER, nickels INTEGER, dimes INTEGER, quarters INTEGER,
    ones INTEGER, fives INTEGER, tens INTEGER, twenties INTEGER,
    fifties INTEGER, hundreds INTEGER,
    total_amount REAL NOT NULL,
    counted_by TEXT,
    counted_at TEXT NOT NULL,
    notes TEXT
);
```

---

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/cash-drawer/open` | POST | Get ESC/POS command to open drawer |
| `/api/cash-drawer/count` | POST | Record a cash count |
| `/api/shifts` | POST | Open a new shift |
| `/api/shifts/:shift_uuid/close` | POST | Close shift with closing count |
| `/api/shifts/terminal/:terminal_id` | GET | Get current open shift |
| `/api/reports/cash-variance` | GET | Cash variance report |
| `/api/printers` | GET | List discovered printers |
| `/api/printers/queue` | GET | List pending print jobs |
