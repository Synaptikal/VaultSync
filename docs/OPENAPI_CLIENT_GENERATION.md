# OpenAPI Client Generation

This document describes how to generate a type-safe Dart client from the VaultSync API.

## Prerequisites

1. [OpenAPI Generator](https://openapi-generator.tech/) installed:
   ```bash
   npm install @openapitools/openapi-generator-cli -g
   ```

2. VaultSync backend running (for OpenAPI spec export)

## Generating the Client

### Step 1: Export OpenAPI Spec

The VaultSync API exposes its OpenAPI specification at:
```
http://localhost:3000/api-docs/openapi.json
```

Download the spec:
```bash
curl http://localhost:3000/api-docs/openapi.json -o openapi.json
```

### Step 2: Generate Dart Client

```bash
openapi-generator-cli generate \
  -i openapi.json \
  -g dart-dio \
  -o frontend/lib/src/api/generated \
  --additional-properties=pubName=vaultsync_api,pubAuthor=VaultSync
```

### Step 3: Install Dependencies

```bash
cd frontend
flutter pub add dio
flutter pub add built_value
flutter pub add built_collection
flutter pub get
```

## Regenerating After API Changes

Whenever the backend API changes:

1. Restart the backend
2. Re-download the OpenAPI spec
3. Re-run the generator
4. Rebuild the Flutter app

## Deprecation Notice

The manual `api_service.dart` file is deprecated. New features should use 
the generated client. The migration path is:

| Old (Manual)         | New (Generated)                |
|----------------------|--------------------------------|
| `ApiService.getProducts()` | `ProductsApi.getProducts()` |
| `ApiService.createProduct(p)` | `ProductsApi.createProduct(p)` |
| etc.                 | etc.                           |

## Benefits

- **Type Safety**: Compile-time type checking for all API calls
- **Auto-Sync**: Client always matches backend contract
- **Documentation**: Generated code includes JSDoc from backend
- **Consistency**: No manual copy-paste errors
