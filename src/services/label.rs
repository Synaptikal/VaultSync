use crate::database::Database;
use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

pub struct LabelService {
    db: Arc<Database>,
    barcode_service: Arc<crate::services::BarcodeService>,
    pricing_service: Arc<crate::pricing::PricingService>,
}

impl LabelService {
    pub fn new(
        db: Arc<Database>,
        barcode_service: Arc<crate::services::BarcodeService>,
        pricing_service: Arc<crate::pricing::PricingService>,
    ) -> Self {
        Self {
            db,
            barcode_service,
            pricing_service,
        }
    }

    pub async fn generate_inventory_label_html(&self, inventory_uuid: Uuid) -> Result<String> {
        let item = self
            .db
            .inventory
            .get_by_id(inventory_uuid)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Inventory item not found"))?;

        let product = self
            .db
            .products
            .get_by_id(item.product_uuid)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Product not found"))?;

        let price = self
            .pricing_service
            .get_cached_price(item.product_uuid)
            .await;

        // Use market mid as default label price if available, logic might vary
        let price_display = if let Some(p) = price {
            format!("${:.2}", p.market_mid)
        } else {
            "$-.--".to_string()
        };

        // Generate barcode for the specific inventory UUID so it can be scanned for exact match
        let barcode_svg = self
            .barcode_service
            .generate_svg(&item.inventory_uuid.to_string())?;

        // Standard 2.25" x 1.25" label usually (300dpi approx 675x375 px)
        // We'll use simple HTML/CSS
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <style>
        @page {{ size: 2.25in 1.25in; margin: 0; }}
        body {{ 
            width: 2.25in; 
            height: 1.25in; 
            margin: 0; 
            padding: 0.1in; 
            box-sizing: border-box; 
            font-family: sans-serif; 
            display: flex; 
            flex-direction: column; 
            justify-content: center; 
            align-items: center; 
            text-align: center;
        }}
        .title {{ font-size: 10pt; font-weight: bold; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 100%; }}
        .meta {{ font-size: 8pt; margin: 2px 0; }}
        .price {{ font-size: 14pt; font-weight: bold; margin: 2px 0; }}
        .barcode svg {{ height: 40px; width: auto; display: block; }}
        .uuid {{ font-size: 6pt; font-family: monospace; }}
    </style>
</head>
<body>
    <div class="title">{}</div>
    <div class="meta">{} | {}</div>
    <div class="price">{}</div>
    <div class="barcode">{}</div>
    <div class="uuid">{}</div>
</body>
</html>"#,
            product.name,
            format!("{:?}", item.condition), // e.g. NearMint
            item.location_tag,
            price_display,
            barcode_svg,
            &item.inventory_uuid.to_string()[..8]
        );

        Ok(html)
    }

    pub async fn generate_product_label_html(&self, product_uuid: Uuid) -> Result<String> {
        let product = self
            .db
            .products
            .get_by_id(product_uuid)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Product not found"))?;

        let price = self.pricing_service.get_cached_price(product_uuid).await;

        let price_display = if let Some(p) = price {
            format!("${:.2}", p.market_mid)
        } else {
            "$-.--".to_string()
        };

        // Prefer existing barcode (UPC), else fallback to product UUID
        let code = product.barcode.unwrap_or(product.product_uuid.to_string());
        let barcode_svg = self.barcode_service.generate_svg(&code)?;

        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <style>
        @page {{ size: 2.25in 1.25in; margin: 0; }}
        body {{ 
            width: 2.25in; 
            height: 1.25in; 
            margin: 0; 
            padding: 0.1in; 
            box-sizing: border-box; 
            font-family: sans-serif; 
            display: flex; 
            flex-direction: column; 
            justify-content: center; 
            align-items: center; 
            text-align: center;
        }}
        .title {{ font-size: 11pt; font-weight: bold; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 100%; }}
        .price {{ font-size: 16pt; font-weight: bold; margin: 5px 0; }}
        .barcode svg {{ height: 40px; width: auto; display: block; }}
        .code {{ font-size: 7pt; font-family: monospace; }}
    </style>
</head>
<body>
    <div class="title">{}</div>
    <div class="price">{}</div>
    <div class="barcode">{}</div>
    <div class="code">{}</div>
</body>
</html>"#,
            product.name, price_display, barcode_svg, code
        );

        Ok(html)
    }
}
