import 'package:flutter/material.dart';

/// Quick actions bar for the POS checkout area
class QuickActionsBar extends StatelessWidget {
  final VoidCallback onAddCustomer;
  final VoidCallback onApplyDiscount;
  final VoidCallback onHoldTransaction;

  const QuickActionsBar({
    super.key,
    required this.onAddCustomer,
    required this.onApplyDiscount,
    required this.onHoldTransaction,
  });

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        _QuickActionButton(
          icon: Icons.person_add,
          label: 'Customer',
          onPressed: onAddCustomer,
        ),
        const SizedBox(width: 8),
        _QuickActionButton(
          icon: Icons.discount,
          label: 'Discount',
          onPressed: onApplyDiscount,
        ),
        const SizedBox(width: 8),
        _QuickActionButton(
          icon: Icons.pause_circle_outline,
          label: 'Hold',
          onPressed: onHoldTransaction,
        ),
      ],
    );
  }
}

class _QuickActionButton extends StatelessWidget {
  final IconData icon;
  final String label;
  final VoidCallback onPressed;

  const _QuickActionButton({
    required this.icon,
    required this.label,
    required this.onPressed,
  });

  @override
  Widget build(BuildContext context) {
    return OutlinedButton.icon(
      onPressed: onPressed,
      icon: Icon(icon, size: 18),
      label: Text(label),
      style: OutlinedButton.styleFrom(
        foregroundColor: Theme.of(context).colorScheme.onPrimaryContainer,
        side: BorderSide(
          color: Theme.of(context).colorScheme.onPrimaryContainer.withValues(alpha: 0.3),
        ),
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      ),
    );
  }
}
