$replacements = @{
    ".product_service"              = ".commerce.product";
    ".inventory_service"            = ".commerce.inventory";
    ".pricing_service"              = ".commerce.pricing";
    ".transaction_service"          = ".commerce.transactions";
    ".buylist_service"              = ".commerce.buylist";
    ".holds_service"                = ".commerce.holds";
    ".payment_service"              = ".commerce.payments";
    ".tax_service"                  = ".commerce.taxes";
    ".returns_service"              = ".commerce.returns";
    ".trade_in_protection_service"  = ".commerce.trade_in";
    
    ".audit_service"                = ".system.audit";
    ".event_service"                = ".system.events";
    ".barcode_service"              = ".system.barcode";
    ".receipt_service"              = ".system.receipts";
    ".invoice_service"              = ".system.invoices";
    ".label_service"                = ".system.labels";
    ".cash_drawer_service"          = ".system.cash_drawer";
    ".printer_service"              = ".system.printers";
    ".catalog_lookup_service"       = ".system.catalog";
    ".serialized_inventory_service" = ".system.serialized";
    ".location_service"             = ".system.locations";
    ".reporting_service"            = ".system.reporting";
    ".email_service"                = ".system.email";
    ".sms_service"                  = ".system.sms";
    ".notification_scheduler"       = ".system.notification_scheduler"
}

$files = @(
    "d:\Projects\VaultSync\src\api\handlers_legacy.rs",
    "d:\Projects\VaultSync\src\api\handlers\customers.rs",
    "d:\Projects\VaultSync\src\api\handlers\health.rs",
    "d:\Projects\VaultSync\src\api\handlers\inventory.rs",
    "d:\Projects\VaultSync\src\api\handlers\pricing.rs",
    "d:\Projects\VaultSync\src\api\handlers\products.rs",
    "d:\Projects\VaultSync\src\api\handlers\reports.rs",
    "d:\Projects\VaultSync\src\api\handlers\sync.rs",
    "d:\Projects\VaultSync\src\api\handlers\transactions.rs"
)

foreach ($file in $files) {
    if (Test-Path $file) {
        Write-Host "Processing $file..."
        $content = Get-Content $file -Raw -Encoding UTF8
        $original = $content
        foreach ($key in $replacements.Keys) {
            $content = $content.Replace($key, $replacements[$key])
        }
        
        if ($content -ne $original) {
            Set-Content -Path $file -Value $content -Encoding UTF8
            Write-Host "Updated $file"
        }
        else {
            Write-Host "No changes in $file"
        }
    }
    else {
        Write-Host "File not found: $file"
    }
}
