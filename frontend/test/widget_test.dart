import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';
import 'package:vaultsync_mobile/src/app.dart';
import 'package:vaultsync_mobile/src/providers/product_provider.dart';
import 'package:vaultsync_mobile/src/providers/customer_provider.dart';
import 'package:vaultsync_mobile/src/providers/cart_provider.dart';
import 'package:vaultsync_mobile/src/services/api_service.dart';

void main() {
  testWidgets('App smoke test', (WidgetTester tester) async {
    final apiService = ApiService();

    await tester.pumpWidget(
      MultiProvider(
        providers: [
          Provider<ApiService>.value(value: apiService),
          ChangeNotifierProvider(create: (_) => ProductProvider(apiService)),
          ChangeNotifierProvider(create: (_) => CustomerProvider(apiService)),
          ChangeNotifierProvider(create: (_) => CartProvider(apiService)),
        ],
        child: const VaultSyncApp(),
      ),
    );

    // Verify that we start at the login screen
    expect(find.text('VaultSync'), findsOneWidget);
    expect(find.text('Login'), findsOneWidget);
  });
}
