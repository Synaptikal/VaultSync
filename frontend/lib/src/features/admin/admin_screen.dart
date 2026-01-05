import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'pricing_rules_screen.dart';

class AdminScreen extends StatelessWidget {
  const AdminScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Administration')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          _buildAdminCard(
            context,
            title: 'User Management',
            icon: Icons.manage_accounts,
            description: 'Add, remove, or modify user permissions.',
            onTap: () {},
          ),
          const SizedBox(height: 16),
          _buildAdminCard(
            context,
            title: 'Database Settings',
            icon: Icons.storage,
            description: 'Backup, restore, or optimize the local database.',
            onTap: () {},
          ),
          const SizedBox(height: 16),
          _buildAdminCard(
            context,
            title: 'Sync Configuration',
            icon: Icons.sync_lock,
            description: 'Manage sync nodes and conflict resolution policies.',
            onTap: () => context.go('/admin/sync'),
          ),
          const SizedBox(height: 16),
          _buildAdminCard(
            context,
            title: 'System Logs',
            icon: Icons.terminal,
            description: 'View application logs and error reports.',
            onTap: () {},
          ),
          const SizedBox(height: 16),
          _buildAdminCard(
            context,
            title: 'Pricing Rules',
            icon: Icons.rule,
            description: 'Configure buylist pricing multipliers and rules.',
            onTap: () => Navigator.push(
              context,
              MaterialPageRoute(builder: (context) => const PricingRulesScreen()),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildAdminCard(
    BuildContext context, {
    required String title,
    required IconData icon,
    required String description,
    required VoidCallback onTap,
  }) {
    return Card(
      child: ListTile(
        leading: CircleAvatar(
          backgroundColor: Theme.of(context).colorScheme.primaryContainer,
          child: Icon(icon, color: Theme.of(context).colorScheme.primary),
        ),
        title: Text(title, style: const TextStyle(fontWeight: FontWeight.bold)),
        subtitle: Text(description),
        trailing: const Icon(Icons.arrow_forward_ios, size: 16),
        contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        onTap: onTap,
      ),
    );
  }
}
