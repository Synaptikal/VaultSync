---
description: Frontend development workflow for VaultSync cross-platform app (Flutter)
---

# Frontend Development Workflow

This workflow documents how to develop, build, and test the VaultSync Flutter frontend across platforms.

## Tech Stack
- **Framework**: Flutter 3.38.5 (Dart 3.10.4)
- **State Management**: Provider
- **Routing**: go_router
- **Local DB**: sqflite / sqflite_common_ffi (desktop)
- **HTTP Client**: http package
- **API Models**: Auto-generated from OpenAPI spec via `swagger_parser`

## Supported Platforms
- ✅ Windows (Desktop)
- ✅ Android (Phone/Tablet)
- ✅ iOS (Phone/Tablet)
- ✅ macOS (Desktop)
- ✅ Linux (Desktop)
- ✅ Web (PWA)

## Development Setup

### Prerequisites
1. Flutter SDK installed and in PATH
2. Rust backend running (`cargo run` in project root)
3. For mobile: Android Studio / Xcode with emulators configured

### Initial Setup
```powershell
cd frontend
flutter pub get
dart run swagger_parser
dart run build_runner build --delete-conflicting-outputs
```

### Regenerating API Models
Whenever the backend OpenAPI spec changes:
```powershell
# From project root
cargo run --bin export_spec

# From frontend directory  
cd frontend
dart run swagger_parser
dart run build_runner build --delete-conflicting-outputs
```

## Running the App

### Windows Desktop
// turbo
```powershell
cd frontend
flutter run -d windows
```

### Android
// turbo
```powershell
flutter run -d android
```

### iOS (requires macOS)
```bash
flutter run -d ios
```

### Web
// turbo
```powershell
flutter run -d chrome
```

## Building for Release

### Windows
// turbo
```powershell
flutter build windows --release
```
Output: `build/windows/x64/runner/Release/`

### Android APK
// turbo
```powershell
flutter build apk --release
```

### Android App Bundle (Play Store)
// turbo
```powershell
flutter build appbundle --release
```

### iOS (requires macOS)
```bash
flutter build ios --release
```

### Web
// turbo
```powershell
flutter build web --release
```

## Project Structure
```
frontend/
├── lib/
│   ├── main.dart                 # App entry point, provider setup
│   └── src/
│       ├── api/
│       │   └── generated/        # Auto-generated models from OpenAPI
│       ├── features/             # Feature-based screens
│       │   ├── authentication/
│       │   ├── customers/
│       │   ├── dashboard/
│       │   ├── events/           # NEW: Event management
│       │   ├── inventory/
│       │   ├── pos/
│       │   ├── pricing/
│       │   ├── reports/
│       │   └── wants/            # NEW: Wants list management
│       ├── models/               # Local/custom models
│       ├── providers/            # State management
│       ├── services/             # API & storage services
│       └── shared/               # Common widgets, theme
├── windows/                      # Windows native code
├── android/                      # Android native code
├── ios/                          # iOS native code (if present)
└── web/                          # Web configuration
```

## Key Features Implemented
- [x] Authentication (JWT)
- [x] Dashboard with stats & market pulse
- [x] Inventory Management (List, Matrix, Bulk)
- [x] Point of Sale with barcode scanning
- [x] Customer Management with credit system
- [x] Pricing Dashboard with market trends
- [x] Reports
- [x] **Events Screen** with participant registration
- [x] **Wants List Screen** with customer matching
- [x] **Notifications** with wants match alerts
- [x] **Pricing Rules** management (Admin)
- [x] **Serialized/Graded Inventory** dialog
- [x] **Offline-first sync** with connectivity monitoring

## Offline-First Sync Architecture
The app uses a robust offline-first sync strategy:

### Key Components
- `OfflineSyncService` - Manages connectivity, queuing, and syncing
- `SyncStatusWidget` - Shows sync status and pending changes
- `LocalStorageService` - SQLite-based local storage with change tracking

### Sync Behavior
1. **Queue Changes**: When offline, changes are queued locally
2. **Auto-Detect**: Connectivity is checked every 30 seconds
3. **Auto-Sync**: When back online, pending changes sync automatically
4. **Background Sync**: Every 5 minutes, the app checks for sync needs
5. **Conflict Resolution**: Last-write-wins (server version takes precedence)

### Sync Status UI
- **Dashboard**: Shows sync widget when offline/pending
- **Admin > Sync Configuration**: Full sync management
- **App Bar**: Compact sync indicator on all screens

## Backend API Connection
The app connects to the Rust backend API. Configure the base URL in:
- `lib/src/services/api_service.dart` → `baseUrl` parameter (default: `http://localhost:3000`)

For production, update this or use environment configuration.

## Responsive Design
The app adapts between desktop and mobile layouts:
- **Desktop (>800px)**: NavigationRail sidebar
- **Mobile/Tablet**: Bottom NavigationBar + Drawer for extra items

## Testing
// turbo
```powershell
flutter test
```

## Linting
// turbo
```powershell
flutter analyze
```

## Common Issues

### API Connection Failed
- Ensure backend is running: `cargo run` in project root
- Check firewall/security software
- Verify `baseUrl` in `api_service.dart`

### Build Errors After Model Changes
```powershell
dart run build_runner build --delete-conflicting-outputs
```

### sqflite Issues on Desktop
The app uses `sqflite_common_ffi` for Windows/Linux. Ensure FFI is initialized in `local_storage_service.dart`.
