import 'package:flutter/material.dart';
import '../../../api/generated/models/product.dart';

/// Condition grading dialog for trade-in items
/// Provides visual guides to help staff assess card/item condition
class ConditionGradingDialog extends StatefulWidget {
  final Product product;

  const ConditionGradingDialog({
    super.key,
    required this.product,
  });

  @override
  State<ConditionGradingDialog> createState() => _ConditionGradingDialogState();
}

class _ConditionGradingDialogState extends State<ConditionGradingDialog> {
  String _selectedCondition = 'NM';
  double _customPrice = 0.0;
  bool _useCustomPrice = false;
  final _priceController = TextEditingController();

  // Condition multipliers for buy price calculation
  static const Map<String, double> conditionMultipliers = {
    'NM': 0.60,   // Near Mint - 60% of market
    'LP': 0.50,   // Lightly Played - 50%
    'MP': 0.35,   // Moderately Played - 35%
    'HP': 0.20,   // Heavily Played - 20%
    'DMG': 0.10,  // Damaged - 10%
  };

  static const Map<String, ConditionInfo> conditionDetails = {
    'NM': ConditionInfo(
      name: 'Near Mint',
      description: 'Excellent condition with minimal wear',
      criteria: [
        'No visible scratches or scuffs',
        'Corners are sharp and undamaged',
        'No whitening on edges',
        'Surface is clean and unmarked',
      ],
      color: Colors.green,
    ),
    'LP': ConditionInfo(
      name: 'Lightly Played',
      description: 'Minor wear visible upon close inspection',
      criteria: [
        'Light scratches visible at angle',
        'Slight corner wear',
        'Minor edge whitening',
        'May have light surface wear',
      ],
      color: Colors.lightGreen,
    ),
    'MP': ConditionInfo(
      name: 'Moderately Played',
      description: 'Obvious wear visible at arm\'s length',
      criteria: [
        'Scratches visible without close inspection',
        'Noticeable corner wear',
        'Edge whitening clearly visible',
        'Surface wear apparent',
      ],
      color: Colors.orange,
    ),
    'HP': ConditionInfo(
      name: 'Heavily Played',
      description: 'Heavy wear, but card integrity intact',
      criteria: [
        'Heavy scratching throughout',
        'Corners may be soft or damaged',
        'Significant edge wear',
        'May have minor creases or bends',
      ],
      color: Colors.deepOrange,
    ),
    'DMG': ConditionInfo(
      name: 'Damaged',
      description: 'Significant damage affecting card structure',
      criteria: [
        'Major creases, tears, or water damage',
        'Missing pieces or severe corner damage',
        'Writing, stickers, or heavy markings',
        'Card structure compromised',
      ],
      color: Colors.red,
    ),
  };

  double get estimatedBuyPrice {
    // TODO: Get actual market price from pricing service
    const marketPrice = 10.0; // Placeholder
    final multiplier = conditionMultipliers[_selectedCondition] ?? 0.5;
    return marketPrice * multiplier;
  }

  @override
  Widget build(BuildContext context) {
    return Dialog(
      child: Container(
        width: 600,
        constraints: const BoxConstraints(maxHeight: 700),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Header
            Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.primaryContainer,
                borderRadius: const BorderRadius.vertical(top: Radius.circular(12)),
              ),
              child: Row(
                children: [
                  const Icon(Icons.grading),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        const Text(
                          'Condition Grading',
                          style: TextStyle(fontWeight: FontWeight.bold, fontSize: 18),
                        ),
                        Text(
                          widget.product.name,
                          style: Theme.of(context).textTheme.bodySmall,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ],
                    ),
                  ),
                  IconButton(
                    icon: const Icon(Icons.close),
                    onPressed: () => Navigator.pop(context),
                  ),
                ],
              ),
            ),
            
            // Condition selector
            Flexible(
              child: SingleChildScrollView(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text(
                      'Select Condition',
                      style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16),
                    ),
                    const SizedBox(height: 12),
                    
                    // Condition buttons
                    Wrap(
                      spacing: 8,
                      runSpacing: 8,
                      children: conditionDetails.entries.map((entry) {
                        final isSelected = _selectedCondition == entry.key;
                        return ChoiceChip(
                          label: Text(entry.key),
                          selected: isSelected,
                          selectedColor: entry.value.color.withValues(alpha:0.3),
                          onSelected: (selected) {
                            if (selected) {
                              setState(() => _selectedCondition = entry.key);
                            }
                          },
                          avatar: isSelected
                              ? Icon(Icons.check, size: 16, color: entry.value.color)
                              : null,
                        );
                      }).toList(),
                    ),
                    
                    const SizedBox(height: 24),
                    
                    // Condition details card
                    _buildConditionDetailsCard(),
                    
                    const SizedBox(height: 24),
                    
                    // Price section
                    _buildPriceSection(),
                  ],
                ),
              ),
            ),
            
            // Actions
            Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                border: Border(
                  top: BorderSide(color: Colors.grey.shade200),
                ),
              ),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  TextButton(
                    onPressed: () => Navigator.pop(context),
                    child: const Text('Cancel'),
                  ),
                  const SizedBox(width: 16),
                  ElevatedButton.icon(
                    icon: const Icon(Icons.add_shopping_cart),
                    label: const Text('Add to Trade-In'),
                    onPressed: () {
                      Navigator.pop(context, {
                        'condition': _selectedCondition,
                        'price': _useCustomPrice ? _customPrice : estimatedBuyPrice,
                      });
                    },
                    style: ElevatedButton.styleFrom(
                      backgroundColor: Colors.orange,
                      foregroundColor: Colors.white,
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildConditionDetailsCard() {
    final info = conditionDetails[_selectedCondition]!;
    
    return Card(
      color: info.color.withValues(alpha:0.1),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                  decoration: BoxDecoration(
                    color: info.color,
                    borderRadius: BorderRadius.circular(20),
                  ),
                  child: Text(
                    info.name,
                    style: const TextStyle(
                      color: Colors.white,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
                const SizedBox(width: 12),
                Text(
                  '${(conditionMultipliers[_selectedCondition]! * 100).toInt()}% of market',
                  style: TextStyle(
                    color: info.color,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Text(
              info.description,
              style: const TextStyle(fontStyle: FontStyle.italic),
            ),
            const SizedBox(height: 12),
            const Text(
              'Look for:',
              style: TextStyle(fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 4),
            ...info.criteria.map((criterion) => Padding(
              padding: const EdgeInsets.only(left: 8, top: 4),
              child: Row(
                children: [
                  Icon(Icons.check_circle, size: 16, color: info.color),
                  const SizedBox(width: 8),
                  Expanded(child: Text(criterion)),
                ],
              ),
            )),
          ],
        ),
      ),
    );
  }

  Widget _buildPriceSection() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.attach_money),
                const SizedBox(width: 8),
                const Text(
                  'Buy Price',
                  style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16),
                ),
                const Spacer(),
                // Toggle for custom price
                Switch(
                  value: _useCustomPrice,
                  onChanged: (value) => setState(() => _useCustomPrice = value),
                ),
                const Text('Custom'),
              ],
            ),
            const SizedBox(height: 16),
            if (_useCustomPrice)
              TextField(
                controller: _priceController,
                decoration: const InputDecoration(
                  labelText: 'Custom Price',
                  prefixText: '\$ ',
                  border: OutlineInputBorder(),
                ),
                keyboardType: TextInputType.number,
                onChanged: (value) {
                  setState(() {
                    _customPrice = double.tryParse(value) ?? 0.0;
                  });
                },
              )
            else
              Container(
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: Colors.green.shade50,
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(color: Colors.green.shade200),
                ),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    const Text('Suggested: '),
                    Text(
                      '\$${estimatedBuyPrice.toStringAsFixed(2)}',
                      style: TextStyle(
                        fontSize: 24,
                        fontWeight: FontWeight.bold,
                        color: Colors.green.shade700,
                      ),
                    ),
                  ],
                ),
              ),
          ],
        ),
      ),
    );
  }
}

class ConditionInfo {
  final String name;
  final String description;
  final List<String> criteria;
  final Color color;

  const ConditionInfo({
    required this.name,
    required this.description,
    required this.criteria,
    required this.color,
  });
}
