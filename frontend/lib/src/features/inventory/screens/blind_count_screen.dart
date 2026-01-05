import 'package:flutter/material.dart';
import 'package:uuid/uuid.dart';
import '../../../models/audit_discrepancy.dart';
import '../../../services/api_client.dart';
import '../../../services/api_exceptions.dart';
import 'audit_discrepancies_screen.dart';

/// Blind Count Scanner Screen (PHASE 5 - Inventory Audit)
///
/// Allows managers to perform physical inventory counts without
/// seeing the system's expected quantities (blind count).
///
/// Flow:
/// 1. Start audit session for location
/// 2. Scan barcodes (or manual entry)
/// 3. Record quantities WITHOUT showing DB values
/// 4. Submit to backend
/// 5. View discrepancies
///
/// This prevents bias in physical counts.

class BlindCountScreen extends StatefulWidget {
  final ApiClient apiClient;
  final String? initialLocation;

  const BlindCountScreen({
    Key? key,
    required this.apiClient,
    this.initialLocation,
  }) : super(key: key);

  @override
  State<BlindCountScreen> createState() => _BlindCountScreenState();
}

class _BlindCountScreenState extends State<BlindCountScreen> {
  AuditSession? _session;
  final _barcodeController = TextEditingController();
  final _quantityController = TextEditingController(text: '1');
  bool _isSubmitting = false;

  @override
  void initState() {
    super.initState();
    if (widget.initialLocation != null) {
      _startSession(widget.initialLocation!);
    }
  }

  @override
  void dispose() {
    _barcodeController.dispose();
    _quantityController.dispose();
    super.dispose();
  }

  void _startSession(String locationTag) {
    setState(() {
      _session = AuditSession(
        sessionId: const Uuid().v4(),
        locationTag: locationTag,
      );
    });
  }

  void _showLocationPicker() {
    // TODO: Load locations from API
    final locations = ['Front Case', 'Back Room', 'Display Wall', 'Storage'];

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Select Location'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: locations
              .map((loc) => ListTile(
                    title: Text(loc),
                    onTap: () {
                      Navigator.of(context).pop();
                      _startSession(loc);
                    },
                  ))
              .toList(),
        ),
      ),
    );
  }

  void _addScannedItem() {
    final barcode = _barcodeController.text.trim();
    final quantity = int.tryParse(_quantityController.text) ?? 1;

    if (barcode.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Please enter a barcode')),
      );
      return;
    }

    // Create item (in real app, would lookup product from barcode)
    final item = BlindCountItem(
      productUuid: barcode, // TODO: Lookup from API
      productName: 'Product $barcode', // TODO: Get from API
      condition: 'NM', // TODO: Add condition picker
      quantity: quantity,
    );

    setState(() {
      _session!.addItem(item);
      _barcodeController.clear();
      _quantityController.text = '1';
    });

    // Auto-focus barcode field for next scan
    FocusScope.of(context).requestFocus(FocusNode());
  }

  Future<void> _submitAudit() async {
    if (_session == null || _session!.items.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('No items scanned')),
      );
      return;
    }

    setState(() {
      _isSubmitting = true;
    });

    try {
      // Build request payload
      final items = _session!.items.map((item) => item.toJson()).toList();

      // Submit to backend
      final discrepancies = await widget.apiClient.submitBlindCount(items);

      // Parse discrepancies
      final results =
          discrepancies.map((json) => AuditDiscrepancy.fromJson(json)).toList();

      setState(() {
        _session!.discrepancies = results;
        _session!.completedAt = DateTime.now();
        _isSubmitting = false;
      });

      // Navigate to results
      if (mounted) {
        Navigator.of(context).pushReplacement(
          MaterialPageRoute(
            builder: (context) => AuditDiscrepanciesScreen(
              session: _session!,
              apiClient: widget.apiClient,
            ),
          ),
        );
      }
    } on ApiException catch (e) {
      if (mounted) {
        setState(() {
          _isSubmitting = false;
        });

        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Failed to submit audit: ${e.message}'),
            backgroundColor: Colors.red,
          ),
        );
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_session == null) {
      return Scaffold(
        appBar: AppBar(
          title: const Text('Blind Count Audit'),
        ),
        body: Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const Icon(Icons.inventory_2, size: 64, color: Colors.blue),
              const SizedBox(height: 24),
              const Text(
                'Start Inventory Audit',
                style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold),
              ),
              const SizedBox(height: 16),
              const Padding(
                padding: EdgeInsets.symmetric(horizontal: 48),
                child: Text(
                  'Select a location to begin counting physical inventory.',
                  textAlign: TextAlign.center,
                  style: TextStyle(color: Colors.grey),
                ),
              ),
              const SizedBox(height: 32),
              ElevatedButton.icon(
                onPressed: _showLocationPicker,
                icon: const Icon(Icons.location_on),
                label: const Text('Select Location'),
                style: ElevatedButton.styleFrom(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 32,
                    vertical: 16,
                  ),
                ),
              ),
            ],
          ),
        ),
      );
    }

    return Scaffold(
      appBar: AppBar(
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('Blind Count Audit'),
            Text(
              _session!.locationTag,
              style: const TextStyle(fontSize: 14),
            ),
          ],
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.info_outline),
            onPressed: () => _showInstructions(),
            tooltip: 'Instructions',
          ),
        ],
      ),
      body: Column(
        children: [
          _buildScannerSection(),
          _buildItemsList(),
        ],
      ),
      bottomNavigationBar: _buildBottomBar(),
    );
  }

  Widget _buildScannerSection() {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.blue[50],
        border: Border(
          bottom: BorderSide(color: Colors.blue[200]!),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            'Scan or Enter Product',
            style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16),
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                flex: 3,
                child: TextField(
                  controller: _barcodeController,
                  decoration: InputDecoration(
                    hintText: 'Barcode',
                    prefixIcon: const Icon(Icons.qr_code_scanner),
                    filled: true,
                    fillColor: Colors.white,
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(8),
                    ),
                  ),
                  onSubmitted: (_) => _addScannedItem(),
                ),
              ),
              const SizedBox(width: 8),
              SizedBox(
                width: 80,
                child: TextField(
                  controller: _quantityController,
                  decoration: InputDecoration(
                    labelText: 'Qty',
                    filled: true,
                    fillColor: Colors.white,
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(8),
                    ),
                  ),
                  keyboardType: TextInputType.number,
                  textAlign: TextAlign.center,
                ),
              ),
              const SizedBox(width: 8),
              ElevatedButton(
                onPressed: _addScannedItem,
                child: const Icon(Icons.add),
              ),
            ],
          ),
          const SizedBox(height: 8),
          const Text(
            'Note: System quantities are hidden to prevent bias',
            style: TextStyle(fontSize: 12, color: Colors.orange),
          ),
        ],
      ),
    );
  }

  Widget _buildItemsList() {
    if (_session!.items.isEmpty) {
      return Expanded(
        child: Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(Icons.qr_code_scanner, size: 64, color: Colors.grey[300]),
              const SizedBox(height: 16),
              const Text(
                'No items scanned yet',
                style: TextStyle(color: Colors.grey),
              ),
              const SizedBox(height: 8),
              const Text(
                'Scan barcodes to begin counting',
                style: TextStyle(fontSize: 12, color: Colors.grey),
              ),
            ],
          ),
        ),
      );
    }

    return Expanded(
      child: ListView.builder(
        padding: const EdgeInsets.all(16),
        itemCount: _session!.items.length,
        itemBuilder: (context, index) {
          final item = _session!.items[index];
          return Card(
            margin: const EdgeInsets.only(bottom: 8),
            child: ListTile(
              leading: CircleAvatar(
                backgroundColor: Colors.blue,
                child: Text(
                  '${item.quantity}',
                  style: const TextStyle(color: Colors.white),
                ),
              ),
              title: Text(item.productName),
              subtitle: Text('${item.condition} • UUID: ${item.productUuid}'),
              trailing: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  IconButton(
                    icon: const Icon(Icons.remove),
                    onPressed: () {
                      setState(() {
                        if (item.quantity > 1) {
                          item.quantity--;
                        } else {
                          _session!.items.removeAt(index);
                        }
                      });
                    },
                  ),
                  IconButton(
                    icon: const Icon(Icons.add),
                    onPressed: () {
                      setState(() {
                        item.quantity++;
                      });
                    },
                  ),
                ],
              ),
            ),
          );
        },
      ),
    );
  }

  Widget _buildBottomBar() {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.white,
        boxShadow: [
          BoxShadow(
            color: Colors.black.withAlpha(25),
            blurRadius: 4,
            offset: const Offset(0, -2),
          ),
        ],
      ),
      child: Row(
        children: [
          Expanded(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  '${_session!.items.length} unique items',
                  style: const TextStyle(fontWeight: FontWeight.bold),
                ),
                Text(
                  '${_session!.totalItemsScanned} total units',
                  style: const TextStyle(fontSize: 12, color: Colors.grey),
                ),
                Text(
                  'Duration: ${_session!.durationText}',
                  style: const TextStyle(fontSize: 12, color: Colors.grey),
                ),
              ],
            ),
          ),
          ElevatedButton.icon(
            onPressed: _isSubmitting ? null : _submitAudit,
            icon: _isSubmitting
                ? const SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(
                      strokeWidth: 2,
                      valueColor: AlwaysStoppedAnimation(Colors.white),
                    ),
                  )
                : const Icon(Icons.check),
            label: Text(_isSubmitting ? 'Submitting...' : 'Complete Audit'),
            style: ElevatedButton.styleFrom(
              padding: const EdgeInsets.symmetric(
                horizontal: 24,
                vertical: 16,
              ),
            ),
          ),
        ],
      ),
    );
  }

  void _showInstructions() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Blind Count Instructions'),
        content: const SingleChildScrollView(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                '1. Physically count items',
                style: TextStyle(fontWeight: FontWeight.bold),
              ),
              SizedBox(height: 4),
              Text('Do NOT look at the system quantities.'),
              SizedBox(height: 16),
              Text(
                '2. Scan or enter barcodes',
                style: TextStyle(fontWeight: FontWeight.bold),
              ),
              SizedBox(height: 4),
              Text('Record what you actually see on the shelf.'),
              SizedBox(height: 16),
              Text(
                '3. Adjust quantities',
                style: TextStyle(fontWeight: FontWeight.bold),
              ),
              SizedBox(height: 4),
              Text('Use +/- buttons if you counted multiple.'),
              SizedBox(height: 16),
              Text(
                '4. Submit when complete',
                style: TextStyle(fontWeight: FontWeight.bold),
              ),
              SizedBox(height: 4),
              Text('System will compare your counts and show discrepancies.'),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Got it'),
          ),
        ],
      ),
    );
  }
}
