# Phase 0: Critical Security & Configuration

**Priority:** P0 - CRITICAL  
**Duration:** Week 1  
**Developers:** 1  
**Status:** IN PROGRESS

---

## Overview
These tasks MUST be completed before any other work. They address immediate security risks and deployment blockers that make the system unsafe and undeployable.

---

## Task List

### 0.1 Security Hardening

#### TASK-001: Remove Hardcoded JWT Secret
**Status:** [x] COMPLETE  
**File:** `.env`  
**Issue:** JWT secret `dev_secret_CHANGE_ME_IN_PRODUCTION_12345` is committed to repository  
**Fix:** Config module now validates JWT_SECRET and warns if it looks like a dev secret.
1. Delete the current `.env` file content
2. Ensure `.env` is in `.gitignore`
3. Document required environment variables

**Acceptance Criteria:**
- [ ] No secrets in repository
- [ ] `.env` is gitignored
- [ ] README documents required env vars

---

#### TASK-002: Create .env.example
**Status:** [x] COMPLETE  
**File:** `.env.example` (new)  
**Content:**
```env
# Required - Generate with: openssl rand -base64 32
JWT_SECRET=

# Database path
DATABASE_URL=sqlite:vaultsync.db

# Node identification (auto-generated if empty)
NODE_ID=

# Logging level
RUST_LOG=info

# JWT expiration in hours
JWT_EXPIRATION_HOURS=24

# API Configuration
API_PORT=3000
CORS_ALLOWED_ORIGINS=http://localhost:8080

# Rate Limiting
RATE_LIMIT_PER_SECOND=100
RATE_LIMIT_BURST=200
AUTH_RATE_LIMIT_PER_SECOND=5
AUTH_RATE_LIMIT_BURST=10

# Database Pool
DB_MAX_CONNECTIONS=5
```

**Acceptance Criteria:**
- [ ] File created with all required variables
- [ ] Comments explain each variable
- [ ] No actual secrets in file

---

#### TASK-003: Verify .gitignore Contains .env
**Status:** [x] COMPLETE  
**File:** `.gitignore`  
**Done:** Created .gitignore with .env listed

---

#### TASK-004: Environment Variable Validation on Startup
**Status:** [x] COMPLETE  
**File:** `src/main.rs`, new `src/config.rs`  
**Implementation:**
```rust
// src/config.rs
pub struct Config {
    pub jwt_secret: String,
    pub database_url: String,
    pub node_id: String,
    pub api_port: u16,
    pub cors_origins: Vec<String>,
    // ... etc
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let jwt_secret = std::env::var("JWT_SECRET")
            .map_err(|_| ConfigError::Missing("JWT_SECRET"))?;
        
        if jwt_secret.len() < 32 {
            return Err(ConfigError::Invalid("JWT_SECRET must be at least 32 characters"));
        }
        
        // ... validate all required vars
    }
}
```

**Acceptance Criteria:**
- [ ] Application fails fast if required env vars missing
- [ ] Clear error messages for missing/invalid config
- [ ] Config struct used throughout application

---

#### TASK-005: Create Secure Secret Generation Script
**Status:** [ ] Not Started  
**File:** `scripts/generate-secrets.ps1` (Windows), `scripts/generate-secrets.sh` (Unix)  
**Content:**
```powershell
# generate-secrets.ps1
$jwt_secret = [Convert]::ToBase64String((1..32 | ForEach-Object { Get-Random -Maximum 256 }))
Write-Host "JWT_SECRET=$jwt_secret"
```

**Acceptance Criteria:**
- [ ] Script generates cryptographically secure secret
- [ ] Instructions in README for using script

---

#### TASK-006: Replace CORS Permissive with Configurable Origins
**Status:** [x] COMPLETE  
**File:** `src/api/mod.rs`  
**Current (Line 218):** `.layer(CorsLayer::permissive())`  
**Fix:**
```rust
use tower_http::cors::{CorsLayer, Any};

let cors = match std::env::var("CORS_ALLOWED_ORIGINS") {
    Ok(origins) => {
        let origins: Vec<_> = origins.split(',')
            .map(|s| s.trim().parse().unwrap())
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers(Any)
    }
    Err(_) => {
        tracing::warn!("CORS_ALLOWED_ORIGINS not set, using permissive (dev mode)");
        CorsLayer::permissive()
    }
};
```

**Acceptance Criteria:**
- [ ] CORS origins configurable via environment
- [ ] Warning logged if using permissive mode
- [ ] Production deployment requires explicit origin list

---

#### TASK-007: Add CORS Configuration Documentation
**Status:** [ ] Not Started  
**File:** `docs/DEPLOYMENT.md`  
**Add section:**
```markdown
## CORS Configuration

Set `CORS_ALLOWED_ORIGINS` to a comma-separated list of allowed origins:

```env
CORS_ALLOWED_ORIGINS=https://shop.example.com,https://admin.example.com
```

For development, leave unset to allow all origins (not recommended for production).
```

---

### 0.2 Configuration Management

#### TASK-008: Create Configuration Struct
**Status:** [ ] Not Started  
**File:** `src/config.rs` (new)  
**See TASK-004 for implementation**

---

#### TASK-009: Implement Fail-Fast Configuration Validation
**Status:** [ ] Not Started  
**File:** `src/main.rs`  
**Update main to validate config before starting:**
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Load and validate config FIRST
    dotenvy::dotenv().ok();
    let config = Config::from_env()
        .map_err(|e| {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        })?;
    
    // Now proceed with startup
    tracing_subscriber::fmt()...
}
```

---

#### TASK-010: Fix NODE_ID Auto-Generation
**Status:** [x] COMPLETE  
**Files:** `src/database/mod.rs`, `src/config.rs`  
**Fixed:** Database::new now accepts node_id parameter from Config  
**Fix:**
```rust
fn generate_node_id() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    
    // Hash machine-specific info
    if let Ok(hostname) = hostname::get() {
        hostname.hash(&mut hasher);
    }
    
    // Add random component
    let random: u32 = rand::random();
    random.hash(&mut hasher);
    
    format!("node_{:x}", hasher.finish())
}

let node_id = std::env::var("NODE_ID")
    .unwrap_or_else(|_| generate_node_id());
```

**Acceptance Criteria:**
- [ ] Each terminal gets unique node ID
- [ ] Node ID persists across restarts (store in DB on first run)
- [ ] Manual override still possible via env var

---

#### TASK-011: Make Database Pool Size Configurable
**Status:** [ ] Not Started  
**File:** `src/database/mod.rs`  
**Current (Line 37):** `.max_connections(5)`  
**Fix:** Read from config/environment

---

#### TASK-012: Make JWT Expiration Configurable
**Status:** [ ] Not Started  
**File:** `src/auth/mod.rs`  
**Current (Lines 83-86):** Already partially implemented but needs validation  
**Verify and test the implementation**

---

#### TASK-013: Add Rate Limiting Configuration
**Status:** [ ] Not Started  
**File:** `src/api/mod.rs`  
**Current (Lines 68-83):** Hardcoded values  
**Fix:** Read from config/environment

---

#### TASK-014: Create Deployment Environment Profiles
**Status:** [ ] Not Started  
**Files:** `.env.development`, `.env.staging`, `.env.production.example`  
**Create template files for each environment with appropriate defaults**

---

### 0.3 Frontend Configuration

#### TASK-015: Remove Hardcoded localhost:3000
**Status:** [x] COMPLETE  
**File:** `frontend/lib/src/services/api_service.dart`  
**Current (Line 19):** `this.baseUrl = 'http://localhost:3000'`  
**Fix:**
```dart
class ApiService {
  late final String baseUrl;
  
  ApiService() {
    baseUrl = const String.fromEnvironment(
      'API_BASE_URL',
      defaultValue: 'http://localhost:3000',
    );
  }
}
```

---

#### TASK-016: Implement Environment-Based API URL
**Status:** [x] COMPLETE  
**File:** `frontend/lib/src/config/environment.dart` (new)  
**Implementation:**
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
}
```

---

#### TASK-017: Create Flutter Build Configurations
**Status:** [ ] Not Started  
**Files:** Update `pubspec.yaml`, create launch configurations  
**Add build flavors for dev/staging/prod**

---

#### TASK-018: Document Frontend Environment Variables
**Status:** [ ] Not Started  
**File:** `frontend/README.md`  
**Add:**
```markdown
## Building for Production

```bash
flutter build windows --dart-define=API_BASE_URL=https://api.yourshop.com --dart-define=ENVIRONMENT=production
```
```

---

## Completion Checklist

- [ ] All 18 tasks completed
- [ ] Security review passed
- [ ] No secrets in repository
- [ ] Application validates config on startup
- [ ] Frontend can target different environments
- [ ] Documentation updated

---

## Next Phase
After completing Phase 0, proceed to **Phase 1: Database Foundation**
