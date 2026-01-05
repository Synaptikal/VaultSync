import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:flutter/foundation.dart';

abstract class StorageService {
  Future<void> write({required String key, required String value});
  Future<String?> read({required String key});
  Future<void> delete({required String key});
}

class SecureStorageService implements StorageService {
  final _secureStorage = const FlutterSecureStorage();

  @override
  Future<void> write({required String key, required String value}) async {
    try {
      await _secureStorage.write(key: key, value: value);
    } catch (e) {
      if (kDebugMode) {
        print(
            '[SecureStorageService] Write failed: $e. Fallback to SharedPreferences.');
      }
      final prefs = await SharedPreferences.getInstance();
      await prefs.setString(key, value);
    }
  }

  @override
  Future<String?> read({required String key}) async {
    try {
      final value = await _secureStorage.read(key: key);
      // If found in secure storage, return it.
      if (value != null) return value;
    } catch (e) {
      // Ignore read errors from secure storage, check fallback
      if (kDebugMode) {
        print(
            '[SecureStorageService] Read failed: $e. Checking SharedPreferences.');
      }
    }

    // Fallback legacy check (or if write failed previously)
    final prefs = await SharedPreferences.getInstance();
    return prefs.getString(key);
  }

  @override
  Future<void> delete({required String key}) async {
    try {
      await _secureStorage.delete(key: key);
    } catch (e) {
      // ignore
    }
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(key);
  }
}
