import 'package:flutter/foundation.dart';
import '../services/api_service.dart';
import '../api/generated/models/event.dart';
import '../api/generated/models/event_participant.dart';

/// EventsProvider (Refactored to include registerParticipant)
///
/// Manages event and participant state.
/// Now includes isOffline flag for UI feedback.

class EventsProvider with ChangeNotifier {
  final ApiService _apiService;

  EventsProvider(this._apiService);

  List<Event> _events = [];
  List<Event> get events => _events;

  bool _isLoading = false;
  bool get isLoading => _isLoading;

  bool _isOffline = false;
  bool get isOffline => _isOffline;

  String? _error;
  String? get error => _error;

  Future<void> loadEvents() async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      _events = await _apiService.getEvents();
      // Sort by date desc
      _events.sort((a, b) => b.date.compareTo(a.date));
      _isOffline = false;
    } catch (e) {
      _error = e.toString();
      _isOffline = true;
      if (kDebugMode) print('Failed to load events: $e');
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<void> createEvent(Event event) async {
    _isLoading = true;
    notifyListeners();

    try {
      await _apiService.createEvent(event);
      await loadEvents();
    } catch (e) {
      _error = e.toString();
      // Re-throw so UI can show error dialog/snackbar
      rethrow;
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  /// Register a participant for an event
  Future<void> registerParticipant(
      String eventUuid, EventParticipant participant) async {
    try {
      await _apiService.registerParticipant(eventUuid, participant);
    } catch (e) {
      if (kDebugMode) print('Failed to register participant: $e');
      rethrow;
    }
  }

  /// Get event by UUID
  Event? getById(String eventUuid) {
    return _events.where((e) => e.eventUuid == eventUuid).firstOrNull;
  }
}
