use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Printer service for managing print operations
pub struct PrinterService {
    printers: Arc<RwLock<Vec<PrinterInfo>>>,
    print_queue: Arc<RwLock<Vec<PrintJob>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterInfo {
    pub printer_id: String,
    pub name: String,
    pub printer_type: PrinterType,
    pub connection: PrinterConnection,
    pub status: PrinterStatus,
    pub paper_width: u32, // in mm
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PrinterType {
    Thermal,
    Label,
    Standard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrinterConnection {
    Usb { vendor_id: u16, product_id: u16 },
    Network { ip: String, port: u16 },
    Serial { port: String, baud_rate: u32 },
    Bluetooth { address: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PrinterStatus {
    Online,
    Offline,
    PaperOut,
    CoverOpen,
    Error,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintJob {
    pub job_uuid: Uuid,
    pub printer_id: String,
    pub job_type: PrintJobType,
    pub content: Vec<u8>,
    pub status: PrintJobStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PrintJobType {
    Receipt,
    Label,
    Invoice,
    Report,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PrintJobStatus {
    Queued,
    Printing,
    Completed,
    Failed,
    Cancelled,
}

/// ESC/POS command builder for thermal printers
pub struct EscPosBuilder {
    buffer: Vec<u8>,
}

impl EscPosBuilder {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Initialize printer
    pub fn init(mut self) -> Self {
        self.buffer.extend_from_slice(&[0x1B, 0x40]); // ESC @
        self
    }

    /// Set text alignment
    pub fn align(mut self, alignment: Alignment) -> Self {
        let code = match alignment {
            Alignment::Left => 0x00,
            Alignment::Center => 0x01,
            Alignment::Right => 0x02,
        };
        self.buffer.extend_from_slice(&[0x1B, 0x61, code]); // ESC a n
        self
    }

    /// Set bold mode
    pub fn bold(mut self, on: bool) -> Self {
        let code = if on { 0x01 } else { 0x00 };
        self.buffer.extend_from_slice(&[0x1B, 0x45, code]); // ESC E n
        self
    }

    /// Set double height/width
    pub fn double_size(mut self, on: bool) -> Self {
        let code = if on { 0x11 } else { 0x00 };
        self.buffer.extend_from_slice(&[0x1D, 0x21, code]); // GS ! n
        self
    }

    /// Set underline
    pub fn underline(mut self, mode: UnderlineMode) -> Self {
        let code = match mode {
            UnderlineMode::Off => 0x00,
            UnderlineMode::Single => 0x01,
            UnderlineMode::Double => 0x02,
        };
        self.buffer.extend_from_slice(&[0x1B, 0x2D, code]); // ESC - n
        self
    }

    /// Add text
    pub fn text(mut self, s: &str) -> Self {
        self.buffer.extend_from_slice(s.as_bytes());
        self
    }

    /// Add newline
    pub fn newline(mut self) -> Self {
        self.buffer.push(0x0A); // LF
        self
    }

    /// Feed paper (n lines)
    pub fn feed(mut self, lines: u8) -> Self {
        self.buffer.extend_from_slice(&[0x1B, 0x64, lines]); // ESC d n
        self
    }

    /// Cut paper (partial cut)
    pub fn cut(mut self) -> Self {
        self.buffer.extend_from_slice(&[0x1D, 0x56, 0x01]); // GS V 1
        self
    }

    /// Full cut
    pub fn full_cut(mut self) -> Self {
        self.buffer.extend_from_slice(&[0x1D, 0x56, 0x00]); // GS V 0
        self
    }

    /// Open cash drawer
    pub fn open_drawer(mut self) -> Self {
        self.buffer
            .extend_from_slice(&[0x1B, 0x70, 0x00, 0x19, 0xFA]); // ESC p 0 25 250
        self
    }

    /// Print horizontal line
    pub fn horizontal_line(mut self, char_width: usize) -> Self {
        self.buffer
            .extend_from_slice("-".repeat(char_width).as_bytes());
        self.buffer.push(0x0A);
        self
    }

    /// Print a receipt line item
    pub fn line_item(mut self, name: &str, price: f64, qty: i32, width: usize) -> Self {
        let qty_str = if qty > 1 {
            format!("{}x", qty)
        } else {
            String::new()
        };
        let price_str = format!("${:.2}", price);
        let name_width = width - price_str.len() - qty_str.len() - 2;
        let name_truncated = if name.len() > name_width {
            &name[..name_width]
        } else {
            name
        };
        let padding = name_width - name_truncated.len();

        let line = format!(
            "{}{} {}{}",
            qty_str,
            name_truncated,
            " ".repeat(padding),
            price_str
        );
        self.buffer.extend_from_slice(line.as_bytes());
        self.buffer.push(0x0A);
        self
    }

    /// Build the final byte buffer
    pub fn build(self) -> Vec<u8> {
        self.buffer
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub enum UnderlineMode {
    Off,
    Single,
    Double,
}

impl PrinterService {
    pub fn new() -> Self {
        Self {
            printers: Arc::new(RwLock::new(Vec::new())),
            print_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// TASK-147: Discover available printers
    pub async fn discover_printers(&self) -> Vec<PrinterInfo> {
        // In a real implementation, this would scan for USB/Network printers
        // For now, return any registered printers
        self.printers.read().await.clone()
    }

    /// Register a printer manually
    pub async fn register_printer(&self, printer: PrinterInfo) -> Result<()> {
        let mut printers = self.printers.write().await;

        // Check if already registered
        if printers.iter().any(|p| p.printer_id == printer.printer_id) {
            tracing::info!(
                "Printer {} already registered, updating",
                printer.printer_id
            );
            printers.retain(|p| p.printer_id != printer.printer_id);
        }

        printers.push(printer.clone());
        tracing::info!(
            "Registered printer: {} ({})",
            printer.name,
            printer.printer_id
        );
        Ok(())
    }

    /// Get printer by ID
    pub async fn get_printer(&self, printer_id: &str) -> Option<PrinterInfo> {
        self.printers
            .read()
            .await
            .iter()
            .find(|p| p.printer_id == printer_id)
            .cloned()
    }

    /// TASK-149: Add print job to queue
    pub async fn queue_print_job(
        &self,
        printer_id: String,
        job_type: PrintJobType,
        content: Vec<u8>,
    ) -> Result<Uuid> {
        let job = PrintJob {
            job_uuid: Uuid::new_v4(),
            printer_id,
            job_type,
            content,
            status: PrintJobStatus::Queued,
            created_at: chrono::Utc::now(),
            completed_at: None,
            error_message: None,
        };

        let job_id = job.job_uuid;
        self.print_queue.write().await.push(job);

        tracing::info!("Print job {} queued", job_id);
        Ok(job_id)
    }

    /// TASK-149: Get pending print jobs
    pub async fn get_pending_jobs(&self) -> Vec<PrintJob> {
        self.print_queue
            .read()
            .await
            .iter()
            .filter(|j| j.status == PrintJobStatus::Queued)
            .cloned()
            .collect()
    }

    /// TASK-149: Process next print job (simulated - returns the content to send)
    pub async fn process_next_job(&self, printer_id: &str) -> Option<PrintJob> {
        let mut queue = self.print_queue.write().await;

        if let Some(pos) = queue
            .iter()
            .position(|j| j.printer_id == printer_id && j.status == PrintJobStatus::Queued)
        {
            queue[pos].status = PrintJobStatus::Printing;
            return Some(queue[pos].clone());
        }
        None
    }

    /// Mark job as completed
    pub async fn complete_job(&self, job_uuid: Uuid) -> Result<()> {
        let mut queue = self.print_queue.write().await;

        if let Some(job) = queue.iter_mut().find(|j| j.job_uuid == job_uuid) {
            job.status = PrintJobStatus::Completed;
            job.completed_at = Some(chrono::Utc::now());
        }
        Ok(())
    }

    /// Mark job as failed
    pub async fn fail_job(&self, job_uuid: Uuid, error: &str) -> Result<()> {
        let mut queue = self.print_queue.write().await;

        if let Some(job) = queue.iter_mut().find(|j| j.job_uuid == job_uuid) {
            job.status = PrintJobStatus::Failed;
            job.error_message = Some(error.to_string());
        }
        Ok(())
    }

    /// TASK-146: Generate receipt using ESC/POS
    pub fn generate_receipt_escpos(
        &self,
        store_name: &str,
        store_address: &str,
        items: &[(String, f64, i32)], // (name, price, qty)
        subtotal: f64,
        tax: f64,
        total: f64,
        payment_info: &str,
    ) -> Vec<u8> {
        const LINE_WIDTH: usize = 42; // Standard 80mm thermal = ~42 chars

        let builder = EscPosBuilder::new()
            .init()
            .align(Alignment::Center)
            .bold(true)
            .double_size(true)
            .text(store_name)
            .newline()
            .double_size(false)
            .bold(false)
            .text(store_address)
            .newline()
            .newline()
            .horizontal_line(LINE_WIDTH)
            .align(Alignment::Left);

        // Add items
        let mut builder = builder;
        for (name, price, qty) in items {
            builder = builder.line_item(name, *price, *qty, LINE_WIDTH);
        }

        builder
            .horizontal_line(LINE_WIDTH)
            .align(Alignment::Right)
            .text(&format!("Subtotal: ${:.2}", subtotal))
            .newline()
            .text(&format!("Tax: ${:.2}", tax))
            .newline()
            .bold(true)
            .text(&format!("TOTAL: ${:.2}", total))
            .newline()
            .bold(false)
            .newline()
            .align(Alignment::Center)
            .text(payment_info)
            .newline()
            .newline()
            .text("Thank you for shopping with us!")
            .newline()
            .feed(3)
            .cut()
            .build()
    }

    /// TASK-148: Generate label using ESC/POS (for label printers)
    pub fn generate_label_escpos(
        &self,
        product_name: &str,
        price: f64,
        barcode_data: &str,
    ) -> Vec<u8> {
        // For label printers (like Zebra), actual implementation would use ZPL
        // This is a simplified ESC/POS version for thermal label printers
        EscPosBuilder::new()
            .init()
            .align(Alignment::Center)
            .bold(true)
            .text(product_name)
            .newline()
            .bold(false)
            .double_size(true)
            .text(&format!("${:.2}", price))
            .double_size(false)
            .newline()
            .text(barcode_data) // Would use GS k for actual barcode
            .newline()
            .feed(1)
            .cut()
            .build()
    }
}

impl Default for EscPosBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PrinterService {
    fn default() -> Self {
        Self::new()
    }
}
