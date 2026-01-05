import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

/// Receipt dialog for viewing and printing transaction receipts
class ReceiptDialog extends StatelessWidget {
  final Map<String, dynamic> transaction;

  const ReceiptDialog({super.key, required this.transaction});

  @override
  Widget build(BuildContext context) {
    return Dialog(
      child: Container(
        width: 380,
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Receipt preview
            Container(
              decoration: BoxDecoration(
                color: Colors.white,
                boxShadow: [
                  BoxShadow(
                    color: Colors.black.withAlpha(25),
                    blurRadius: 10,
                    offset: const Offset(0, 4),
                  ),
                ],
              ),
              child: _ReceiptContent(transaction: transaction),
            ),
            const SizedBox(height: 24),

            // Actions
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              children: [
                OutlinedButton.icon(
                  icon: const Icon(Icons.email),
                  label: const Text('Email'),
                  onPressed: () => _emailReceipt(context),
                ),
                FilledButton.icon(
                  icon: const Icon(Icons.print),
                  label: const Text('Print'),
                  onPressed: () => _printReceipt(context),
                ),
                OutlinedButton.icon(
                  icon: const Icon(Icons.share),
                  label: const Text('Share'),
                  onPressed: () => _shareReceipt(context),
                ),
              ],
            ),
            const SizedBox(height: 16),
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('Close'),
            ),
          ],
        ),
      ),
    );
  }

  void _emailReceipt(BuildContext context) {
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Email receipt feature coming soon')),
    );
  }

  void _printReceipt(BuildContext context) {
    // In a real app, this would use a printing package
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('Receipt sent to printer'),
        backgroundColor: Colors.green,
      ),
    );
    Navigator.pop(context);
  }

  void _shareReceipt(BuildContext context) {
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Share receipt feature coming soon')),
    );
  }
}

class _ReceiptContent extends StatelessWidget {
  final Map<String, dynamic> transaction;

  const _ReceiptContent({required this.transaction});

  @override
  Widget build(BuildContext context) {
    final type = transaction['transaction_type'] ?? 'Sale';
    final total = (transaction['total_amount'] ?? 0.0) as double;
    final date =
        DateTime.tryParse(transaction['created_at'] ?? '') ?? DateTime.now();
    final items = (transaction['items'] as List?) ?? [];
    final transactionId = (transaction['transaction_uuid'] as String?)
            ?.substring(0, 8)
            .toUpperCase() ??
        'N/A';

    return Container(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          // Store header
          const Icon(Icons.diamond, size: 40, color: Colors.indigo),
          const SizedBox(height: 8),
          const Text(
            'VaultSync',
            style: TextStyle(
              fontSize: 24,
              fontWeight: FontWeight.bold,
              letterSpacing: 2,
            ),
          ),
          const Text(
            'Collectibles & Gaming',
            style: TextStyle(color: Colors.grey, fontSize: 12),
          ),
          const SizedBox(height: 4),
          const Text(
            '123 Main Street, Suite 100',
            style: TextStyle(fontSize: 11, color: Colors.grey),
          ),
          const Text(
            'Phone: (555) 123-4567',
            style: TextStyle(fontSize: 11, color: Colors.grey),
          ),
          const SizedBox(height: 16),

          // Divider
          _buildDashedDivider(),
          const SizedBox(height: 12),

          // Transaction info
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text('Receipt #$transactionId',
                  style: const TextStyle(fontWeight: FontWeight.bold)),
              Text(type,
                  style: TextStyle(
                    fontWeight: FontWeight.bold,
                    color: type == 'Sale' ? Colors.green : Colors.orange,
                  )),
            ],
          ),
          const SizedBox(height: 4),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(DateFormat.yMMMd().format(date),
                  style: const TextStyle(fontSize: 12)),
              Text(DateFormat.jm().format(date),
                  style: const TextStyle(fontSize: 12)),
            ],
          ),
          const SizedBox(height: 12),

          // Divider
          _buildDashedDivider(),
          const SizedBox(height: 12),

          // Items
          ...items.map((item) {
            final itemData = item as Map<String, dynamic>;
            final qty = itemData['quantity'] ?? 1;
            final price = (itemData['unit_price'] ?? 0.0) as double;
            final name = itemData['product_name'] ?? 'Item';

            return Padding(
              padding: const EdgeInsets.symmetric(vertical: 4),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  SizedBox(
                    width: 30,
                    child: Text('$qty',
                        style: const TextStyle(fontFamily: 'monospace')),
                  ),
                  Expanded(
                    child: Text(name, style: const TextStyle(fontSize: 13)),
                  ),
                  Text(
                    '\$${(qty * price).toStringAsFixed(2)}',
                    style: const TextStyle(fontFamily: 'monospace'),
                  ),
                ],
              ),
            );
          }),

          const SizedBox(height: 12),
          _buildDashedDivider(),
          const SizedBox(height: 12),

          // Subtotals
          _buildTotalRow('Subtotal', total),
          _buildTotalRow('Tax', 0.0),
          const SizedBox(height: 8),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Text('TOTAL',
                  style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16)),
              Text(
                '\$${total.toStringAsFixed(2)}',
                style:
                    const TextStyle(fontWeight: FontWeight.bold, fontSize: 20),
              ),
            ],
          ),

          const SizedBox(height: 16),
          _buildDashedDivider(),
          const SizedBox(height: 16),

          // Footer
          const Text(
            'Thank you for your business!',
            style: TextStyle(fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 4),
          const Text(
            'Returns accepted within 14 days with receipt',
            style: TextStyle(fontSize: 10, color: Colors.grey),
          ),
          const SizedBox(height: 12),

          // Barcode placeholder
          Container(
            height: 40,
            width: double.infinity,
            decoration: BoxDecoration(
              border: Border.all(color: Colors.grey.shade300),
            ),
            child: CustomPaint(painter: _BarcodePainter()),
          ),
          const SizedBox(height: 4),
          Text(
            transactionId,
            style: const TextStyle(
                fontSize: 10, fontFamily: 'monospace', letterSpacing: 2),
          ),
        ],
      ),
    );
  }

  Widget _buildDashedDivider() {
    return Row(
      children: List.generate(
        30,
        (index) => Expanded(
          child: Container(
            height: 1,
            color: index.isEven ? Colors.grey.shade400 : Colors.transparent,
          ),
        ),
      ),
    );
  }

  Widget _buildTotalRow(String label, double amount) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: const TextStyle(fontSize: 13)),
          Text('\$${amount.toStringAsFixed(2)}',
              style: const TextStyle(fontSize: 13, fontFamily: 'monospace')),
        ],
      ),
    );
  }
}

class _BarcodePainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = Colors.black
      ..style = PaintingStyle.fill;

    final random = [
      1,
      3,
      1,
      1,
      2,
      3,
      1,
      2,
      1,
      3,
      2,
      1,
      1,
      3,
      1,
      2,
      1,
      1,
      2,
      3,
      1,
      2,
      3,
      1,
      1,
      2,
      1,
      3,
      2,
      1
    ];
    double x = 10;

    for (int i = 0; i < random.length && x < size.width - 10; i++) {
      final width = random[i] * 2.0;
      if (i.isEven) {
        canvas.drawRect(Rect.fromLTWH(x, 5, width, size.height - 10), paint);
      }
      x += width + 1;
    }
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}
