//! Product Service - Business logic layer for product operations

use crate::core::Product;
use crate::database::repositories::products::ProductRepository;
use crate::errors::Result;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ProductService {
    repository: Arc<ProductRepository>,
}

impl ProductService {
    pub fn new(repository: ProductRepository) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }

    /// Get all products (limited to 100 by default)
    pub async fn get_all(&self) -> Result<Vec<Product>> {
        self.repository.get_all().await
    }

    /// Get a product by UUID
    pub async fn get_by_id(&self, product_uuid: Uuid) -> Result<Option<Product>> {
        self.repository.get_by_id(product_uuid).await
    }

    /// Search products by name, barcode, or set code
    pub async fn search(&self, query: &str, limit: i32, offset: i32) -> Result<Vec<Product>> {
        self.repository.search(query, limit, offset).await
    }

    /// Insert or update a product
    pub async fn upsert(&self, product: &Product) -> Result<()> {
        self.repository.insert(product).await
    }

    /// Get products by category
    pub async fn get_by_category(&self, category: crate::core::Category) -> Result<Vec<Product>> {
        self.repository.get_by_category(category).await
    }
}
