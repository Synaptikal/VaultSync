# VaultSync Remediation: Quick Start Guide

**Date:** 2026-01-02  
**Purpose:** Get started immediately with the most critical fixes
**Last Updated:** 2026-01-02  
**Status:** ✅ PHASE 0-2 COMPLETE | 🟨 PHASE 3-5 IN PROGRESS

---

## ✅ IMPLEMENTATION COMPLETE

The following sections describe what was planned. **All Week 1 priorities have been implemented.**

### Completed Items:
- ✅ JWT Secret handling with validation
- ✅ `.env.example` created with comprehensive documentation  
- ✅ `.gitignore` configured
- ✅ Config module (`src/config.rs`) with fail-fast validation
- ✅ Node ID auto-generation (unique per instance)
- ✅ CORS configuration (environment-based)
- ✅ Frontend `Environment.dart` configuration
- ✅ Rate limiting configuration
- ✅ Database migrations (14-19) adding business tables
- ✅ TaxService implementation
- ✅ PaymentService implementation  
- ✅ HoldsService (layaway) implementation
- ✅ TransactionValidationService implementation (Atomic)
- ✅ API endpoints for Tax and Holds
- ✅ Pokemon TCG real API integration
- ✅ **Barcode System (Phase 4)**: Generation & Scanning
- ✅ **Network Discovery (Phase 5)**: Real mDNS & Sync Triggering

### Build Status:
```
cargo build --release  ✅ SUCCESS
cargo check            ✅ SUCCESS  
```

---

## 🚨 ORIGINAL PLAN - WEEK 1 PRIORITIES (COMPLETED)

Execute these tasks in order. Each one unblocks the next.

---

## Day 1: Security Fixes (4-6 hours) ✅ DONE

### Step 1: Remove Hardcoded JWT Secret ✅
```powershell
# 1. First, add .env to .gitignore if not present
Add-Content -Path "d:\Projects\VaultSync\.gitignore" -Value "`n.env"

# 2. Delete the compromised .env
Remove-Item "d:\Projects\VaultSync\.env"
```

### Step 2: Create .env.example ✅
**Already Created - see `d:\Projects\VaultSync\.env.example`**

### Step 3: Create local development .env
Create `d:\Projects\VaultSync\.env` (this won't be committed):
```env
JWT_SECRET=dev_local_secret_minimum_32_characters_long_here
DATABASE_URL=sqlite:vaultsync.db
NODE_ID=dev_node_001
RUST_LOG=debug
```

### Step 4: Verify .gitignore
Confirm `.env` is ignored:
```powershell
git status
# .env should NOT appear in untracked files
```

---

## Day 1-2: Configuration Validation (4-6 hours)

### Step 5: Create Config Module
Create file `d:\Projects\VaultSync\src\config.rs`:
```rust
use crate::errors::{Result, VaultSyncError};

#[derive(Debug, Clone)]
pub struct Config {
    pub jwt_secret: String,
    pub database_url: String,
    pub node_id: String,
    pub api_port: u16,
    pub cors_origins: Option<Vec<String>>,
    pub jwt_expiration_hours: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // JWT Secret - REQUIRED
        let jwt_secret = std::env::var("JWT_SECRET")
            .map_err(|_| VaultSyncError::ConfigError(
                "JWT_SECRET environment variable is required".into()
            ))?;
        
        if jwt_secret.len() < 32 {
            return Err(VaultSyncError::ConfigError(
                "JWT_SECRET must be at least 32 characters".into()
            ));
        }
        
        // Database URL
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:vaultsync.db".to_string());
        
        // Node ID - generate if not provided
        let node_id = std::env::var("NODE_ID")
            .unwrap_or_else(|_| generate_node_id());
        
        // API Port
        let api_port = std::env::var("API_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);
        
        // CORS origins
        let cors_origins = std::env::var("CORS_ALLOWED_ORIGINS")
            .ok()
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect());
        
        // JWT expiration
        let jwt_expiration_hours = std::env::var("JWT_EXPIRATION_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse()
            .unwrap_or(24);
        
        Ok(Self {
            jwt_secret,
            database_url,
            node_id,
            api_port,
            cors_origins,
            jwt_expiration_hours,
        })
    }
}

fn generate_node_id() -> String {
    use rand::Rng;
    let random: u64 = rand::thread_rng().gen();
    format!("node_{:x}", random)
}
```

### Step 6: Add Config to lib.rs
Edit `d:\Projects\VaultSync\src\lib.rs`, add:
```rust
pub mod config;
pub use config::Config;
```

### Step 7: Update main.rs to Use Config
Edit `d:\Projects\VaultSync\src\main.rs`:
```rust
use vaultsync::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file
    dotenvy::dotenv().ok();
    
    // Validate configuration FIRST (fail fast)
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            eprintln!("\nPlease ensure all required environment variables are set.");
            eprintln!("See .env.example for required variables.");
            std::process::exit(1);
        }
    };
    
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    tracing::info!("VaultSync starting with node_id: {}", config.node_id);
    
    // ... rest of initialization using config
}
```

### Step 8: Add ConfigError to errors
Edit `d:\Projects\VaultSync\src\errors\mod.rs`, add variant:
```rust
#[derive(Debug, Error)]
pub enum VaultSyncError {
    // ... existing variants
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}
```

---

## Day 2-3: Fix Frontend Hardcoded URL (2-4 hours)

### Step 9: Create Environment Config for Flutter
Create `d:\Projects\VaultSync\frontend\lib\src\config\environment.dart`:
```dart
class Environment {
  static const String apiBaseUrl = String.fromEnvironment(
    'API_BASE_URL',
    defaultValue: 'http://localhost:3000',
  );
  
  static const bool isProduction = String.fromEnvironment(
    'ENVIRONMENT',
    defaultValue: 'development',
  ) == 'production';
  
  static const bool enableLogging = !isProduction;
}
```

### Step 10: Update ApiService
Edit `d:\Projects\VaultSync\frontend\lib\src\services\api_service.dart`:
```dart
import '../config/environment.dart';

class ApiService {
  final Dio _dio;
  
  ApiService() : _dio = Dio(BaseOptions(
    baseUrl: Environment.apiBaseUrl,
    connectTimeout: const Duration(seconds: 10),
    receiveTimeout: const Duration(seconds: 10),
  )) {
    if (Environment.enableLogging) {
      _dio.interceptors.add(LogInterceptor());
    }
  }
  
  // ... rest of implementation
}
```

### Step 11: Update Frontend README
Add to `d:\Projects\VaultSync\frontend\README.md`:
```markdown
## Building for Different Environments

### Development (default)
```bash
flutter run
```

### Production
```bash
flutter build windows --dart-define=API_BASE_URL=https://api.yourshop.com --dart-define=ENVIRONMENT=production
```
```

---

## Day 3: Fix Node ID Generation (2 hours)

### Step 12: Update Database Module
Edit `d:\Projects\VaultSync\src\database\mod.rs`, change line ~51:

**Before:**
```rust
let node_id = std::env::var("NODE_ID").unwrap_or_else(|_| "node_001".to_string());
```

**After:**
```rust
// Node ID is now injected from Config
// This change requires updating the Database::new signature
```

Better approach - pass node_id to Database::new:
```rust
impl Database {
    pub async fn new(connection_string: &str, node_id: String) -> Result<Self> {
        // ...
        Ok(Self {
            // ...
            node_id,
        })
    }
}
```

Update main.rs to pass config.node_id.

---

## Day 4: Fix CORS (2 hours)

### Step 13: Update CORS Configuration
Edit `d:\Projects\VaultSync\src\api\mod.rs`:

**Before (line 218):**
```rust
.layer(CorsLayer::permissive())
```

**After:**
```rust
use tower_http::cors::{CorsLayer, Any, AllowOrigin};
use axum::http::Method;

// Build CORS layer based on configuration
fn build_cors_layer(origins: Option<Vec<String>>) -> CorsLayer {
    match origins {
        Some(origins) if !origins.is_empty() => {
            let origins: Vec<_> = origins
                .iter()
                .filter_map(|s| s.parse().ok())
                .collect();
            
            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
                .allow_headers(Any)
                .allow_credentials(true)
        }
        _ => {
            tracing::warn!("CORS_ALLOWED_ORIGINS not configured - using permissive mode (not recommended for production)");
            CorsLayer::permissive()
        }
    }
}

// In create_router:
.layer(build_cors_layer(config.cors_origins))
```

This requires passing config to create_router.

---

## Day 5: Compile and Test (4 hours)

### Step 14: Build and Verify
```powershell
cd d:\Projects\VaultSync

# Create local .env if not exists
if (-not (Test-Path .env)) {
    Copy-Item .env.example .env
    # Edit .env and add JWT_SECRET
}

# Build
cargo build

# Run tests
cargo test

# Start server
cargo run
```

### Step 15: Verify Configuration Validation
Test that the application fails appropriately:
```powershell
# Remove JWT_SECRET and try to start
$env:JWT_SECRET = ""
cargo run
# Should fail with clear error message
```

### Step 16: Test Frontend Connection
```powershell
cd d:\Projects\VaultSync\frontend
flutter run -d windows
# Should connect to localhost:3000
```

---

## Week 1 Completion Checklist

- [ ] `.env` removed from git history (or at minimum, not tracked)
- [ ] `.env.example` created with all variables documented
- [ ] Config module validates all required settings
- [ ] Application fails fast with clear error if config invalid
- [ ] Node ID auto-generates unique value
- [ ] Frontend uses environment-based API URL
- [ ] CORS is configurable
- [ ] Application builds and runs
- [ ] Tests pass

---

## What's Next?

After Week 1, proceed to:

1. **Phase 1: Database Foundation** (Weeks 2-3)
   - Add missing tables
   - Add missing columns
   - Create new repositories
   
2. **Phase 2: Core Business Logic** (Weeks 3-5)
   - Tax calculation
   - Payment processing
   - Transaction validation

See `MASTER_REMEDIATION_PLAN.md` for full roadmap.

---

## Files Modified in Week 1

| File | Action |
|------|--------|
| `.gitignore` | Add .env |
| `.env` | Delete and recreate (not tracked) |
| `.env.example` | Create new |
| `src/config.rs` | Create new |
| `src/lib.rs` | Add config module |
| `src/main.rs` | Use Config, fail fast |
| `src/errors/mod.rs` | Add ConfigError |
| `src/database/mod.rs` | Accept node_id parameter |
| `src/api/mod.rs` | Configurable CORS |
| `frontend/lib/src/config/environment.dart` | Create new |
| `frontend/lib/src/services/api_service.dart` | Use Environment |
| `frontend/README.md` | Add build instructions |

---

## Estimated Time: 20-30 hours

This can be completed by one developer in one week, working part-time, or 3-4 days full-time.
