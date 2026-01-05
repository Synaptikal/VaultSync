/// Typed API Exceptions (PHASE 1 - Error Handling)
///
/// Provides strongly-typed exceptions for different failure scenarios.
/// This allows the UI to handle errors appropriately instead of showing
/// generic "Something went wrong" messages.
///
/// Usage:
/// ```dart
/// try {
///   await apiClient.post('/api/products', data: product);
/// } on NetworkException catch (e) {
///   showSnackBar('No internet. Saved locally.');
/// } on ValidationException catch (e) {
///   showError(e.message);  // "Invalid barcode format"
/// } on ConflictException catch (e) {
///   navigateToConflictResolution();
/// }
/// ```

sealed class ApiException implements Exception {
  final String message;
  const ApiException(this.message);

  @override
  String toString() => message;
}

/// Network connectivity issues (offline, timeout, etc.)
class NetworkException extends ApiException {
  const NetworkException(super.message);
}

/// Authentication failure (401)
class AuthenticationException extends ApiException {
  const AuthenticationException(super.message);
}

/// Authorization failure (403)
class AuthorizationException extends ApiException {
  const AuthorizationException(super.message);
}

/// Validation error (400)
class ValidationException extends ApiException {
  const ValidationException(super.message);
}

/// Resource not found (404)
class NotFoundException extends ApiException {
  const NotFoundException(super.message);
}

/// Conflict detected (409) - triggers conflict resolution UI
class ConflictException extends ApiException {
  const ConflictException(super.message);
}

/// Server error (500+)
class ServerException extends ApiException {
  const ServerException(super.message);
}

/// Unknown/unexpected error
class UnknownException extends ApiException {
  const UnknownException(super.message);
}
