import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:uuid/uuid.dart';
import 'package:intl/intl.dart';
import '../../providers/customer_provider.dart';
import '../../providers/events_provider.dart';
import '../../api/generated/models/event.dart';
import '../../api/generated/models/event_participant.dart';
import '../../api/generated/models/customer.dart';

/// Event Detail Screen (Refactored to use Providers)
///
/// Now uses CustomerProvider and EventsProvider instead of ApiService.
/// Enables offline customer listing for event registration.

class EventDetailScreen extends StatefulWidget {
  final Event event;

  const EventDetailScreen({super.key, required this.event});

  @override
  State<EventDetailScreen> createState() => _EventDetailScreenState();
}

class _EventDetailScreenState extends State<EventDetailScreen> {
  final List<EventParticipant> _participants = [];
  bool _isRegistering = false;

  @override
  void initState() {
    super.initState();
    // Ensure customers are loaded for registration
    WidgetsBinding.instance.addPostFrameCallback((_) {
      context.read<CustomerProvider>().loadCustomers();
    });
  }

  Future<void> _showRegisterDialog() async {
    // Get customers from provider
    final customerProvider = context.read<CustomerProvider>();
    if (customerProvider.customers.isEmpty) {
      await customerProvider.loadCustomers();
    }

    if (!mounted) return;

    final customers = customerProvider.customers;
    Customer? selectedCustomer;
    bool payWithCredit = false;

    await showDialog(
      context: context,
      builder: (dialogContext) => StatefulBuilder(
        builder: (sbContext, setDialogState) => AlertDialog(
          title: const Text('Register Participant'),
          content: SizedBox(
            width: 400,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                if (customerProvider.isOffline)
                  Container(
                    padding: const EdgeInsets.all(8),
                    margin: const EdgeInsets.only(bottom: 16),
                    decoration: BoxDecoration(
                      color: Colors.orange.shade100,
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: const Row(
                      children: [
                        Icon(Icons.cloud_off, size: 16, color: Colors.orange),
                        SizedBox(width: 8),
                        Text('Offline - using cached customers'),
                      ],
                    ),
                  ),
                DropdownButtonFormField<Customer>(
                  value: selectedCustomer,
                  decoration: const InputDecoration(
                    labelText: 'Select Customer',
                    prefixIcon: Icon(Icons.person),
                  ),
                  items: customers
                      .map((c) => DropdownMenuItem(
                          value: c,
                          child: Text(
                              '${c.name} (Credit: \$${c.storeCredit.toStringAsFixed(2)})')))
                      .toList(),
                  onChanged: (v) => setDialogState(() => selectedCustomer = v),
                ),
                const SizedBox(height: 16),
                if (selectedCustomer != null) ...[
                  Card(
                    color: Theme.of(dialogContext).colorScheme.primaryContainer,
                    child: Padding(
                      padding: const EdgeInsets.all(16),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                              'Entry Fee: \$${widget.event.entryFee.toStringAsFixed(2)}',
                              style: const TextStyle(
                                  fontSize: 18, fontWeight: FontWeight.bold)),
                          const SizedBox(height: 8),
                          Text(
                              'Customer Credit: \$${selectedCustomer!.storeCredit.toStringAsFixed(2)}'),
                          if (selectedCustomer!.storeCredit >=
                              widget.event.entryFee)
                            CheckboxListTile(
                              value: payWithCredit,
                              title: const Text('Pay with Store Credit'),
                              onChanged: (v) =>
                                  setDialogState(() => payWithCredit = v!),
                              contentPadding: EdgeInsets.zero,
                            )
                          else
                            const Text('Insufficient credit',
                                style: TextStyle(color: Colors.orange)),
                        ],
                      ),
                    ),
                  ),
                ],
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(dialogContext),
              child: const Text('Cancel'),
            ),
            FilledButton(
              onPressed: selectedCustomer == null || _isRegistering
                  ? null
                  : () async {
                      setDialogState(() => _isRegistering = true);
                      try {
                        // Create participant matching the generated model
                        final participant = EventParticipant(
                          participantUuid: const Uuid().v4(),
                          eventUuid: widget.event.eventUuid,
                          customerUuid: selectedCustomer!.customerUuid,
                          name: selectedCustomer!.name,
                          paid: payWithCredit,
                          createdAt: DateTime.now(),
                        );

                        // Use EventsProvider for registration
                        await context
                            .read<EventsProvider>()
                            .registerParticipant(
                                widget.event.eventUuid, participant);

                        if (dialogContext.mounted) {
                          Navigator.pop(dialogContext);
                          if (mounted) {
                            ScaffoldMessenger.of(context).showSnackBar(
                              SnackBar(
                                  content: Text(
                                      '${selectedCustomer!.name} registered!'),
                                  backgroundColor: Colors.green),
                            );
                            // Update local state
                            setState(() {
                              _participants.add(participant);
                            });
                          }
                        }
                      } catch (e) {
                        if (mounted) {
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(
                                content: Text('Error: $e'),
                                backgroundColor: Colors.red),
                          );
                        }
                      } finally {
                        if (dialogContext.mounted) {
                          setDialogState(() => _isRegistering = false);
                        }
                      }
                    },
              child: _isRegistering
                  ? const SizedBox(
                      width: 20,
                      height: 20,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Text('Register'),
            ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(widget.event.name),
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Event Info Card
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        CircleAvatar(
                          radius: 30,
                          backgroundColor:
                              Theme.of(context).colorScheme.primaryContainer,
                          child: Icon(Icons.event,
                              size: 30,
                              color: Theme.of(context).colorScheme.primary),
                        ),
                        const SizedBox(width: 16),
                        Expanded(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(widget.event.name,
                                  style: Theme.of(context)
                                      .textTheme
                                      .headlineSmall),
                              Chip(label: Text(widget.event.eventType)),
                            ],
                          ),
                        ),
                      ],
                    ),
                    const Divider(height: 32),
                    _InfoRow(
                        icon: Icons.calendar_today,
                        label: 'Date',
                        value: DateFormat.yMMMMd()
                            .format(widget.event.date.toLocal())),
                    _InfoRow(
                        icon: Icons.access_time,
                        label: 'Time',
                        value: DateFormat.jm()
                            .format(widget.event.date.toLocal())),
                    _InfoRow(
                        icon: Icons.attach_money,
                        label: 'Entry Fee',
                        value: '\$${widget.event.entryFee.toStringAsFixed(2)}'),
                    if (widget.event.maxParticipants != null)
                      _InfoRow(
                          icon: Icons.group,
                          label: 'Max Participants',
                          value: '${widget.event.maxParticipants}'),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 24),

            const SizedBox(height: 24),

            // Tournament Management (Mock)
            if (widget.event.eventType == 'Tournament') ...[
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text('Tournament Actions',
                      style: Theme.of(context).textTheme.titleLarge),
                  Chip(
                      label: const Text('Round 0'),
                      backgroundColor: Colors.amber.shade100),
                ],
              ),
              const SizedBox(height: 16),
              SizedBox(
                width: double.infinity,
                child: Wrap(
                  spacing: 16,
                  runSpacing: 8,
                  alignment: WrapAlignment.start,
                  children: [
                    FilledButton.icon(
                      icon: const Icon(Icons.play_circle_fill),
                      label: const Text('Start Round 1'),
                      style:
                          FilledButton.styleFrom(backgroundColor: Colors.green),
                      onPressed: _participants.length < 2
                          ? null
                          : () {
                              showDialog(
                                  context: context,
                                  builder: (c) => AlertDialog(
                                        title: const Text('Pairings Generated'),
                                        content: const Text(
                                            'Round 1 pairings have been generated based on random seed.'),
                                        actions: [
                                          TextButton(
                                              onPressed: () => Navigator.pop(c),
                                              child: const Text('OK'))
                                        ],
                                      ));
                            },
                    ),
                    OutlinedButton.icon(
                      icon: const Icon(Icons.list_alt),
                      label: const Text('View Standings'),
                      onPressed: () {},
                    ),
                    OutlinedButton.icon(
                      icon: const Icon(Icons.print),
                      label: const Text('Print Slips'),
                      onPressed: () {},
                    ),
                  ],
                ),
              ),
              const Divider(height: 32),
            ],

            // Participants Section
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text('Participants (${_participants.length})',
                    style: Theme.of(context).textTheme.titleLarge),
                FilledButton.icon(
                  onPressed: _showRegisterDialog,
                  icon: const Icon(Icons.person_add),
                  label: const Text('Register'),
                ),
              ],
            ),
            const SizedBox(height: 16),
            if (_participants.isEmpty)
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(32),
                  child: Center(
                    child: Column(
                      children: [
                        Icon(Icons.people_outline,
                            size: 48, color: Colors.grey.shade400),
                        const SizedBox(height: 8),
                        const Text('No participants yet'),
                        const Text('Click "Register" to add players',
                            style: TextStyle(color: Colors.grey)),
                      ],
                    ),
                  ),
                ),
              )
            else
              Card(
                child: ListView.separated(
                  shrinkWrap: true,
                  physics: const NeverScrollableScrollPhysics(),
                  itemCount: _participants.length,
                  separatorBuilder: (_, __) => const Divider(height: 1),
                  itemBuilder: (context, index) {
                    final p = _participants[index];
                    return ListTile(
                      leading: CircleAvatar(child: Text('${index + 1}')),
                      title: Text(p.name),
                      subtitle: Text(
                          'Registered: ${DateFormat.jm().format(p.createdAt.toLocal())}'),
                      trailing: p.paid
                          ? Chip(
                              label: const Text('Paid'),
                              backgroundColor: Colors.green.shade100)
                          : Chip(
                              label: const Text('Unpaid'),
                              backgroundColor: Colors.orange.shade100),
                    );
                  },
                ),
              ),
          ],
        ),
      ),
    );
  }
}

class _InfoRow extends StatelessWidget {
  final IconData icon;
  final String label;
  final String value;

  const _InfoRow(
      {required this.icon, required this.label, required this.value});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        children: [
          Icon(icon, color: Colors.grey, size: 20),
          const SizedBox(width: 12),
          Text('$label:', style: const TextStyle(color: Colors.grey)),
          const SizedBox(width: 8),
          Text(value, style: const TextStyle(fontWeight: FontWeight.w500)),
        ],
      ),
    );
  }
}
