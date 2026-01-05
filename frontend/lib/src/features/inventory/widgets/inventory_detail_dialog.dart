import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../../api/generated/models/inventory_item.dart';
import '../../../api/generated/models/product.dart';
import '../../../services/api_service.dart';
import 'add_serialized_item_dialog.dart';
import 'barcode_label_dialog.dart';

class InventoryDetailDialog extends StatefulWidget {
  final InventoryItem item;
  const InventoryDetailDialog({super.key, required this.item});

  @override
  State<InventoryDetailDialog> createState() => _InventoryDetailDialogState();
}

class _InventoryDetailDialogState extends State<InventoryDetailDialog> {
  Product? _product;
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _loadProduct();
  }

  Future<void> _loadProduct() async {
    try {
      final p = await context.read<ApiService>().getProductById(widget.item.productUuid);
      if (mounted) setState(() { _product = p; _loading = false; });
    } catch (e) {
      if (mounted) setState(() => _loading = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final images = widget.item.serializedDetails is Map 
        ? (widget.item.serializedDetails['images'] as List?)
        : null;

    return Dialog(
       backgroundColor: Colors.transparent,
       child: Container(
         width: 800,
         height: 600,
         decoration: BoxDecoration(
           color: Theme.of(context).colorScheme.surface,
           borderRadius: BorderRadius.circular(24),
           boxShadow: const [BoxShadow(blurRadius: 20, color: Colors.black26)],
         ),
         child: Row(
           children: [
             // Left: Image / Visual
             Expanded(
               flex: 4,
               child: Container(
                 decoration: BoxDecoration(
                   color: Colors.black87,
                   borderRadius: const BorderRadius.only(topLeft: Radius.circular(24), bottomLeft: Radius.circular(24)),
                   image: images != null && images.isNotEmpty
                      ? DecorationImage(image: NetworkImage(images[0]), fit: BoxFit.contain)
                      : null,
                 ),
                 child: images == null || images.isEmpty 
                    ? const Center(child: Column(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Icon(Icons.image_not_supported, size: 64, color: Colors.white24),
                          SizedBox(height: 16),
                          Text('No Image', style: TextStyle(color: Colors.white38)),
                        ],
                      ))
                    : null,
               ),
             ),
             // Right: Details
             Expanded(
               flex: 6,
               child: Padding(
                 padding: const EdgeInsets.all(32),
                 child: Column(
                   crossAxisAlignment: CrossAxisAlignment.start,
                   children: [
                     Row(
                       mainAxisAlignment: MainAxisAlignment.spaceBetween,
                       children: [
                         if (_loading) const SizedBox(width: 20, height: 20, child: CircularProgressIndicator())
                         else Expanded(
                           child: Column(
                             crossAxisAlignment: CrossAxisAlignment.start,
                             children: [
                               Text(_product?.name ?? 'Unknown Product', style: Theme.of(context).textTheme.headlineSmall),
                               Text(_product?.setCode ?? '', style: Theme.of(context).textTheme.titleSmall?.copyWith(color: Theme.of(context).colorScheme.primary)),
                             ],
                           ),
                         ),
                         IconButton(icon: const Icon(Icons.close), onPressed: () => Navigator.pop(context)),
                       ],
                     ),
                     const Divider(height: 32),
                     
                     _buildDetailRow('Condition', widget.item.condition.name),
                     _buildDetailRow('Variant', widget.item.variantType?.name ?? 'Normal'),
                     _buildDetailRow('Quantity', widget.item.quantityOnHand.toString()),
                     _buildDetailRow('Location', widget.item.locationTag.isEmpty ? 'Unassigned' : widget.item.locationTag),
                     if (widget.item.specificPrice != null)
                        _buildDetailRow('Price', '\$${widget.item.specificPrice}'),
                     
                     if (widget.item.serializedDetails is Map) ...[
                        const SizedBox(height: 16),
                        const Text('Grading Details', style: TextStyle(fontWeight: FontWeight.bold)),
                        const SizedBox(height: 8),
                        Text((widget.item.serializedDetails as Map).entries
                            .where((e) => e.key != 'images')
                            .map((e) => '${e.key}: ${e.value}').join('\n'),
                            style: TextStyle(color: Colors.grey.shade700)),
                     ],

                     const Spacer(),
                     Row(
                       mainAxisAlignment: MainAxisAlignment.end,
                       children: [
                         TextButton.icon(
                            icon: const Icon(Icons.delete, color: Colors.red), 
                            label: const Text('Dispose', style: TextStyle(color: Colors.red)), 
                            onPressed: (){}
                         ),
                         const SizedBox(width: 8),
                         OutlinedButton.icon(
                           icon: const Icon(Icons.edit), 
                           label: const Text('Edit'), 
                           onPressed: () async {
                             if (_product == null) return;
                             Navigator.pop(context);
                             await showDialog(
                               context: context,
                               builder: (_) => AddSerializedItemDialog(
                                 product: _product!,
                                 existingItem: widget.item,
                               ),
                             );
                           },
                         ),
                         const SizedBox(width: 8),
                         FilledButton.icon(
                           icon: const Icon(Icons.print), 
                           label: const Text('Print Label'), 
                           onPressed: () async {
                             final result = await showDialog(
                               context: context,
                               builder: (_) => BarcodeLabelDialog(
                                 item: widget.item,
                                 productName: _product?.name,
                               ),
                             );
                             if (result != null && context.mounted) {
                               ScaffoldMessenger.of(context).showSnackBar(
                                 SnackBar(
                                   content: Text('Printing ${result['quantity']} label(s)...'),
                                   backgroundColor: Colors.green,
                                 ),
                               );
                             }
                           },
                         ),
                       ],
                     )
                   ],
                 ),
               ),
             ),
           ],
         ),
       ),
    );
  }

  Widget _buildDetailRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        children: [
          SizedBox(width: 100, child: Text(label, style: const TextStyle(fontWeight: FontWeight.bold, color: Colors.grey))),
          Text(value, style: const TextStyle(fontSize: 18)),
        ],
      ),
    );
  }
}
