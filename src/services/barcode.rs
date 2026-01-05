use anyhow::Result;
use barcoders::generators::svg::{Color, SVG};
use barcoders::sym::code128::Code128;

use crate::database::Database;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct BarcodeService {
    db: Arc<Database>,
}

impl BarcodeService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn generate_svg(&self, data: &str) -> Result<String> {
        let barcode = Code128::new(format!("{}", data))?;
        let encoded = barcode.encode();

        let svg = SVG::new(50)
            .xdim(2)
            .foreground(Color::black())
            .background(Color::white())
            .generate(&encoded)?;

        Ok(svg)
    }

    pub fn generate_qr_svg(&self, data: &str) -> Result<String> {
        let code = qrcode::QrCode::new(data)?;
        let image = code
            .render::<qrcode::render::svg::Color>()
            .min_dimensions(200, 200)
            .dark_color(qrcode::render::svg::Color("#000000"))
            .light_color(qrcode::render::svg::Color("#ffffff"))
            .build();
        Ok(image)
    }

    pub async fn log_scan(
        &self,
        value: &str,
        scan_type: &str,
        result_type: Option<&str>,
        result_uuid: Option<Uuid>,
        user_uuid: Option<Uuid>,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO Scan_Logs (scan_uuid, scanned_value, scan_type, result_type, result_uuid, user_uuid, scanned_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(Uuid::new_v4().to_string())
        .bind(value)
        .bind(scan_type)
        .bind(result_type)
        .bind(result_uuid.map(|u| u.to_string()))
        .bind(user_uuid.map(|u| u.to_string()))
        .bind(Utc::now().to_rfc3339())
        .execute(&self.db.pool)
        .await?;

        Ok(())
    }
}
