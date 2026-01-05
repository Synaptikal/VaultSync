import 'package:flutter/material.dart';
import '../../services/api_exceptions.dart';

/// Error Message Helper (PHASE 6 - Polish)
///
/// Converts exceptions into user-friendly, actionable messages.
/// Provides icons and colors for visual consistency.

class ErrorMessages {
  /// Get user-friendly message for an exception
  static String forException(Exception e) {
    if (e is NetworkException) {
      return 'No internet connection. Your changes are saved locally and will sync automatically when you\'re back online.';
    }

    if (e is AuthenticationException) {
      return 'Your session has expired. Please log in again to continue.';
    }

    if (e is AuthorizationException) {
      return 'You don\'t have permission to perform this action. Please contact your manager.';
    }

    if (e is ValidationException) {
      // Backend already provides user-friendly validation errors
      return e.message;
    }

    if (e is NotFoundException) {
      return 'The requested item was not found. It may have been deleted.';
    }

    if (e is ConflictException) {
      return 'A conflict was detected with changes from another terminal. Please review and resolve.';
    }

    if (e is ServerException) {
      return 'The server is experiencing issues. Please try again in a few moments.';
    }

    if (e is UnknownException) {
      return 'An unexpected error occurred. Please try again or contact support if the problem persists.';
    }

    // Fallback
    return 'An error occurred: ${e.toString()}';
  }

  /// Get appropriate icon for an exception
  static IconData iconForException(Exception e) {
    if (e is NetworkException) return Icons.wifi_off;
    if (e is AuthenticationException) return Icons.lock;
    if (e is AuthorizationException) return Icons.block;
    if (e is ValidationException) return Icons.error_outline;
    if (e is NotFoundException) return Icons.search_off;
    if (e is ConflictException) return Icons.warning_amber;
    if (e is ServerException) return Icons.cloud_off;
    return Icons.error;
  }

  /// Get appropriate color for an exception
  static Color colorForException(Exception e) {
    if (e is NetworkException) return Colors.orange;
    if (e is AuthenticationException) return Colors.red;
    if (e is AuthorizationException) return Colors.red;
    if (e is ValidationException) return Colors.blue;
    if (e is NotFoundException) return Colors.grey;
    if (e is ConflictException) return Colors.orange;
    if (e is ServerException) return Colors.red;
    return Colors.red;
  }

  /// Check if exception requires user action
  static bool requiresAction(Exception e) {
    return e is AuthenticationException ||
        e is ConflictException ||
        e is AuthorizationException;
  }

  /// Get action label for exception
  static String? actionLabelFor(Exception e) {
    if (e is AuthenticationException) return 'Log In';
    if (e is ConflictException) return 'Resolve';
    if (e is AuthorizationException) return 'Contact Manager';
    return null;
  }
}

/// Specific error messages for common scenarios
class CommonErrors {
  static const String noInternet =
      'No internet connection. Changes saved locally.';

  static const String syncFailed =
      'Failed to sync with server. Will retry automatically.';

  static const String invalidBarcode =
      'Invalid barcode format. Please scan again or enter manually.';

  static const String productNotFound =
      'Product not found in database. Please add it first.';

  static const String insufficientQuantity =
      'Not enough quantity available for this transaction.';

  static const String auditInProgress =
      'An audit is already in progress for this location.';

  static const String conflictDetected =
      'Changes conflict with another terminal. Please resolve before continuing.';
}
