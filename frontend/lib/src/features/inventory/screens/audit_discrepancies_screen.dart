import 'package:flutter/material.dart';
import '../../../models/audit_discrepancy.dart';
import '../../../services/api_client.dart';

/// Audit Discrepancies Screen (PHASE 5 - Inventory Audit)
///
/// Shows the results of a blind count audit.
/// Displays variance between expected and actual quantities.
///
/// Features:
/// - List of all discrepancies
/// - Color-coded by severity
/// - Filter by overage/shortage
/// - Export capability
/// - Drill-down details

class AuditDiscrepanciesScreen extends StatefulWidget {
  final AuditSession session;
  final ApiClient apiClient;

  const AuditDiscrepanciesScreen({
    Key? key,
    required this.session,
    required this.apiClient,
  }) : super(key: key);

  @override
  State<AuditDiscrepanciesScreen> createState() =>
      _AuditDiscrepanciesScreenState();
}

class _AuditDiscrepanciesScreenState extends State<AuditDiscrepanciesScreen> {
  String _filter = 'all'; // 'all', 'overage', 'shortage'

  List<AuditDiscrepancy> get _filteredDiscrepancies {
    final discrepancies = widget.session.discrepancies ?? [];

    switch (_filter) {
      case 'overage':
        return discrepancies.where((d) => d.isOverage).toList();
      case 'shortage':
        return discrepancies.where((d) => d.isShortage).toList();
      default:
        return discrepancies;
    }
  }

  @override
  Widget build(BuildContext context) {
    final discrepancies = widget.session.discrepancies ?? [];
    final hasDiscrepancies = discrepancies.isNotEmpty;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Audit Results'),
        actions: [
          if (hasDiscrepancies)
            IconButton(
              icon: const Icon(Icons.download),
              onPressed: _exportResults,
              tooltip: 'Export',
            ),
        ],
      ),
      body: Column(
        children: [
          _buildSummaryCard(),
          if (hasDiscrepancies) ...[
            _buildFilterChips(),
            Expanded(child: _buildDiscrepanciesList()),
          ] else
            _buildNoDiscrepanciesView(),
        ],
      ),
    );
  }

  Widget _buildSummaryCard() {
    final discrepancies = widget.session.discrepancies ?? [];
    final totalVariance = discrepancies.fold<int>(
      0,
      (sum, d) => sum + d.variance.abs(),
    );
    final overages = discrepancies.where((d) => d.isOverage).length;
    final shortages = discrepancies.where((d) => d.isShortage).length;

    return Card(
      margin: const EdgeInsets.all(16),
      color: discrepancies.isEmpty ? Colors.green[50] : Colors.orange[50],
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            Row(
              children: [
                Icon(
                  discrepancies.isEmpty
                      ? Icons.check_circle
                      : Icons.warning_amber_rounded,
                  size: 48,
                  color: discrepancies.isEmpty ? Colors.green : Colors.orange,
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        discrepancies.isEmpty
                            ? 'Perfect Match!'
                            : '${discrepancies.length} Discrepancy${discrepancies.length > 1 ? 's' : ''} Found',
                        style: const TextStyle(
                          fontSize: 18,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        widget.session.locationTag,
                        style: TextStyle(color: Colors.grey[700]),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        'Completed in ${widget.session.durationText}',
                        style:
                            const TextStyle(fontSize: 12, color: Colors.grey),
                      ),
                    ],
                  ),
                ),
              ],
            ),
            if (discrepancies.isNotEmpty) ...[
              const Divider(height: 24),
              Row(
                children: [
                  Expanded(
                    child: _buildStat('Total Variance', '$totalVariance units'),
                  ),
                  Expanded(
                    child: _buildStat('Overages', '$overages'),
                  ),
                  Expanded(
                    child: _buildStat('Shortages', '$shortages'),
                  ),
                ],
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildStat(String label, String value) {
    return Column(
      children: [
        Text(
          value,
          style: const TextStyle(
            fontSize: 20,
            fontWeight: FontWeight.bold,
          ),
        ),
        const SizedBox(height: 4),
        Text(
          label,
          style: const TextStyle(fontSize: 12, color: Colors.grey),
        ),
      ],
    );
  }

  Widget _buildFilterChips() {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Row(
        children: [
          const Text('Filter: ', style: TextStyle(fontWeight: FontWeight.w500)),
          const SizedBox(width: 8),
          _buildChip('All', 'all'),
          const SizedBox(width: 8),
          _buildChip('Overages', 'overage'),
          const SizedBox(width: 8),
          _buildChip('Shortages', 'shortage'),
        ],
      ),
    );
  }

  Widget _buildChip(String label, String value) {
    final isSelected = _filter == value;

    return FilterChip(
      label: Text(label),
      selected: isSelected,
      onSelected: (selected) {
        setState(() {
          _filter = selected ? value : 'all';
        });
      },
    );
  }

  Widget _buildDiscrepanciesList() {
    final filtered = _filteredDiscrepancies;

    if (filtered.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.filter_list_off, size: 64, color: Colors.grey[300]),
            const SizedBox(height: 16),
            Text(
              'No ${_filter == 'all' ? '' : _filter}s found',
              style: const TextStyle(color: Colors.grey),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      padding: const EdgeInsets.all(16),
      itemCount: filtered.length,
      itemBuilder: (context, index) {
        final discrepancy = filtered[index];
        return _buildDiscrepancyCard(discrepancy);
      },
    );
  }

  Widget _buildDiscrepancyCard(AuditDiscrepancy discrepancy) {
    return Card(
      margin: const EdgeInsets.only(bottom: 12),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                _buildSeverityBadge(discrepancy.severity),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        discrepancy.displayName,
                        style: const TextStyle(
                          fontWeight: FontWeight.bold,
                          fontSize: 16,
                        ),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        discrepancy.condition,
                        style: TextStyle(
                          fontSize: 12,
                          color: Colors.grey[600],
                        ),
                      ),
                    ],
                  ),
                ),
                Text(
                  discrepancy.varianceText,
                  style: TextStyle(
                    fontWeight: FontWeight.bold,
                    fontSize: 16,
                    color: discrepancy.isOverage ? Colors.green : Colors.red,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: _buildQuantityBox(
                    'Expected',
                    discrepancy.expectedQuantity,
                    Colors.blue,
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: _buildQuantityBox(
                    'Counted',
                    discrepancy.actualQuantity,
                    discrepancy.isOverage ? Colors.green : Colors.red,
                  ),
                ),
              ],
            ),
            if (discrepancy.variancePercentage >= 20) ...[
              const SizedBox(height: 12),
              Container(
                padding: const EdgeInsets.all(8),
                decoration: BoxDecoration(
                  color: Colors.orange[50],
                  borderRadius: BorderRadius.circular(4),
                  border: Border.all(color: Colors.orange[200]!),
                ),
                child: Row(
                  children: [
                    const Icon(Icons.info_outline,
                        size: 16, color: Colors.orange),
                    const SizedBox(width: 8),
                    Text(
                      '${discrepancy.variancePercentage.toStringAsFixed(1)}% variance - Investigate',
                      style:
                          const TextStyle(fontSize: 12, color: Colors.orange),
                    ),
                  ],
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildSeverityBadge(DiscrepancySeverity severity) {
    IconData icon;
    Color color;

    switch (severity) {
      case DiscrepancySeverity.high:
        icon = Icons.error;
        color = Colors.red;
        break;
      case DiscrepancySeverity.medium:
        icon = Icons.warning;
        color = Colors.orange;
        break;
      case DiscrepancySeverity.low:
        icon = Icons.info;
        color = Colors.blue;
        break;
    }

    return Container(
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: color.withAlpha(25),
        shape: BoxShape.circle,
      ),
      child: Icon(icon, color: color, size: 24),
    );
  }

  Widget _buildQuantityBox(String label, int quantity, Color color) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: color.withAlpha(25),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withAlpha(77)),
      ),
      child: Column(
        children: [
          Text(
            label,
            style: TextStyle(
              fontSize: 12,
              fontWeight: FontWeight.bold,
              color: color,
            ),
          ),
          const SizedBox(height: 4),
          Text(
            '$quantity',
            style: TextStyle(
              fontSize: 24,
              fontWeight: FontWeight.bold,
              color: color,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildNoDiscrepanciesView() {
    return Expanded(
      child: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.check_circle, size: 80, color: Colors.green[300]),
            const SizedBox(height: 24),
            const Text(
              'All Counts Match!',
              style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 8),
            const Text(
              'Your physical count matches the system perfectly.',
              style: TextStyle(color: Colors.grey),
            ),
            const SizedBox(height: 32),
            ElevatedButton.icon(
              onPressed: () => Navigator.of(context).pop(),
              icon: const Icon(Icons.check),
              label: const Text('Done'),
              style: ElevatedButton.styleFrom(
                backgroundColor: Colors.green,
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

  void _exportResults() {
    // TODO: Implement CSV/PDF export
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Export feature coming soon')),
    );
  }
}
