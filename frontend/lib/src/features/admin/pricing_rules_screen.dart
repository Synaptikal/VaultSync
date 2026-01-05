import 'package:flutter/material.dart';

class PricingRulesScreen extends StatefulWidget {
  const PricingRulesScreen({super.key});

  @override
  State<PricingRulesScreen> createState() => _PricingRulesScreenState();
}

class _PricingRulesScreenState extends State<PricingRulesScreen> {
  // Sample rules - in production, these would come from the backend
  final List<Map<String, dynamic>> _rules = [
    {
      'id': '1',
      'name': 'MTG Standard Buylist',
      'category': 'MTG',
      'condition_multipliers': {
        'NM': 0.65,
        'LP': 0.55,
        'MP': 0.40,
        'HP': 0.25,
        'DMG': 0.10
      },
      'min_value': 0.50,
      'enabled': true,
    },
    {
      'id': '2',
      'name': 'Pokemon Premium',
      'category': 'Pokemon',
      'condition_multipliers': {
        'NM': 0.70,
        'LP': 0.60,
        'MP': 0.45,
        'HP': 0.30,
        'DMG': 0.15
      },
      'min_value': 1.00,
      'enabled': true,
    },
    {
      'id': '3',
      'name': 'Sports Cards Standard',
      'category': 'SportsCards',
      'condition_multipliers': {
        'NM': 0.60,
        'LP': 0.50,
        'MP': 0.35,
        'HP': 0.20,
        'DMG': 0.05
      },
      'min_value': 0.25,
      'enabled': true,
    },
  ];

  void _showAddRuleDialog() {
    final nameController = TextEditingController();
    String selectedCategory = 'MTG';
    final nmController = TextEditingController(text: '0.65');
    final lpController = TextEditingController(text: '0.55');
    final mpController = TextEditingController(text: '0.40');
    final hpController = TextEditingController(text: '0.25');
    final dmgController = TextEditingController(text: '0.10');
    final minValueController = TextEditingController(text: '0.50');

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Add Pricing Rule'),
        content: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              TextField(
                controller: nameController,
                decoration: const InputDecoration(labelText: 'Rule Name'),
              ),
              const SizedBox(height: 16),
              DropdownButtonFormField<String>(
                initialValue: selectedCategory,
                decoration: const InputDecoration(labelText: 'Category'),
                items: ['MTG', 'Pokemon', 'SportsCards', 'Other']
                    .map((c) => DropdownMenuItem(value: c, child: Text(c)))
                    .toList(),
                onChanged: (v) => selectedCategory = v!,
              ),
              const SizedBox(height: 24),
              const Text('Condition Multipliers (% of market value)',
                  style: TextStyle(fontWeight: FontWeight.bold)),
              const SizedBox(height: 8),
              Row(
                children: [
                  Expanded(
                      child: TextField(
                          controller: nmController,
                          decoration: const InputDecoration(labelText: 'NM'),
                          keyboardType: TextInputType.number)),
                  const SizedBox(width: 8),
                  Expanded(
                      child: TextField(
                          controller: lpController,
                          decoration: const InputDecoration(labelText: 'LP'),
                          keyboardType: TextInputType.number)),
                ],
              ),
              Row(
                children: [
                  Expanded(
                      child: TextField(
                          controller: mpController,
                          decoration: const InputDecoration(labelText: 'MP'),
                          keyboardType: TextInputType.number)),
                  const SizedBox(width: 8),
                  Expanded(
                      child: TextField(
                          controller: hpController,
                          decoration: const InputDecoration(labelText: 'HP'),
                          keyboardType: TextInputType.number)),
                ],
              ),
              Row(
                children: [
                  Expanded(
                      child: TextField(
                          controller: dmgController,
                          decoration: const InputDecoration(labelText: 'DMG'),
                          keyboardType: TextInputType.number)),
                  const SizedBox(width: 8),
                  Expanded(
                      child: TextField(
                          controller: minValueController,
                          decoration:
                              const InputDecoration(labelText: 'Min \$'),
                          keyboardType: TextInputType.number)),
                ],
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('Cancel')),
          FilledButton(
            onPressed: () {
              setState(() {
                _rules.add({
                  'id': DateTime.now().millisecondsSinceEpoch.toString(),
                  'name': nameController.text,
                  'category': selectedCategory,
                  'condition_multipliers': {
                    'NM': double.tryParse(nmController.text) ?? 0.65,
                    'LP': double.tryParse(lpController.text) ?? 0.55,
                    'MP': double.tryParse(mpController.text) ?? 0.40,
                    'HP': double.tryParse(hpController.text) ?? 0.25,
                    'DMG': double.tryParse(dmgController.text) ?? 0.10,
                  },
                  'min_value': double.tryParse(minValueController.text) ?? 0.50,
                  'enabled': true,
                });
              });
              Navigator.pop(context);
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(content: Text('Rule "${nameController.text}" added')),
              );
            },
            child: const Text('Add Rule'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Pricing Rules'),
        actions: [
          IconButton(
            icon: const Icon(Icons.help_outline),
            tooltip: 'Help',
            onPressed: () {
              showDialog(
                context: context,
                builder: (context) => AlertDialog(
                  title: const Text('About Pricing Rules'),
                  content: const Text(
                      'Pricing rules determine how much the store offers when buying items from customers.\n\n'
                      'The condition multiplier is applied to the market value. For example, if a card is worth \$100 '
                      'and the NM multiplier is 0.65, the store will offer \$65.\n\n'
                      'Rules are matched by category. More specific rules take priority.'),
                  actions: [
                    TextButton(
                        onPressed: () => Navigator.pop(context),
                        child: const Text('Got it')),
                  ],
                ),
              );
            },
          ),
        ],
      ),
      body: ListView.builder(
        padding: const EdgeInsets.all(16),
        itemCount: _rules.length,
        itemBuilder: (context, index) {
          final rule = _rules[index];
          final multipliers =
              rule['condition_multipliers'] as Map<String, dynamic>;

          return Card(
            margin: const EdgeInsets.only(bottom: 12),
            child: ExpansionTile(
              leading: Switch(
                value: rule['enabled'] as bool,
                onChanged: (v) => setState(() => rule['enabled'] = v),
              ),
              title: Text(rule['name'],
                  style: const TextStyle(fontWeight: FontWeight.bold)),
              subtitle: Text(
                  'Category: ${rule['category']} • Min: \$${rule['min_value']}'),
              children: [
                Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text('Condition Multipliers:',
                          style: TextStyle(fontWeight: FontWeight.w500)),
                      const SizedBox(height: 8),
                      Wrap(
                        spacing: 8,
                        runSpacing: 8,
                        children: multipliers.entries
                            .map((e) => Chip(
                                  label: Text(
                                      '${e.key}: ${(e.value * 100).toStringAsFixed(0)}%'),
                                  backgroundColor: _getConditionColor(e.key),
                                ))
                            .toList(),
                      ),
                      const SizedBox(height: 16),
                      Row(
                        mainAxisAlignment: MainAxisAlignment.end,
                        children: [
                          TextButton.icon(
                            icon: const Icon(Icons.edit),
                            label: const Text('Edit'),
                            onPressed: () {
                              // TODO: Edit dialog
                            },
                          ),
                          const SizedBox(width: 8),
                          TextButton.icon(
                            icon: const Icon(Icons.delete, color: Colors.red),
                            label: const Text('Delete',
                                style: TextStyle(color: Colors.red)),
                            onPressed: () {
                              setState(() => _rules.removeAt(index));
                            },
                          ),
                        ],
                      ),
                    ],
                  ),
                ),
              ],
            ),
          );
        },
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: _showAddRuleDialog,
        icon: const Icon(Icons.add),
        label: const Text('Add Rule'),
      ),
    );
  }

  Color _getConditionColor(String condition) {
    switch (condition) {
      case 'NM':
        return Colors.green.shade100;
      case 'LP':
        return Colors.lightGreen.shade100;
      case 'MP':
        return Colors.yellow.shade100;
      case 'HP':
        return Colors.orange.shade100;
      case 'DMG':
        return Colors.red.shade100;
      default:
        return Colors.grey.shade100;
    }
  }
}
