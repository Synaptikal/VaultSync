use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogItem {
    pub title: String,
    pub description: Option<String>,
    pub brand: Option<String>,
    pub category_guess: Option<String>,
    pub image_url: Option<String>,
    pub identifiers: CatalogIdentifiers,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogIdentifiers {
    pub upc: Option<String>,
    pub isbn: Option<String>,
    pub ean: Option<String>,
}

#[async_trait::async_trait]
pub trait CatalogProvider: Send + Sync {
    async fn lookup_upc(&self, upc: &str) -> Result<Option<CatalogItem>>;
    async fn lookup_isbn(&self, isbn: &str) -> Result<Option<CatalogItem>>;
}

pub struct CatalogLookupService {
    providers: Vec<Box<dyn CatalogProvider>>,
}

impl CatalogLookupService {
    pub fn new() -> Self {
        Self {
            providers: vec![
                Box::new(OpenLibraryProvider),
                // Add more providers here (e.g., UPC DB, OpenFoodFacts)
            ],
        }
    }

    pub async fn lookup(&self, barcode: &str) -> Result<Option<CatalogItem>> {
        // Simple heuristic: 10 or 13 digits starting with 978/979 is often ISBN
        // But we can just try all methods or specific ones.

        // Try ISBN first if it looks like one
        if self.looks_like_isbn(barcode) {
            for provider in &self.providers {
                if let Ok(Some(item)) = provider.lookup_isbn(barcode).await {
                    return Ok(Some(item));
                }
            }
        }

        // Try UPC/General
        for provider in &self.providers {
            if let Ok(Some(item)) = provider.lookup_upc(barcode).await {
                return Ok(Some(item));
            }
        }

        Ok(None)
    }

    fn looks_like_isbn(&self, code: &str) -> bool {
        let clean = code.replace("-", "");
        (clean.len() == 10 || clean.len() == 13)
            && (clean.starts_with("978") || clean.starts_with("979") || clean.len() == 10)
    }
}

// --- Providers ---

struct OpenLibraryProvider;

#[async_trait::async_trait]
impl CatalogProvider for OpenLibraryProvider {
    async fn lookup_upc(&self, _upc: &str) -> Result<Option<CatalogItem>> {
        // OpenLibrary is mainly books/ISBN, though sometimes has other IDs.
        Ok(None)
    }

    async fn lookup_isbn(&self, isbn: &str) -> Result<Option<CatalogItem>> {
        let url = format!("https://openlibrary.org/isbn/{}.json", isbn);
        let client = reqwest::Client::new();
        let resp = client.get(&url).send().await?;

        if resp.status().is_success() {
            let json: serde_json::Value = resp.json().await?;
            let title = json["title"]
                .as_str()
                .unwrap_or("Unknown Title")
                .to_string();
            let subtitle = json["subtitle"].as_str();
            let full_title = match subtitle {
                Some(sub) => format!("{}: {}", title, sub),
                None => title,
            };

            // Covers
            let covers = json["covers"].as_array();
            let image_url = if let Some(cv) = covers.and_then(|c| c.first()) {
                Some(format!("https://covers.openlibrary.org/b/id/{}-M.jpg", cv))
            } else {
                None
            };

            // Authors? (We didn't define author field in CatalogItem, add to desc?)

            Ok(Some(CatalogItem {
                title: full_title,
                description: None,
                brand: None,
                category_guess: Some("Book".to_string()),
                image_url,
                identifiers: CatalogIdentifiers {
                    upc: None,
                    isbn: Some(isbn.to_string()),
                    ean: None,
                },
            }))
        } else {
            Ok(None)
        }
    }
}
