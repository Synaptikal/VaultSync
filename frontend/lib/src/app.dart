import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'features/authentication/login_screen.dart';
import 'features/dashboard/dashboard_screen.dart';
import 'features/inventory/inventory_screen.dart';
import 'features/pos/pos_screen.dart';
import 'features/customers/customers_screen.dart';
import 'features/admin/admin_screen.dart';
import 'features/notifications/notifications_screen.dart';
import 'features/reports/reports_screen.dart';
import 'features/pricing/pricing_screen.dart';
import 'features/admin/sync_config_screen.dart';
import 'features/events/events_screen.dart';
import 'features/wants/wants_screen.dart';
import 'features/transactions/transactions_screen.dart';
import 'features/cash_drawer/cash_drawer_screen.dart';
import 'shared/main_layout.dart';
import 'shared/theme.dart';

class VaultSyncApp extends StatelessWidget {
  const VaultSyncApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp.router(
      title: 'VaultSync',
      theme: AppTheme.lightTheme,
      darkTheme: AppTheme.darkTheme,
      themeMode: ThemeMode.system,
      routerConfig: _router,
      debugShowCheckedModeBanner: false,
    );
  }
}

final _router = GoRouter(
  initialLocation: '/login',
  routes: [
    GoRoute(
      path: '/login',
      builder: (context, state) => const LoginScreen(),
    ),
    ShellRoute(
      builder: (context, state, child) {
        return MainLayout(location: state.uri.toString(), child: child);
      },
      routes: [
        GoRoute(
          path: '/dashboard',
          builder: (context, state) => const DashboardScreen(),
        ),
        GoRoute(
          path: '/inventory',
          builder: (context, state) => const InventoryScreen(),
        ),
        GoRoute(
          path: '/pos',
          builder: (context, state) => const POSScreen(),
        ),
        GoRoute(
          path: '/customers',
          builder: (context, state) => const CustomersScreen(),
        ),
        GoRoute(
          path: '/transactions',
          builder: (context, state) => const TransactionsScreen(),
        ),
        GoRoute(
          path: '/cash-drawer',
          builder: (context, state) => const CashDrawerScreen(),
        ),
        GoRoute(
          path: '/admin',
          builder: (context, state) => const AdminScreen(),
          routes: [
            GoRoute(
              path: 'sync',
              builder: (context, state) => const SyncConfigScreen(),
            ),
          ],
        ),
        GoRoute(
          path: '/notifications',
          builder: (context, state) => const NotificationsScreen(),
        ),
        GoRoute(
          path: '/reports',
          builder: (context, state) => const ReportsScreen(),
        ),
        GoRoute(
          path: '/pricing',
          builder: (context, state) => const PricingScreen(),
        ),
        GoRoute(
          path: '/events',
          builder: (context, state) => const EventsScreen(),
        ),
        GoRoute(
          path: '/wants',
          builder: (context, state) => const WantsScreen(),
        ),
      ],
    ),
  ],
);
