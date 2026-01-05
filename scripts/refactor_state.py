import os

replacements = {
    "state.product_service": "state.commerce.product",
    "state.inventory_service": "state.commerce.inventory",
    "state.pricing_service": "state.commerce.pricing",
    "state.transaction_service": "state.commerce.transactions",
    "state.buylist_service": "state.commerce.buylist",
    "state.holds_service": "state.commerce.holds",
    "state.payment_service": "state.commerce.payments",
    "state.tax_service": "state.commerce.taxes",
    "state.returns_service": "state.commerce.returns",
    "state.trade_in_protection_service": "state.commerce.trade_in",
    
    "state.audit_service": "state.system.audit",
    "state.event_service": "state.system.events",
    "state.barcode_service": "state.system.barcode",
    "state.receipt_service": "state.system.receipts",
    "state.invoice_service": "state.system.invoices",
    "state.label_service": "state.system.labels",
    "state.cash_drawer_service": "state.system.cash_drawer",
    "state.printer_service": "state.system.printers",
    "state.catalog_lookup_service": "state.system.catalog",
    "state.serialized_inventory_service": "state.system.serialized",
    "state.location_service": "state.system.locations",
    "state.reporting_service": "state.system.reporting",
    "state.email_service": "state.system.email",
    "state.sms_service": "state.system.sms",
    "state.notification_scheduler": "state.system.notification_scheduler",
}

def process_file(filepath):
    print(f"Processing {filepath}...")
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        new_content = content
        for old, new in replacements.items():
            new_content = new_content.replace(old, new)
            
        if content != new_content:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(new_content)
            print(f"Updated {filepath}")
        else:
            print(f"No changes in {filepath}")
    except Exception as e:
        print(f"Error processing {filepath}: {e}")

# Target directories/files
targets = [
    r"d:\Projects\VaultSync\src\api\handlers_legacy.rs",
]

handlers_dir = r"d:\Projects\VaultSync\src\api\handlers"
for filename in os.listdir(handlers_dir):
    if filename.endswith(".rs"):
        targets.append(os.path.join(handlers_dir, filename))

for target in targets:
    if os.path.exists(target):
        process_file(target)
    else:
        print(f"File not found: {target}")
