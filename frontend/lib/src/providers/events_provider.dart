import 'package:flutter/foundation.dart';
import '../services/api_service.dart';
import '../api/generated/models/event.dart';

class EventsProvider with ChangeNotifier {
  final ApiService _apiService;
  
  EventsProvider(this._apiService);
  
  List<Event> _events = [];
  List<Event> get events => _events;
  
  bool _isLoading = false;
  bool get isLoading => _isLoading;

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
    } catch (e) {
      _error = e.toString();
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
}
