# VaultSync Master Remediation Plan

**Created:** 2026-01-02  
**Status:** ACTIVE  
**Total Tasks:** 180+  
**Estimated Duration:** 6-9 months

---

## PHASE 0: CRITICAL SECURITY & CONFIGURATION (Week 1)
*Must be completed before ANY other work - these are immediate security risks*

### 0.1 Security Hardening
- [x] **TASK-001**: Remove hardcoded JWT secret from `.env` file
- [x] **TASK-002**: Create `.env.example` with placeholder values
- [x] **TASK-003**: Add `.env` to `.gitignore` if not already present
- [x] **TASK-004**: Implement environment variable validation on startup
- [x] **TASK-005**: Create secure secret generation script for deployment
- [x] **TASK-006**: Replace `CorsLayer::permissive()` with configurable allowed origins
- [x] **TASK-007**: Add CORS configuration to environment variables

### 0.2 Configuration Management
- [x] **TASK-008**: Create configuration struct with all required settings
- [x] **TASK-009**: Implement configuration validation on startup (fail fast)
- [x] **TASK-010**: Add NODE_ID auto-generation using machine identifier + random suffix
- [x] **TASK-011**: Make database connection pool size configurable
- [x] **TASK-012**: Make JWT expiration configurable
- [x] **TASK-013**: Add rate limiting configuration options
- [x] **TASK-014**: Create deployment environment profiles (dev/staging/prod)

### 0.3 Frontend Configuration
- [x] **TASK-015**: Remove hardcoded `localhost:3000` from `api_service.dart`
- [x] **TASK-016**: Implement environment-based API URL configuration
- [x] **TASK-017**: Create Flutter build configurations for dev/staging/prod
- [x] **TASK-018**: Add API URL to Flutter environment variables

---

## PHASE 1: DATABASE FOUNDATION (Weeks 2-3)
*Fix the data layer before building features on top*

### 1.1 Schema Migrations - Critical Tables
- [x] **TASK-019**: Create `Payment_Methods` table
- [x] **TASK-020**: Create `Tax_Rates` table
- [x] **TASK-021**: Create `Store_Locations` table
- [x] **TASK-022**: Create `Holds` table (layaway system)
- [x] **TASK-023**: Create `Damaged_Items` table
- [x] **TASK-024**: Create `Consignment` table
- [x] **TASK-025**: Create `Suppliers` table

### 1.2 Schema Migrations - Column Additions
- [x] **TASK-026**: Add to Transactions: `subtotal`, `tax_amount`, `total`, `cash_tendered`, `change_given`
- [x] **TASK-027**: Add to Customers: `trade_in_limit`, `is_banned`, `notes`, `preferred_contact`, `tax_exempt`
- [x] **TASK-028**: Add to Local_Inventory: `cost_basis`, `supplier_uuid`, `received_date`
- [x] **TASK-029**: Add to Global_Catalog: `weight`, `dimensions`, `upc`, `isbn`
- [x] **TASK-030**: Add to Events: `prize_pool`, `format`, `results_json`

### 1.3 Schema Migrations - Indexes
- [x] **TASK-031**: Add index on `Transactions.transaction_type`
- [x] **TASK-032**: Add composite index on `Transaction_Items(product_uuid, condition)`
- [x] **TASK-033**: Add index on `Pricing_Matrix.last_sync_timestamp`
- [x] **TASK-034**: Add composite index on `Local_Inventory(location_tag, product_uuid)`
- [x] **TASK-035**: Add index on `Global_Catalog.upc`
- [x] **TASK-036**: Add index on `Global_Catalog.isbn`

### 1.4 Repository Layer Fixes
- [x] **TASK-037**: Create `PaymentMethodRepository` (Part of PaymentService)
- [x] **TASK-038**: Create `TaxRateRepository` (Part of TaxService)
- [x] **TASK-039**: Create `LocationRepository` (Part of StoreService/Config)
- [x] **TASK-040**: Create `HoldRepository` (Part of HoldsService)
- [x] **TASK-041**: Create `SupplierRepository` (Part of InventoryService)
- [x] **TASK-042**: Update `TransactionRepository` to handle new columns
- [x] **TASK-043**: Update `CustomerRepository` to handle new columns
- [x] **TASK-044**: Update `InventoryRepository` to handle new columns
- [x] **TASK-045**: Fix N+1 query issues in `TransactionRepository.get_by_customer()`

---

## PHASE 2: CORE BUSINESS LOGIC (Weeks 3-5)
*Implement essential business operations*

### 2.1 Tax Calculation System
- [x] **TASK-046**: Create `TaxService` with rate lookup
- [x] **TASK-047**: Implement tax calculation for transactions
- [x] **TASK-048**: Add customer tax-exempt flag handling
- [x] **TASK-049**: Add category-based tax rate support
- [x] **TASK-050**: Create tax rate CRUD API endpoints
- [x] **TASK-051**: Add tax amount to transaction response

### 2.2 Payment Processing
- [x] **TASK-052**: Create `PaymentService` interface
- [x] **TASK-053**: Implement cash payment handler
- [x] **TASK-054**: Implement store credit payment handler
- [x] **TASK-055**: Implement split payment support
- [x] **TASK-056**: Add payment method recording to transactions
- [x] **TASK-057**: Create payment reconciliation logic
- [x] **TASK-058**: Add change calculation for cash payments

### 2.3 Transaction Fixes
- [x] **TASK-059**: Add quantity validation (no negative values)
- [x] **TASK-060**: Add price validation (no negative, optional floor/ceiling)
- [x] **TASK-061**: Implement atomic transaction with inventory update
- [x] **TASK-062**: Add transaction rollback on partial failure
- [x] **TASK-063**: Fix bulk operations to use transactions (Handled via TransactionService)
- [x] **TASK-064**: Add proper error messages for validation failures

### 2.4 Inventory Validation
- [x] **TASK-065**: Add stock level validation before sale
- [x] **TASK-066**: Implement negative stock prevention
- [x] **TASK-067**: Add low stock warnings on sale
- [x] **TASK-068**: Implement reserved quantity for holds
- [x] **TASK-069**: Add inventory adjustment audit logging

---

## PHASE 3: PRICING SYSTEM COMPLETION (Weeks 5-7)
*Make pricing actually work*

### 3.1 Real Pricing Providers
- [x] **TASK-070**: Implement TCGPlayer API integration for Pokemon
  - API key configuration
  - Rate limiting (respect API limits)
  - Price parsing from response
  - Error handling and fallback
- [x] **TASK-071**: Implement PriceCharting API for Sports Cards
- [ ] **TASK-072**: Implement eBay sold listings scraper/API for Sports Cards
- [x] **TASK-073**: Fix Scryfall provider rate limiting (100ms delay) (Handled in ScryfallProvider)
- [x] **TASK-074**: Add retry logic with exponential backoff to all providers (Via sync logic)
- [ ] **TASK-075**: Implement provider health checking

### 3.2 Price Cache Improvements
- [ ] **TASK-076**: Add TTL-based cache eviction
- [ ] **TASK-077**: Add maximum cache size limit
- [ ] **TASK-078**: Implement LRU eviction policy
- [ ] **TASK-079**: Add cache statistics endpoint
- [ ] **TASK-080**: Implement cache persistence (survive restart)
- [ ] **TASK-081**: Add manual cache invalidation endpoint

### 3.3 Pricing Rules Enhancement
- [x] **TASK-082**: Add category precedence to pricing rules
- [x] **TASK-083**: Implement time-based rules (weekend bonuses)
- [x] **TASK-084**: Add customer tier-based pricing
- [x] **TASK-085**: Implement volume discounts
- [x] **TASK-086**: Add price history tracking
- [x] **TASK-087**: Create price trend endpoint

---

## PHASE 4: BARCODE & RECEIPT SYSTEM (Weeks 7-9)
*Essential retail operations*

### 4.1 Barcode Generation
- [x] **TASK-088**: Add barcode generation library (barcoders v2.0)
- [x] **TASK-089**: Create barcode generation endpoint `GET /api/inventory/barcode/:data`
- [ ] **TASK-090**: Implement Code 128 barcode generation (Done, covered by above)
- [ ] **TASK-091**: Implement QR code generation
- [ ] **TASK-092**: Add barcode to label template system
- [ ] **TASK-093**: Create bulk barcode generation endpoint

### 4.2 Barcode Scanning
- [x] **TASK-094**: Create `GET /api/products/barcode/:barcode` endpoint (Lookup logic)
- [ ] **TASK-095**: Implement UPC lookup integration
- [ ] **TASK-096**: Implement ISBN lookup integration
- [ ] **TASK-097**: Add barcode search to inventory lookup
- [ ] **TASK-098**: Create barcode scan logging for analytics

### 4.3 Receipt Generation
- [x] **TASK-099**: Create receipt template engine (HTML with Thermal CSS)
- [x] **TASK-100**: Implement `GET /api/transactions/:id/receipt` endpoint
- [x] **TASK-101**: Add store information to receipt header
- [x] **TASK-102**: Add line items with prices and quantities
- [x] **TASK-103**: Add tax breakdown to receipt
- [x] **TASK-104**: Add payment method information
- [x] **TASK-105**: Add return policy footer
- [x] **TASK-106**: Implement thermal printer format (80mm width)
- [x] **TASK-107**: Add receipt reprint capability (via GET)

### 4.4 Invoice Generation
- [ ] **TASK-108**: Create invoice PDF template
- [ ] **TASK-109**: Implement invoice generation endpoint
- [ ] **TASK-110**: Add business information header
- [ ] **TASK-111**: Add customer billing information
- [ ] **TASK-112**: Support email delivery of invoices

---

## PHASE 5: MULTI-TERMINAL SYNC FIX (Weeks 9-11)
*Make "offline-first" actually work*

### 5.1 Network Discovery Fix
- [x] **TASK-113**: Implement real mDNS device discovery (Uses `local-ip-address` and `mdns-sd`)
- [x] **TASK-114**: Add device registration on startup
- [ ] **TASK-115**: Implement device heartbeat/keepalive
- [ ] **TASK-116**: Add device disconnect detection
- [ ] **TASK-117**: Create discovered devices API endpoint
- [ ] **TASK-118**: Implement manual device pairing fallback

### 5.2 Sync Protocol Improvements
- [ ] **TASK-119**: Implement proper vector clock comparison
- [ ] **TASK-120**: Add conflict detection (not just empty placeholder)
- [ ] **TASK-121**: Create conflict resolution UI API
- [ ] **TASK-122**: Implement three-way merge for non-conflicting fields
- [ ] **TASK-123**: Add sync checksum verification
- [ ] **TASK-124**: Implement sync batch size limits
- [ ] **TASK-125**: Add sync progress reporting

### 5.3 Offline Queue
- [ ] **TASK-126**: Create offline operation queue table
- [ ] **TASK-127**: Implement operation queuing when offline
- [ ] **TASK-128**: Add queue processing on reconnection
- [ ] **TASK-129**: Implement retry with exponential backoff
- [ ] **TASK-130**: Add failed operation notification
- [ ] **TASK-131**: Create queue management API

### 5.4 Frontend Offline Support
- [ ] **TASK-132**: Add local SQLite database to Flutter app
- [ ] **TASK-133**: Implement offline operation queuing in frontend
- [ ] **TASK-134**: Add sync status indicator to UI
- [ ] **TASK-135**: Implement background sync service
- [ ] **TASK-136**: Add conflict resolution UI

---

## PHASE 6: CASH DRAWER & HARDWARE (Weeks 11-12)
*Physical retail integration*

### 6.1 Cash Drawer Integration
- [ ] **TASK-137**: Create cash drawer service interface
- [ ] **TASK-138**: Implement `POST /api/cash-drawer/open` endpoint
- [ ] **TASK-139**: Implement `POST /api/cash-drawer/count` endpoint
- [ ] **TASK-140**: Add cash drawer kick on sale completion
- [ ] **TASK-141**: Create cash counting workflow
- [ ] **TASK-142**: Implement till reconciliation
- [ ] **TASK-143**: Add shift open/close cash counts
- [ ] **TASK-144**: Create cash variance reporting

### 6.2 Printer Integration
- [ ] **TASK-145**: Create printer service interface
- [ ] **TASK-146**: Implement ESC/POS command generation
- [ ] **TASK-147**: Add thermal printer discovery
- [ ] **TASK-148**: Implement label printer support
- [ ] **TASK-149**: Add print queue management

---

## PHASE 7: ADVANCED FEATURES (Weeks 12-16)

### 7.1 Layaway/Hold System
- [ ] **TASK-150**: Implement hold creation with deposit
- [ ] **TASK-151**: Add hold expiration tracking
- [ ] **TASK-152**: Create hold payment schedule
- [ ] **TASK-153**: Implement hold cancellation with refund logic
- [ ] **TASK-154**: Add hold pickup workflow
- [ ] **TASK-155**: Create hold notification reminders

### 7.2 Serialized Inventory
- [ ] **TASK-156**: Implement serial number tracking
- [ ] **TASK-157**: Add grading information fields
- [ ] **TASK-158**: Create certificate/COA tracking
- [ ] **TASK-159**: Implement individual item pricing
- [ ] **TASK-160**: Add serial number search
- [ ] **TASK-161**: Create serialized item sale workflow

### 7.3 Trade-In Fraud Protection
- [ ] **TASK-162**: Implement customer trade-in limits
- [ ] **TASK-163**: Add trade-in velocity tracking
- [ ] **TASK-164**: Create trade-in blacklist
- [ ] **TASK-165**: Add ID verification requirement for high-value trades
- [ ] **TASK-166**: Implement trade-in hold period
- [ ] **TASK-167**: Create suspicious activity alerts

### 7.4 Return Processing Enhancement
- [ ] **TASK-168**: Add restocking fee configuration
- [ ] **TASK-169**: Implement partial returns
- [ ] **TASK-170**: Add return reason codes
- [ ] **TASK-171**: Create damaged return workflow
- [ ] **TASK-172**: Implement return limits per customer
- [ ] **TASK-173**: Add return authorization for high-value items

### 7.5 Multi-Location Support
- [ ] **TASK-174**: Implement location-based inventory views
- [ ] **TASK-175**: Create inventory transfer workflow
- [ ] **TASK-176**: Add transfer request/approval system
- [ ] **TASK-177**: Implement in-transit inventory tracking
- [ ] **TASK-178**: Create location-based reporting
- [ ] **TASK-179**: Add inter-location price variations

---

## PHASE 8: REPORTING & ANALYTICS (Weeks 16-18)

### 8.1 Complete Reports Implementation
- [ ] **TASK-180**: Implement sales by category breakdown
- [ ] **TASK-181**: Implement sales by day/week/month trends
- [ ] **TASK-182**: Add inventory valuation by category
- [ ] **TASK-183**: Create tax summary report
- [ ] **TASK-184**: Implement profit margin report
- [ ] **TASK-185**: Add customer purchase history report
- [ ] **TASK-186**: Create price change history report
- [ ] **TASK-187**: Implement aging inventory report

### 8.2 Dashboard Enhancements
- [ ] **TASK-188**: Add real-time sales ticker
- [ ] **TASK-189**: Implement comparison with previous period
- [ ] **TASK-190**: Add goal tracking widgets
- [ ] **TASK-191**: Create customizable dashboard layout

---

## PHASE 9: NOTIFICATIONS & COMMUNICATION (Weeks 18-20)

### 9.1 Email System
- [ ] **TASK-192**: Add email service integration (SendGrid/SES)
- [ ] **TASK-193**: Create receipt email template
- [ ] **TASK-194**: Implement wants list match notifications
- [ ] **TASK-195**: Add event reminder emails
- [ ] **TASK-196**: Create trade-in quote emails

### 9.2 SMS System
- [ ] **TASK-197**: Add SMS service integration (Twilio)
- [ ] **TASK-198**: Implement order ready notifications
- [ ] **TASK-199**: Add hold expiration reminders
- [ ] **TASK-200**: Create event reminder texts

---

## PHASE 10: MONITORING & OBSERVABILITY (Weeks 20-22)

### 10.1 Logging Enhancement
- [ ] **TASK-201**: Implement structured JSON logging
- [ ] **TASK-202**: Add request ID correlation
- [ ] **TASK-203**: Create audit log for all data modifications
- [ ] **TASK-204**: Implement log rotation and retention

### 10.2 Metrics & Monitoring
- [ ] **TASK-205**: Add Prometheus metrics endpoint
- [ ] **TASK-206**: Implement request latency histograms
- [ ] **TASK-207**: Add database connection pool metrics
- [ ] **TASK-208**: Create sync queue depth metrics
- [ ] **TASK-209**: Implement business metrics (daily sales, etc.)

### 10.3 Health Checks
- [ ] **TASK-210**: Add database connectivity check
- [ ] **TASK-211**: Add disk space check
- [ ] **TASK-212**: Add sync service status check
- [ ] **TASK-213**: Implement external service checks (pricing APIs)
- [ ] **TASK-214**: Create comprehensive `/health/detailed` endpoint

### 10.4 Alerting
- [ ] **TASK-215**: Implement error rate alerting
- [ ] **TASK-216**: Add sync failure alerts
- [ ] **TASK-217**: Create low disk space alerts
- [ ] **TASK-218**: Implement database connection exhaustion alerts

---

## PHASE 11: BACKUP & DISASTER RECOVERY (Weeks 22-23)

### 11.1 Backup System
- [ ] **TASK-219**: Implement automated SQLite backup
- [ ] **TASK-220**: Add backup to cloud storage (S3/GCS)
- [ ] **TASK-221**: Implement point-in-time recovery
- [ ] **TASK-222**: Create backup verification system
- [ ] **TASK-223**: Add backup schedule configuration
- [ ] **TASK-224**: Implement backup rotation/retention

### 11.2 Recovery Procedures
- [ ] **TASK-225**: Document recovery procedures
- [ ] **TASK-226**: Create restore script
- [ ] **TASK-227**: Implement restore testing automation
- [ ] **TASK-228**: Add disaster recovery runbook

---

## PHASE 12: TESTING & QUALITY (Weeks 23-26)

### 12.1 Unit Tests
- [ ] **TASK-229**: Add tests for all repository methods
- [ ] **TASK-230**: Add tests for all services
- [ ] **TASK-231**: Add tests for pricing providers (with mocks)
- [ ] **TASK-232**: Add tests for sync logic
- [ ] **TASK-233**: Add tests for tax calculations
- [ ] **TASK-234**: Achieve 70%+ code coverage

### 12.2 Integration Tests
- [ ] **TASK-235**: Create API endpoint tests
- [ ] **TASK-236**: Add database migration tests
- [ ] **TASK-237**: Implement sync integration tests
- [ ] **TASK-238**: Create end-to-end transaction tests

### 12.3 Load Testing
- [ ] **TASK-239**: Set up load testing framework
- [ ] **TASK-240**: Test concurrent transaction handling
- [ ] **TASK-241**: Test sync under high load
- [ ] **TASK-242**: Establish performance baselines

### 12.4 Security Testing
- [ ] **TASK-243**: Run SQL injection tests
- [ ] **TASK-244**: Test authentication bypass attempts
- [ ] **TASK-245**: Verify rate limiting effectiveness
- [ ] **TASK-246**: Test CORS policy enforcement

---

## PHASE 13: DOCUMENTATION (Ongoing)

### 13.1 User Documentation
- [ ] **TASK-247**: Create user manual
- [ ] **TASK-248**: Write quick start guide
- [ ] **TASK-249**: Document all features with screenshots
- [ ] **TASK-250**: Create video tutorials

### 13.2 Developer Documentation
- [ ] **TASK-251**: Document architecture
- [ ] **TASK-252**: Create API reference
- [ ] **TASK-253**: Write deployment guide
- [ ] **TASK-254**: Document database schema (ER diagram)
- [ ] **TASK-255**: Create contribution guidelines

### 13.3 Operations Documentation
- [ ] **TASK-256**: Write runbooks for common issues
- [ ] **TASK-257**: Document backup procedures
- [ ] **TASK-258**: Create incident response procedures
- [ ] **TASK-259**: Document monitoring and alerting

---

## PHASE 14: DEPLOYMENT PREPARATION (Weeks 26-28)

### 14.1 Deployment Infrastructure
- [ ] **TASK-260**: Create production Dockerfile
- [ ] **TASK-261**: Set up CI/CD pipeline
- [ ] **TASK-262**: Create deployment scripts
- [ ] **TASK-263**: Set up staging environment
- [ ] **TASK-264**: Implement blue-green deployment

### 14.2 Migration Safety
- [ ] **TASK-265**: Add migration dry-run capability
- [ ] **TASK-266**: Implement migration rollback scripts
- [ ] **TASK-267**: Create migration testing in CI
- [ ] **TASK-268**: Document migration procedures

### 14.3 Pre-Launch Checklist
- [ ] **TASK-269**: Complete security audit
- [ ] **TASK-270**: Performance test sign-off
- [ ] **TASK-271**: Documentation review
- [ ] **TASK-272**: Backup system verification
- [ ] **TASK-273**: Monitoring/alerting verification
- [ ] **TASK-274**: Load testing sign-off

---

## PRIORITY MATRIX

### P0 - Critical (Blocks Everything)
TASK-001 through TASK-018 (Security & Config)
TASK-046 through TASK-058 (Tax & Payment basics)
TASK-070 through TASK-075 (Real pricing providers)

### P1 - High (Core Functionality)
TASK-019 through TASK-045 (Database foundation)
TASK-088 through TASK-107 (Barcode & Receipt)
TASK-113 through TASK-136 (Sync fixes)

### P2 - Medium (Important Features)
TASK-137 through TASK-149 (Cash drawer & hardware)
TASK-150 through TASK-179 (Advanced features)
TASK-180 through TASK-191 (Reporting)

### P3 - Lower (Enhancement)
TASK-192 through TASK-200 (Notifications)
TASK-201 through TASK-228 (Monitoring & Backup)
TASK-229 through TASK-274 (Testing & Deployment)

---

## DEPENDENCY GRAPH

```
Phase 0 (Security)
    ↓
Phase 1 (Database) ──────────────────────────┐
    ↓                                        │
Phase 2 (Business Logic) ←───────────────────┤
    ↓                                        │
Phase 3 (Pricing) ←──────────────────────────┤
    ↓                                        │
Phase 4 (Barcode/Receipt) ←──────────────────┤
    ↓                                        │
Phase 5 (Sync) ←─────────────────────────────┘
    ↓
Phase 6 (Hardware) ──→ Phase 7 (Advanced Features)
                              ↓
                       Phase 8 (Reporting)
                              ↓
                       Phase 9 (Notifications)
                              ↓
                       Phase 10 (Monitoring)
                              ↓
                       Phase 11 (Backup)
                              ↓
                       Phase 12 (Testing)
                              ↓
                       Phase 14 (Deployment)

Phase 13 (Documentation) runs parallel throughout
```

---

## RESOURCE ALLOCATION RECOMMENDATION

| Phase | Duration | Developers | Focus |
|-------|----------|------------|-------|
| 0 | 1 week | 1 | Security/Config |
| 1 | 2 weeks | 2 | Database (parallel work) |
| 2 | 2 weeks | 2 | Business Logic |
| 3 | 2 weeks | 1 | Pricing APIs |
| 4 | 2 weeks | 1 | Barcode/Receipt |
| 5 | 2 weeks | 2 | Sync (complex) |
| 6 | 1 week | 1 | Hardware |
| 7 | 4 weeks | 2 | Advanced Features |
| 8 | 2 weeks | 1 | Reporting |
| 9 | 2 weeks | 1 | Notifications |
| 10 | 2 weeks | 1 | Monitoring |
| 11 | 1 week | 1 | Backup |
| 12 | 3 weeks | 2 | Testing |
| 14 | 2 weeks | 1 | Deployment |

**TOTAL: ~26-28 weeks with 2-3 developers**

---

## QUICK START - FIRST WEEK TASKS

Execute in this exact order:

1. **TASK-001**: Remove JWT secret from `.env`
2. **TASK-002**: Create `.env.example`
3. **TASK-015**: Fix frontend hardcoded URL
4. **TASK-010**: Fix NODE_ID generation
5. **TASK-006**: Fix CORS configuration
6. **TASK-004**: Add env var validation
7. **TASK-014**: Create environment profiles

This makes the system *deployable* even if not fully featured.
