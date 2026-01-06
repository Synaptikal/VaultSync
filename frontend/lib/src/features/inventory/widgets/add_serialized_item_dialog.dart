import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:uuid/uuid.dart';
import '../../../api/generated/models/product.dart';
import '../../../api/generated/models/condition.dart';
import '../../../api/generated/models/inventory_item.dart';
import '../../../providers/inventory_provider.dart';
import 'package:file_selector/file_selector.dart';

/// Dialog for adding or editing serialized/graded items with detailed specs
/// (TASK-AUD-001i: Refactored to use InventoryProvider for offline-first)
class AddSerializedItemDialog extends StatefulWidget {
  final Product product;
  final InventoryItem? existingItem; // null = Add mode, non-null = Edit mode

  const AddSerializedItemDialog(
      {super.key, required this.product, this.existingItem});

  bool get isEditMode => existingItem != null;

  @override
  State<AddSerializedItemDialog> createState() =>
      _AddSerializedItemDialogState();
}

class _AddSerializedItemDialogState extends State<AddSerializedItemDialog> {
  final _formKey = GlobalKey<FormState>();

  // Controllers
  final _certNumberController = TextEditingController();
  final _priceController = TextEditingController();
  final _notesController = TextEditingController();
  final _imagesController = TextEditingController();

  final _subCentering = TextEditingController();
  final _subCorners = TextEditingController();
  final _subEdges = TextEditingController();
  final _subSurface = TextEditingController();

  String _selectedGrader = 'PSA';
  String _selectedGrade = '10';
  Condition _selectedCondition = Condition.nm;
  String _locationTag = 'Display Case';

  bool _hasSubgrades = false;
  bool _isSigned = false;
  String _autoGrade = '10';
  bool _isSaving = false;
  String? _error;

  final List<String> _graders = [
    'PSA',
    'BGS',
    'CGC',
    'SGC',
    'TAG',
    'HGA',
    'KSA',
    'AGS',
    'Other',
    'Raw'
  ];
  final List<String> _grades = [
    '10',
    '9.5',
    '9',
    '8.5',
    '8',
    '7.5',
    '7',
    '6.5',
    '6',
    '5.5',
    '5',
    '4.5',
    '4',
    '3.5',
    '3',
    '2.5',
    '2',
    '1.5',
    '1',
    'Authentic',
    'A/Altered'
  ];
  final List<String> _autoGrades = ['10', '9', '8', 'Auto'];
  final List<String> _locationOptions = [
    'Display Case',
    'Wall Limit',
    'Safe',
    'Box A',
    'Box B',
    'Unsorted',
    'Other'
  ];

  @override
  void initState() {
    super.initState();
    _populateFromExisting();
  }

  void _populateFromExisting() {
    final item = widget.existingItem;
    if (item == null) return;

    // Populate condition
    _selectedCondition = item.condition;

    // Validate location tag is in allowed options, default to 'Display Case' if not
    final locations = [
      'Display Case',
      'Wall Limit',
      'Safe',
      'Box A',
      'Box B',
      'Unsorted',
      'Other'
    ];
    if (item.locationTag.isNotEmpty && locations.contains(item.locationTag)) {
      _locationTag = item.locationTag;
    } else {
      _locationTag = 'Display Case';
    }

    if (item.specificPrice != null) {
      _priceController.text = item.specificPrice.toString();
    }

    // Populate from serialized_details
    final details = item.serializedDetails;
    if (details is Map) {
      // Validate grader is in list, default to PSA if not
      final grader = details['grader'];
      _selectedGrader =
          (grader != null && _graders.contains(grader)) ? grader : 'PSA';

      // Validate grade is in list, default to 10 if not
      final grade = details['grade'];
      _selectedGrade =
          (grade != null && _grades.contains(grade)) ? grade : '10';

      _certNumberController.text = details['certification_number'] ?? '';
      _notesController.text = details['notes'] ?? '';

      final images = details['images'];
      if (images is List) {
        _imagesController.text = images.join('\n');
      }

      final subgrades = details['subgrades'];
      if (subgrades is Map) {
        _hasSubgrades = true;
        _subCentering.text = subgrades['centering'] ?? '';
        _subCorners.text = subgrades['corners'] ?? '';
        _subEdges.text = subgrades['edges'] ?? '';
        _subSurface.text = subgrades['surface'] ?? '';
      }

      final autograph = details['autograph'];
      if (autograph is Map) {
        _isSigned = true;
        final autoGradeVal = autograph['grade'];
        _autoGrade =
            (autoGradeVal != null && _autoGrades.contains(autoGradeVal))
                ? autoGradeVal
                : '10';
      }
    }
  }

  @override
  void dispose() {
    _certNumberController.dispose();
    _priceController.dispose();
    _notesController.dispose();
    _imagesController.dispose();
    _subCentering.dispose();
    _subCorners.dispose();
    _subEdges.dispose();
    _subSurface.dispose();
    super.dispose();
  }

  Future<void> _submitItem() async {
    if (!_formKey.currentState!.validate()) return;

    setState(() {
      _isSaving = true;
      _error = null;
    });

    try {
      final serializedDetails = {
        'grader': _selectedGrader,
        'grade': _selectedGrade,
        'certification_number': _certNumberController.text,
        'notes': _notesController.text,
        'images': _imagesController.text
            .split('\n')
            .where((s) => s.isNotEmpty)
            .toList(),
      };

      if (_hasSubgrades) {
        serializedDetails['subgrades'] = {
          'centering': _subCentering.text,
          'corners': _subCorners.text,
          'edges': _subEdges.text,
          'surface': _subSurface.text,
        };
      }

      if (_isSigned) {
        serializedDetails['autograph'] = {
          'grade': _autoGrade,
        };
      }

      final isEdit = widget.isEditMode;
      final inventoryUuid =
          isEdit ? widget.existingItem!.inventoryUuid : const Uuid().v4();

      // Create InventoryItem model
      final item = InventoryItem(
        inventoryUuid: inventoryUuid,
        productUuid: widget.product.productUuid,
        condition: _selectedCondition,
        quantityOnHand: isEdit ? widget.existingItem!.quantityOnHand : 1,
        locationTag: _locationTag,
        specificPrice: double.tryParse(_priceController.text),
        serializedDetails: serializedDetails,
        minStockLevel: isEdit ? widget.existingItem!.minStockLevel : 0,
      );

      // Use InventoryProvider for offline-first operations
      final provider = context.read<InventoryProvider>();
      if (isEdit) {
        await provider.updateItem(item);
      } else {
        await provider.addItem(item);
      }

      if (mounted) {
        Navigator.pop(context, true);
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(isEdit
                ? 'Updated ${widget.product.name}'
                : 'Added graded ${widget.product.name}'),
            backgroundColor: Colors.green,
          ),
        );
      }
    } catch (e) {
      if (mounted) {
        setState(() => _error = e.toString());
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Error: $e'), backgroundColor: Colors.red),
        );
      }
    } finally {
      if (mounted) setState(() => _isSaving = false);
    }
  }

  Future<void> _pickImage() async {
    const XTypeGroup typeGroup = XTypeGroup(
      label: 'images',
      extensions: <String>['jpg', 'jpeg', 'png', 'webp'],
    );
    final XFile? file =
        await openFile(acceptedTypeGroups: <XTypeGroup>[typeGroup]);
    if (file != null) {
      setState(() {
        if (_imagesController.text.isNotEmpty) _imagesController.text += '\n';
        _imagesController.text += file.path;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Dialog(
      child: Container(
        width: 600,
        height: 700, // Taller
        padding: const EdgeInsets.all(24),
        child: Form(
          key: _formKey,
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Header
                Row(
                  children: [
                    Container(
                      padding: const EdgeInsets.all(12),
                      decoration: BoxDecoration(
                          color: Colors.amber.shade100,
                          borderRadius: BorderRadius.circular(12)),
                      child: Icon(Icons.diamond,
                          color: Colors.amber.shade800, size: 32),
                    ),
                    const SizedBox(width: 16),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                              widget.isEditMode
                                  ? 'Edit Graded Item'
                                  : 'Add Graded Item',
                              style: const TextStyle(
                                  fontSize: 18, fontWeight: FontWeight.bold)),
                          Text(widget.product.name,
                              style: const TextStyle(color: Colors.grey)),
                        ],
                      ),
                    ),
                  ],
                ),
                const Divider(height: 32),

                // Error display
                if (_error != null)
                  Container(
                    width: double.infinity,
                    padding: const EdgeInsets.all(8),
                    margin: const EdgeInsets.only(bottom: 16),
                    decoration: BoxDecoration(
                      color: Colors.red.shade100,
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(_error!,
                        style: const TextStyle(color: Colors.red)),
                  ),

                // Grading Info
                const Text('Grading Details',
                    style: TextStyle(fontWeight: FontWeight.bold)),
                const SizedBox(height: 16),
                Row(
                  children: [
                    Expanded(
                      child: DropdownButtonFormField<String>(
                        value: _selectedGrader,
                        decoration: const InputDecoration(
                            labelText: 'Grader', border: OutlineInputBorder()),
                        items: _graders
                            .map((g) =>
                                DropdownMenuItem(value: g, child: Text(g)))
                            .toList(),
                        onChanged: (v) => setState(() => _selectedGrader = v!),
                      ),
                    ),
                    const SizedBox(width: 16),
                    Expanded(
                      child: DropdownButtonFormField<String>(
                        value: _selectedGrade,
                        decoration: const InputDecoration(
                            labelText: 'Grade', border: OutlineInputBorder()),
                        items: _grades
                            .map((g) =>
                                DropdownMenuItem(value: g, child: Text(g)))
                            .toList(),
                        onChanged: (v) => setState(() => _selectedGrade = v!),
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 16),
                TextFormField(
                  controller: _certNumberController,
                  decoration: const InputDecoration(
                      labelText: 'Certification #',
                      border: OutlineInputBorder()),
                  validator: (v) => v == null || v.isEmpty ? 'Required' : null,
                ),

                // Subgrades Toggle
                const SizedBox(height: 16),
                SwitchListTile(
                  title: const Text('Includes Subgrades?'),
                  value: _hasSubgrades,
                  onChanged: (v) => setState(() => _hasSubgrades = v),
                  dense: true,
                  contentPadding: EdgeInsets.zero,
                ),

                if (_hasSubgrades) ...[
                  Row(
                    children: [
                      Expanded(
                          child:
                              _buildSubgradeField('Centering', _subCentering)),
                      const SizedBox(width: 8),
                      Expanded(
                          child: _buildSubgradeField('Corners', _subCorners)),
                      const SizedBox(width: 8),
                      Expanded(child: _buildSubgradeField('Edges', _subEdges)),
                      const SizedBox(width: 8),
                      Expanded(
                          child: _buildSubgradeField('Surface', _subSurface)),
                    ],
                  ),
                  const SizedBox(height: 16),
                ],

                // Autograph Toggle
                SwitchListTile(
                  title: const Text('Autographed / Signed?'),
                  value: _isSigned,
                  onChanged: (v) => setState(() => _isSigned = v),
                  dense: true,
                  contentPadding: EdgeInsets.zero,
                ),
                if (_isSigned) ...[
                  DropdownButtonFormField<String>(
                    value: _autoGrade,
                    decoration: const InputDecoration(
                        labelText: 'Auto Grade', border: OutlineInputBorder()),
                    items: _autoGrades
                        .map((g) => DropdownMenuItem(value: g, child: Text(g)))
                        .toList(),
                    onChanged: (v) => setState(() => _autoGrade = v!),
                  ),
                  const SizedBox(height: 16),
                ],

                const SizedBox(height: 8),
                TextFormField(
                  controller: _imagesController,
                  decoration: InputDecoration(
                    labelText: 'Image Paths/URLs',
                    border: const OutlineInputBorder(),
                    helperText: 'Select file or paste URL',
                    suffixIcon: IconButton(
                        icon: const Icon(Icons.add_photo_alternate),
                        onPressed: _pickImage,
                        tooltip: 'Select Image',
                        color: Colors.blue),
                  ),
                  maxLines: 2,
                ),

                const SizedBox(height: 24),
                // Pricing & Storage
                Row(
                  children: [
                    Expanded(
                      child: TextFormField(
                        controller: _priceController,
                        decoration: const InputDecoration(
                            labelText: 'Price Override',
                            prefixText: '\$ ',
                            border: OutlineInputBorder()),
                        keyboardType: const TextInputType.numberWithOptions(
                            decimal: true),
                      ),
                    ),
                    const SizedBox(width: 16),
                    Expanded(
                      child: DropdownButtonFormField<String>(
                        value: _locationTag,
                        decoration: const InputDecoration(
                            labelText: 'Location',
                            border: OutlineInputBorder()),
                        items: _locationOptions
                            .map((l) =>
                                DropdownMenuItem(value: l, child: Text(l)))
                            .toList(),
                        onChanged: (v) => setState(() => _locationTag = v!),
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 24),

                // Actions
                Row(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    TextButton(
                        onPressed: () => Navigator.pop(context),
                        child: const Text('Cancel')),
                    const SizedBox(width: 16),
                    FilledButton.icon(
                      onPressed: _isSaving ? null : _submitItem,
                      icon: _isSaving
                          ? const SizedBox(
                              width: 16,
                              height: 16,
                              child: CircularProgressIndicator(strokeWidth: 2),
                            )
                          : const Icon(Icons.add),
                      label: Text(widget.isEditMode ? 'Update' : 'Add Item'),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildSubgradeField(String label, TextEditingController controller) {
    return TextFormField(
      controller: controller,
      decoration: InputDecoration(
          labelText: label, border: const OutlineInputBorder(), isDense: true),
      keyboardType: TextInputType.number,
    );
  }
}
