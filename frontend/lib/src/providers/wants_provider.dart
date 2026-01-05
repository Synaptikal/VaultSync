import 'package:flutter/foundation.dart';
import '../services/api_service.dart';
import '../api/generated/models/wants_list.dart';

class WantsProvider with ChangeNotifier {
  final ApiService _apiService;
  
  WantsProvider(this._apiService);
  
  List<WantsList> _wantsLists = [];
  List<WantsList> get wantsLists => _wantsLists;
  
  String? _selectedCustomerUuid;
  String? get selectedCustomerUuid => _selectedCustomerUuid;
  
  bool _isLoading = false;
  bool get isLoading => _isLoading;

  String? _error;
  String? get error => _error;

  Future<void> loadWantsLists(String customerUuid) async {
    _selectedCustomerUuid = customerUuid;
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      _wantsLists = await _apiService.getWantsLists(customerUuid);
    } catch (e) {
      _error = e.toString();
      if (kDebugMode) print('Failed to load wants lists: $e');
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<void> createWantsList(WantsList wantsList) async {
    _isLoading = true;
    notifyListeners();
    
    try {
      await _apiService.createWantsList(wantsList);
      if (_selectedCustomerUuid != null) {
        await loadWantsLists(_selectedCustomerUuid!);
      }
    } catch (e) {
      _error = e.toString();
      rethrow;
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  void clear() {
    _wantsLists = [];
    _selectedCustomerUuid = null;
    _error = null;
    notifyListeners();
  }
}
