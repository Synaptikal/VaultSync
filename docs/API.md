# VaultSync API Documentation

Base URL: `http://localhost:3000`

## Authentication

### Register
`POST /auth/register`

Creates a new user account.

**Request Body:**
```json
{
  "username": "admin",
  "password": "securepassword",
  "role": "admin"
}
```

**Response:**
- `201 Created`: User created successfully.
- `409 Conflict`: User already exists.
- `500 Internal Server Error`: Server error.

### Login
`POST /auth/login`

Authenticates a user and returns a JWT token.

**Request Body:**
```json
{
  "username": "admin",
  "password": "securepassword"
}
```

**Response:**
- `200 OK`:
  ```json
  {
    "token": "eyJhbGciOiJIUzI1NiIsInR..."
  }
  ```
- `401 Unauthorized`: Invalid credentials.

## Products

### Get Products
`GET /api/products`

Retrieves all products from the global catalog.

**Headers:**
- `Authorization: Bearer <token>`

**Response:**
- `200 OK`: Array of products.
  ```json
  [
    {
      "product_uuid": "...",
      "name": "Charizard",
      "category": "TCG",
      ...
    }
  ]
  ```

### Create Product
`POST /api/products`

Adds a new product to the global catalog.

**Headers:**
- `Authorization: Bearer <token>`

**Request Body:**
```json
{
  "name": "New Product",
  "category": "TCG",
  "set_code": "Set1",
  "collector_number": "001",
  "metadata": {}
}
```

**Response:**
- `201 Created`: Product created.
- `500 Internal Server Error`: Server error.

## Inventory

### Get Inventory
`GET /api/inventory`

Retrieves all local inventory items.

**Headers:**
- `Authorization: Bearer <token>`

**Response:**
- `200 OK`: Array of inventory items.

### Add Inventory
`POST /api/inventory`

Adds an item to local inventory.

**Headers:**
- `Authorization: Bearer <token>`

**Request Body:**
```json
{
  "product_uuid": "...",
  "variant_type": "Normal",
  "condition": "NM",
  "quantity_on_hand": 10,
  "location_tag": "Box 1"
}
```

**Response:**
- `201 Created`: Item added.
