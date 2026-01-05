import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

class MainLayout extends StatelessWidget {
  final Widget child;
  final String location;

  const MainLayout({
    super.key,
    required this.child,
    required this.location,
  });

  @override
  Widget build(BuildContext context) {
    final isDesktop = MediaQuery.of(context).size.width > 800;

    if (isDesktop) {
      return Scaffold(
        body: Row(
          children: [
            NavigationRail(
              selectedIndex: _getSelectedIndex(location),
              onDestinationSelected: (index) => _onItemTapped(index, context),
              labelType: NavigationRailLabelType.all,
              destinations: const [
                NavigationRailDestination(
                  icon: Icon(Icons.dashboard_outlined),
                  selectedIcon: Icon(Icons.dashboard),
                  label: Text('Dashboard'),
                ),
                NavigationRailDestination(
                  icon: Icon(Icons.point_of_sale_outlined),
                  selectedIcon: Icon(Icons.point_of_sale),
                  label: Text('POS'),
                ),
                NavigationRailDestination(
                  icon: Icon(Icons.inventory_2_outlined),
                  selectedIcon: Icon(Icons.inventory_2),
                  label: Text('Inventory'),
                ),
                NavigationRailDestination(
                  icon: Icon(Icons.price_change_outlined),
                  selectedIcon: Icon(Icons.price_change),
                  label: Text('Pricing'),
                ),
                NavigationRailDestination(
                  icon: Icon(Icons.people_outline),
                  selectedIcon: Icon(Icons.people),
                  label: Text('Customers'),
                ),
                NavigationRailDestination(
                  icon: Icon(Icons.event_outlined),
                  selectedIcon: Icon(Icons.event),
                  label: Text('Events'),
                ),
                NavigationRailDestination(
                  icon: Icon(Icons.admin_panel_settings_outlined),
                  selectedIcon: Icon(Icons.admin_panel_settings),
                  label: Text('Admin'),
                ),
              ],
              leading: const Padding(
                padding: EdgeInsets.symmetric(vertical: 20),
                child: Column(
                  children: [
                    Icon(Icons.diamond, size: 32, color: Color(0xFFFFD700)),
                    SizedBox(height: 8),
                    Text('VAULT', style: TextStyle(fontWeight: FontWeight.bold, fontSize: 10)),
                  ],
                ),
              ),
              trailing: Expanded(
                child: Align(
                  alignment: Alignment.bottomCenter,
                  child: Padding(
                    padding: const EdgeInsets.only(bottom: 20),
                    child: IconButton(
                      icon: const Icon(Icons.notifications_outlined),
                      onPressed: () => context.go('/notifications'),
                    ),
                  ),
                ),
              ),
            ),
            const VerticalDivider(thickness: 1, width: 1),
            Expanded(child: child),
          ],
        ),
      );
    } else {
      // Mobile/Tablet Layout
      return Scaffold(
        appBar: AppBar(
          title: const Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.diamond, color: Color(0xFFFFD700)),
              SizedBox(width: 8),
              Text('VaultSync'),
            ],
          ),
          actions: [
            IconButton(
              icon: const Icon(Icons.notifications_outlined),
              onPressed: () => context.go('/notifications'),
            ),
          ],
        ),
        body: child,
        bottomNavigationBar: NavigationBar(
          selectedIndex: _getMobileSelectedIndex(location),
          onDestinationSelected: (index) => _onItemTapped(index, context),
          destinations: const [
            NavigationDestination(
              icon: Icon(Icons.dashboard_outlined),
              selectedIcon: Icon(Icons.dashboard),
              label: 'Home',
            ),
            NavigationDestination(
              icon: Icon(Icons.point_of_sale_outlined),
              selectedIcon: Icon(Icons.point_of_sale),
              label: 'POS',
            ),
            NavigationDestination(
              icon: Icon(Icons.inventory_2_outlined),
              selectedIcon: Icon(Icons.inventory_2),
              label: 'Inventory',
            ),
            NavigationDestination(
              icon: Icon(Icons.people_outline),
              selectedIcon: Icon(Icons.people),
              label: 'Customers',
            ),
            NavigationDestination(
              icon: Icon(Icons.more_horiz),
              selectedIcon: Icon(Icons.more_horiz),
              label: 'More',
            ),
          ],
        ),
        drawer: Drawer(
          child: ListView(
            padding: EdgeInsets.zero,
            children: [
              const DrawerHeader(
                decoration: BoxDecoration(
                  gradient: LinearGradient(
                    colors: [Color(0xFF1A1A2E), Color(0xFF16213E)],
                    begin: Alignment.topLeft,
                    end: Alignment.bottomRight,
                  ),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    Icon(Icons.diamond, size: 48, color: Color(0xFFFFD700)),
                    SizedBox(height: 8),
                    Text('VaultSync', style: TextStyle(color: Colors.white, fontSize: 24, fontWeight: FontWeight.bold)),
                    Text('Collectibles POS', style: TextStyle(color: Colors.white70)),
                  ],
                ),
              ),
              ListTile(
                leading: const Icon(Icons.price_change),
                title: const Text('Pricing'),
                onTap: () {
                  Navigator.pop(context);
                  context.go('/pricing');
                },
              ),
              ListTile(
                leading: const Icon(Icons.receipt_long),
                title: const Text('Transactions'),
                onTap: () {
                  Navigator.pop(context);
                  context.go('/transactions');
                },
              ),
              ListTile(
                leading: const Icon(Icons.point_of_sale),
                title: const Text('Cash Drawer'),
                onTap: () {
                  Navigator.pop(context);
                  context.go('/cash-drawer');
                },
              ),
              ListTile(
                leading: const Icon(Icons.event),
                title: const Text('Events'),
                onTap: () {
                  Navigator.pop(context);
                  context.go('/events');
                },
              ),
              ListTile(
                leading: const Icon(Icons.checklist),
                title: const Text('Wants Lists'),
                onTap: () {
                  Navigator.pop(context);
                  context.go('/wants');
                },
              ),
              ListTile(
                leading: const Icon(Icons.bar_chart),
                title: const Text('Reports'),
                onTap: () {
                  Navigator.pop(context);
                  context.go('/reports');
                },
              ),
              const Divider(),
              ListTile(
                leading: const Icon(Icons.admin_panel_settings),
                title: const Text('Admin'),
                onTap: () {
                  Navigator.pop(context);
                  context.go('/admin');
                },
              ),
            ],
          ),
        ),
      );
    }
  }

  int _getSelectedIndex(String location) {
    if (location.startsWith('/dashboard')) return 0;
    if (location.startsWith('/pos')) return 1;
    if (location.startsWith('/inventory')) return 2;
    if (location.startsWith('/pricing')) return 3;
    if (location.startsWith('/customers')) return 4;
    if (location.startsWith('/events')) return 5;
    if (location.startsWith('/admin')) return 6;
    return 0;
  }

  int _getMobileSelectedIndex(String location) {
    if (location.startsWith('/dashboard')) return 0;
    if (location.startsWith('/pos')) return 1;
    if (location.startsWith('/inventory')) return 2;
    if (location.startsWith('/customers')) return 3;
    return 4; // More
  }

  void _onItemTapped(int index, BuildContext context) {
    final isDesktop = MediaQuery.of(context).size.width > 800;
    
    if (isDesktop) {
      switch (index) {
        case 0: context.go('/dashboard'); break;
        case 1: context.go('/pos'); break;
        case 2: context.go('/inventory'); break;
        case 3: context.go('/pricing'); break;
        case 4: context.go('/customers'); break;
        case 5: context.go('/events'); break;
        case 6: context.go('/admin'); break;
      }
    } else {
      switch (index) {
        case 0: context.go('/dashboard'); break;
        case 1: context.go('/pos'); break;
        case 2: context.go('/inventory'); break;
        case 3: context.go('/customers'); break;
        case 4: Scaffold.of(context).openDrawer(); break;
      }
    }
  }
}
