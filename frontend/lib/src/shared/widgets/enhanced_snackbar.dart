import 'package:flutter/material.dart';

/// Enhanced SnackBar (PHASE 6 - Polish)
///
/// Provides beautiful, consistent snackbars with icons and colors.
///
/// Usage:
/// ```dart
/// EnhancedSnackBar.show(
///   context,
///   message: 'Product saved successfully',
///   type: SnackBarType.success,
/// );
/// ```

class EnhancedSnackBar {
  static void show(
    BuildContext context, {
    required String message,
    required SnackBarType type,
    Duration duration = const Duration(seconds: 4),
    VoidCallback? onAction,
    String? actionLabel,
  }) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Row(
          children: [
            Icon(_iconFor(type), color: Colors.white, size: 20),
            const SizedBox(width: 12),
            Expanded(
              child: Text(
                message,
                style: const TextStyle(fontSize: 14),
              ),
            ),
          ],
        ),
        backgroundColor: _colorFor(type),
        duration: duration,
        action: onAction != null && actionLabel != null
            ? SnackBarAction(
                label: actionLabel,
                textColor: Colors.white,
                onPressed: onAction,
              )
            : null,
        behavior: SnackBarBehavior.floating,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(8),
        ),
        margin: const EdgeInsets.all(16),
      ),
    );
  }

  /// Show success message
  static void success(BuildContext context, String message) {
    show(context, message: message, type: SnackBarType.success);
  }

  /// Show error message
  static void error(BuildContext context, String message,
      {VoidCallback? onRetry}) {
    show(
      context,
      message: message,
      type: SnackBarType.error,
      duration: const Duration(seconds: 6),
      onAction: onRetry,
      actionLabel: onRetry != null ? 'Retry' : null,
    );
  }

  /// Show warning message
  static void warning(BuildContext context, String message) {
    show(
      context,
      message: message,
      type: SnackBarType.warning,
      duration: const Duration(seconds: 5),
    );
  }

  /// Show info message
  static void info(BuildContext context, String message) {
    show(context, message: message, type: SnackBarType.info);
  }

  static IconData _iconFor(SnackBarType type) {
    switch (type) {
      case SnackBarType.success:
        return Icons.check_circle;
      case SnackBarType.error:
        return Icons.error;
      case SnackBarType.warning:
        return Icons.warning_amber;
      case SnackBarType.info:
        return Icons.info;
    }
  }

  static Color _colorFor(SnackBarType type) {
    switch (type) {
      case SnackBarType.success:
        return const Color(0xFF4CAF50); // Green
      case SnackBarType.error:
        return const Color(0xFFF44336); // Red
      case SnackBarType.warning:
        return const Color(0xFFFF9800); // Orange
      case SnackBarType.info:
        return const Color(0xFF2196F3); // Blue
    }
  }
}

/// SnackBar type enum
enum SnackBarType {
  success,
  error,
  warning,
  info,
}

/// Loading Dialog
///
/// Shows a modal loading dialog with message
class LoadingDialog {
  static void show(BuildContext context, {String message = 'Loading...'}) {
    showDialog(
      context: context,
      barrierDismissible: false,
      builder: (context) => PopScope(
        canPop: false,
        child: AlertDialog(
          content: Row(
            children: [
              const CircularProgressIndicator(),
              const SizedBox(width: 24),
              Expanded(child: Text(message)),
            ],
          ),
        ),
      ),
    );
  }

  static void hide(BuildContext context) {
    Navigator.of(context).pop();
  }
}

/// Confirmation Dialog
///
/// Shows a confirmation dialog with customizable actions
class ConfirmationDialog {
  static Future<bool?> show(
    BuildContext context, {
    required String title,
    required String message,
    String confirmLabel = 'Confirm',
    String cancelLabel = 'Cancel',
    bool isDestructive = false,
  }) {
    return showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(title),
        content: Text(message),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: Text(cancelLabel),
          ),
          ElevatedButton(
            onPressed: () => Navigator.of(context).pop(true),
            style: ElevatedButton.styleFrom(
              backgroundColor: isDestructive ? Colors.red : null,
            ),
            child: Text(confirmLabel),
          ),
        ],
      ),
    );
  }
}
