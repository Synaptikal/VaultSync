import 'package:flutter/material.dart';
import '../../../api/generated/models/inventory_item.dart';

/// Dialog for printing barcode labels for inventory items
class BarcodeLabelDialog extends StatefulWidget {
  final InventoryItem item;
  final String? productName;

  const BarcodeLabelDialog({
    super.key,
    required this.item,
    this.productName,
  });

  @override
  State<BarcodeLabelDialog> createState() => _BarcodeLabelDialogState();
}

class _BarcodeLabelDialogState extends State<BarcodeLabelDialog> {
  int _quantity = 1;
  String _labelSize = 'standard';
  bool _includePrice = true;
  bool _includeCondition = true;
  bool _includeQR = false;

  static const _labelSizes = {
    'small': (width: 1.0, height: 0.5, name: 'Small (1" x 0.5")'),
    'standard': (width: 2.0, height: 1.0, name: 'Standard (2" x 1")'),
    'large': (width: 3.0, height: 2.0, name: 'Large (3" x 2")'),
  };

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Row(
        children: [
          Icon(Icons.qr_code, color: Colors.indigo),
          SizedBox(width: 8),
          Text('Print Labels'),
        ],
      ),
      content: SizedBox(
        width: 500,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Item info
            Card(
              child: Padding(
                padding: const EdgeInsets.all(12),
                child: Row(
                  children: [
                    const Icon(Icons.inventory_2, size: 32),
                    const SizedBox(width: 12),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            widget.productName ?? 'Product',
                            style: const TextStyle(fontWeight: FontWeight.bold),
                          ),
                          Text(
                              'SKU: ${widget.item.inventoryUuid.substring(0, 8).toUpperCase()}'),
                          Text('Condition: ${widget.item.condition.name}'),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),

            // Label size
            const Text('Label Size',
                style: TextStyle(fontWeight: FontWeight.bold)),
            const SizedBox(height: 8),
            SegmentedButton<String>(
              segments: _labelSizes.entries
                  .map((e) => ButtonSegment(
                        value: e.key,
                        label: Text(e.value.name),
                      ))
                  .toList(),
              selected: {_labelSize},
              onSelectionChanged: (selection) {
                setState(() => _labelSize = selection.first);
              },
            ),
            const SizedBox(height: 16),

            // Quantity
            Row(
              children: [
                const Text('Quantity:',
                    style: TextStyle(fontWeight: FontWeight.bold)),
                const SizedBox(width: 16),
                IconButton(
                  icon: const Icon(Icons.remove_circle_outline),
                  onPressed:
                      _quantity > 1 ? () => setState(() => _quantity--) : null,
                ),
                Container(
                  padding:
                      const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                  decoration: BoxDecoration(
                    border: Border.all(color: Colors.grey),
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Text('$_quantity',
                      style: const TextStyle(
                          fontSize: 18, fontWeight: FontWeight.bold)),
                ),
                IconButton(
                  icon: const Icon(Icons.add_circle_outline),
                  onPressed: () => setState(() => _quantity++),
                ),
                const Spacer(),
                // Quick quantity buttons
                Wrap(
                  spacing: 8,
                  children: [1, 5, 10, 25]
                      .map((q) => ActionChip(
                            label: Text('$q'),
                            onPressed: () => setState(() => _quantity = q),
                            backgroundColor: _quantity == q
                                ? Theme.of(context).colorScheme.primaryContainer
                                : null,
                          ))
                      .toList(),
                ),
              ],
            ),
            const SizedBox(height: 16),

            // Options
            const Text('Include on Label',
                style: TextStyle(fontWeight: FontWeight.bold)),
            CheckboxListTile(
              title: const Text('Price'),
              subtitle: Text(
                  '\$${widget.item.specificPrice?.toStringAsFixed(2) ?? 'N/A'}'),
              value: _includePrice,
              onChanged: (v) => setState(() => _includePrice = v!),
              contentPadding: EdgeInsets.zero,
            ),
            CheckboxListTile(
              title: const Text('Condition'),
              subtitle: Text(widget.item.condition.name),
              value: _includeCondition,
              onChanged: (v) => setState(() => _includeCondition = v!),
              contentPadding: EdgeInsets.zero,
            ),
            CheckboxListTile(
              title: const Text('QR Code'),
              subtitle: const Text('For mobile scanning'),
              value: _includeQR,
              onChanged: (v) => setState(() => _includeQR = v!),
              contentPadding: EdgeInsets.zero,
            ),
            const SizedBox(height: 16),

            // Preview
            const Text('Preview',
                style: TextStyle(fontWeight: FontWeight.bold)),
            const SizedBox(height: 8),
            Center(
              child: Container(
                padding: const EdgeInsets.all(8),
                decoration: BoxDecoration(
                  border: Border.all(color: Colors.grey.shade300),
                  color: Colors.white,
                ),
                child: _LabelPreview(
                  productName: widget.productName ?? 'Product',
                  sku: widget.item.inventoryUuid.substring(0, 8).toUpperCase(),
                  price: _includePrice ? widget.item.specificPrice : null,
                  condition:
                      _includeCondition ? widget.item.condition.name : null,
                  showQR: _includeQR,
                  size: _labelSize,
                ),
              ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Cancel'),
        ),
        FilledButton.icon(
          icon: const Icon(Icons.print),
          label: Text('Print $_quantity Label${_quantity > 1 ? 's' : ''}'),
          onPressed: () {
            Navigator.pop(context, {
              'quantity': _quantity,
              'size': _labelSize,
              'includePrice': _includePrice,
              'includeCondition': _includeCondition,
              'includeQR': _includeQR,
            });
          },
        ),
      ],
    );
  }
}

class _LabelPreview extends StatelessWidget {
  final String productName;
  final String sku;
  final double? price;
  final String? condition;
  final bool showQR;
  final String size;

  const _LabelPreview({
    required this.productName,
    required this.sku,
    this.price,
    this.condition,
    required this.showQR,
    required this.size,
  });

  @override
  Widget build(BuildContext context) {
    final isSmall = size == 'small';
    final isLarge = size == 'large';

    return Container(
      width: isSmall ? 100 : (isLarge ? 200 : 150),
      height: isSmall ? 50 : (isLarge ? 120 : 80),
      padding: const EdgeInsets.all(4),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Product name
          Text(
            productName,
            maxLines: isSmall ? 1 : 2,
            overflow: TextOverflow.ellipsis,
            style: TextStyle(
              fontWeight: FontWeight.bold,
              fontSize: isSmall ? 8 : (isLarge ? 12 : 10),
            ),
          ),
          if (!isSmall && condition != null) ...[
            Text(
              condition!,
              style: TextStyle(fontSize: isLarge ? 10 : 8, color: Colors.grey),
            ),
          ],
          const Spacer(),
          Row(
            children: [
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    if (price != null)
                      Text(
                        '\$${price!.toStringAsFixed(2)}',
                        style: TextStyle(
                          fontWeight: FontWeight.bold,
                          fontSize: isSmall ? 10 : (isLarge ? 16 : 12),
                        ),
                      ),
                    // Barcode representation
                    SizedBox(
                      height: isSmall ? 12 : (isLarge ? 24 : 16),
                      child: CustomPaint(
                        painter: _MiniBarcodePainter(),
                        size: Size(isSmall ? 50 : (isLarge ? 100 : 70),
                            isSmall ? 12 : (isLarge ? 24 : 16)),
                      ),
                    ),
                    Text(
                      sku,
                      style: TextStyle(
                          fontSize: isSmall ? 6 : 8, fontFamily: 'monospace'),
                    ),
                  ],
                ),
              ),
              if (showQR && !isSmall) ...[
                const SizedBox(width: 4),
                Container(
                  width: isLarge ? 40 : 24,
                  height: isLarge ? 40 : 24,
                  decoration: BoxDecoration(
                    border: Border.all(color: Colors.black, width: 0.5),
                  ),
                  child: const Center(
                    child: Icon(Icons.qr_code_2, size: 16),
                  ),
                ),
              ],
            ],
          ),
        ],
      ),
    );
  }
}

class _MiniBarcodePainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = Colors.black
      ..style = PaintingStyle.fill;

    final pattern = [
      1,
      2,
      1,
      1,
      2,
      1,
      2,
      1,
      1,
      2,
      1,
      2,
      1,
      1,
      2,
      1,
      2,
      1,
      1,
      2
    ];
    double x = 0;

    for (int i = 0; i < pattern.length && x < size.width; i++) {
      final width = pattern[i] * 1.5;
      if (i.isEven) {
        canvas.drawRect(Rect.fromLTWH(x, 0, width, size.height), paint);
      }
      x += width + 0.5;
    }
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}
