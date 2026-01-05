import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:intl/intl.dart';
import 'package:uuid/uuid.dart';
import '../../providers/events_provider.dart';
import '../../api/generated/models/event.dart';
import 'event_detail_screen.dart';

class EventsScreen extends StatefulWidget {
  const EventsScreen({super.key});

  @override
  State<EventsScreen> createState() => _EventsScreenState();
}

class _EventsScreenState extends State<EventsScreen> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      context.read<EventsProvider>().loadEvents();
    });
  }

  Future<void> _showAddEventDialog() async {
    final nameController = TextEditingController();
    final feeController = TextEditingController();
    final maxParticipantsController = TextEditingController();
    String eventType = 'Tournament';
    DateTime selectedDate = DateTime.now();
    TimeOfDay selectedTime = TimeOfDay.now();

    await showDialog(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setState) => AlertDialog(
          title: const Text('Create Event'),
          content: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                TextField(
                    controller: nameController,
                    decoration: const InputDecoration(labelText: 'Event Name')),
                const SizedBox(height: 8),
                DropdownButtonFormField<String>(
                  initialValue: eventType,
                  decoration: const InputDecoration(labelText: 'Type'),
                  items: ['Tournament', 'Casual', 'Release', 'Other']
                      .map((t) => DropdownMenuItem(value: t, child: Text(t)))
                      .toList(),
                  onChanged: (v) => setState(() => eventType = v!),
                ),
                const SizedBox(height: 8),
                Row(
                  children: [
                    Expanded(
                      child: TextButton.icon(
                        icon: const Icon(Icons.calendar_today),
                        label: Text(DateFormat.yMMMd().format(selectedDate)),
                        onPressed: () async {
                          final d = await showDatePicker(
                              context: context,
                              firstDate: DateTime.now(),
                              lastDate:
                                  DateTime.now().add(const Duration(days: 365)),
                              initialDate: selectedDate);
                          if (d != null) setState(() => selectedDate = d);
                        },
                      ),
                    ),
                    Expanded(
                      child: TextButton.icon(
                        icon: const Icon(Icons.access_time),
                        label: Text(selectedTime.format(context)),
                        onPressed: () async {
                          final t = await showTimePicker(
                              context: context, initialTime: selectedTime);
                          if (t != null) setState(() => selectedTime = t);
                        },
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 8),
                TextField(
                  controller: feeController,
                  decoration: const InputDecoration(
                      labelText: 'Entry Fee', prefixText: '\$'),
                  keyboardType:
                      const TextInputType.numberWithOptions(decimal: true),
                ),
                const SizedBox(height: 8),
                TextField(
                  controller: maxParticipantsController,
                  decoration:
                      const InputDecoration(labelText: 'Max Participants'),
                  keyboardType: TextInputType.number,
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
                onPressed: () => Navigator.pop(context),
                child: const Text('Cancel')),
            FilledButton(
                onPressed: () async {
                  try {
                    final finalDate = DateTime(
                        selectedDate.year,
                        selectedDate.month,
                        selectedDate.day,
                        selectedTime.hour,
                        selectedTime.minute);

                    final event = Event(
                      eventUuid: const Uuid().v4(),
                      name: nameController.text,
                      eventType: eventType,
                      date: finalDate,
                      entryFee: double.tryParse(feeController.text) ?? 0.0,
                      maxParticipants:
                          int.tryParse(maxParticipantsController.text),
                      createdAt: DateTime.now(),
                    );

                    await context.read<EventsProvider>().createEvent(event);
                    if (mounted) Navigator.pop(context);
                  } catch (e) {
                    ScaffoldMessenger.of(context)
                        .showSnackBar(SnackBar(content: Text('Error: $e')));
                  }
                },
                child: const Text('Create')),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Events')),
      body: Consumer<EventsProvider>(
        builder: (context, provider, child) {
          if (provider.isLoading) {
            return const Center(child: CircularProgressIndicator());
          }
          if (provider.error != null) {
            return Center(child: Text('Error: ${provider.error}'));
          }

          if (provider.events.isEmpty) {
            return const Center(child: Text('No upcoming events.'));
          }

          return ListView.builder(
            padding: const EdgeInsets.all(16),
            itemCount: provider.events.length,
            itemBuilder: (context, index) {
              final event = provider.events[index];
              return Card(
                child: ListTile(
                  leading: CircleAvatar(
                    backgroundColor:
                        Theme.of(context).colorScheme.primaryContainer,
                    child: Text(event.date.day.toString()),
                  ),
                  title: Text(event.name),
                  subtitle: Text(
                      '${event.eventType} • ${DateFormat.jm().format(event.date.toLocal())}\nEntry: \$${event.entryFee.toStringAsFixed(2)}'),
                  isThreeLine: true,
                  trailing: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      if (event.maxParticipants != null)
                        Chip(label: Text('Max: ${event.maxParticipants}')),
                      const SizedBox(width: 8),
                      const Icon(Icons.chevron_right),
                    ],
                  ),
                  onTap: () {
                    Navigator.push(
                      context,
                      MaterialPageRoute(
                        builder: (context) => EventDetailScreen(event: event),
                      ),
                    );
                  },
                ),
              );
            },
          );
        },
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _showAddEventDialog,
        child: const Icon(Icons.add),
      ),
    );
  }
}
