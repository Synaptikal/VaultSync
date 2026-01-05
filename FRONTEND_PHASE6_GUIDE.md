# Frontend Polish & Testing Guide (PHASE 6)

**Objective:** Bring the VaultSync mobile app to production-grade quality  
**Duration:** 2-3 days  
**Status:** Implementation Guide  

---

## Part 1: Error Message Polish

### Current State
```dart
// Generic, unhelpful
catch (e) {
  showSnackBar('Something went wrong');
}
```

### Target State
```dart
// Specific, actionable
on NetworkException catch (e) {
  showSnackBar('No internet connection. Changes saved locally and will sync when online.');
}
on ValidationException catch (e) {
  showDialog(
    title: 'Invalid Input',
    message: e.message,  // "Barcode must be 12 digits"
    actions: [FocusField('barcode')],
  );
}
```

### Implementation Checklist

#### ✅ 1. Create Error Message Helper
**File:** `lib/src/shared/helpers/error_messages.dart`

```dart
class ErrorMessages {
  static String forException(Exception e) {
    if (e is NetworkException) {
      return 'No internet connection. Your changes are saved locally and will sync automatically when you\'re back online.';
    }
    if (e is AuthenticationException) {
      return 'Your session has expired. Please log in again.';
    }
    if (e is ValidationException) {
      return e.message; // Already user-friendly from backend
    }
    if (e is ConflictException) {
      return 'A conflict was detected with changes from another terminal. Please review and resolve.';
    }
    if (e is ServerException) {
      return 'The server is experiencing issues. Please try again in a few moments.';
    }
    return 'An unexpected error occurred. Please try again.';
  }

  static IconData iconForException(Exception e) {
    if (e is NetworkException) return Icons.wifi_off;
    if (e is AuthenticationException) return Icons.lock;
    if (e is ValidationException) return Icons.error_outline;
    if (e is ConflictException) return Icons.warning_amber;
    return Icons.error;
  }

  static Color colorForException(Exception e) {
    if (e is NetworkException) return Colors.orange;
    if (e is AuthenticationException) return Colors.red;
    if (e is ValidationException) return Colors.blue;
    if (e is ConflictException) return Colors.orange;
    return Colors.red;
  }
}
```

#### ✅ 2. Create Enhanced SnackBar Widget
**File:** `lib/src/shared/widgets/enhanced_snackbar.dart`

```dart
class EnhancedSnackBar {
  static void show(
    BuildContext context, {
    required String message,
    required SnackBarType type,
    Duration duration = const Duration(seconds: 4),
    VoidCallback? action,
    String? actionLabel,
  }) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Row(
          children: [
            Icon(_iconFor(type), color: Colors.white),
            const SizedBox(width: 12),
            Expanded(child: Text(message)),
          ],
        ),
        backgroundColor: _colorFor(type),
        duration: duration,
        action: action != null
            ? SnackBarAction(
                label: actionLabel ?? 'Action',
                textColor: Colors.white,
                onPressed: action,
              )
            : null,
        behavior: SnackBarBehavior.floating,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(8),
        ),
      ),
    );
  }

  static IconData _iconFor(SnackBarType type) {
    switch (type) {
      case SnackBarType.success:
        return Icons.check_circle;
      case SnackBarType.error:
        return Icons.error;
      case SnackBarType.warning:
        return Icons.warning;
      case SnackBarType.info:
        return Icons.info;
    }
  }

  static Color _colorFor(SnackBarType type) {
    switch (type) {
      case SnackBarType.success:
        return Colors.green;
      case SnackBarType.error:
        return Colors.red;
      case SnackBarType.warning:
        return Colors.orange;
      case SnackBarType.info:
        return Colors.blue;
    }
  }
}

enum SnackBarType { success, error, warning, info }
```

---

## Part 2: Loading States & Animations

### Implementation Checklist

#### ✅ 1. Create Shimmer Loading Widget
**File:** `lib/src/shared/widgets/shimmer_loading.dart`

```dart
import 'package:flutter/material.dart';

class ShimmerLoading extends StatefulWidget {
  final Widget child;
  final bool isLoading;

  const ShimmerLoading({
    Key? key,
    required this.child,
    required this.isLoading,
  }) : super(key: key);

  @override
  State<ShimmerLoading> createState() => _ShimmerLoadingState();
}

class _ShimmerLoadingState extends State<ShimmerLoading>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1500),
    )..repeat();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (!widget.isLoading) return widget.child;

    return AnimatedBuilder(
      animation: _controller,
      builder: (context, child) {
        return ShaderMask(
          shaderCallback: (bounds) {
            return LinearGradient(
              begin: Alignment.topLeft,
              end: Alignment.bottomRight,
              colors: const [
                Color(0xFFEBEBF4),
                Color(0xFFF4F4F4),
                Color(0xFFEBEBF4),
              ],
              stops: [
                _controller.value - 0.3,
                _controller.value,
                _controller.value + 0.3,
              ],
            ).createShader(bounds);
          },
          child: widget.child,
        );
      },
    );
  }
}

// Skeleton widgets for common patterns
class SkeletonListItem extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: Row(
        children: [
          Container(
            width: 48,
            height: 48,
            decoration: BoxDecoration(
              color: Colors.grey[300],
              shape: BoxShape.circle,
            ),
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Container(
                  width: double.infinity,
                  height: 16,
                  color: Colors.grey[300],
                ),
                const SizedBox(height: 8),
                Container(
                  width: 150,
                  height: 12,
                  color: Colors.grey[300],
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
```

#### ✅ 2. Add Loading State to Screens

**Example: ProductListScreen**
```dart
// Before
if (_isLoading) {
  return CircularProgressIndicator();
}

// After
if (_isLoading) {
  return ListView.builder(
    itemCount: 10,
    itemBuilder: (context, index) => ShimmerLoading(
      isLoading: true,
      child: SkeletonListItem(),
    ),
  );
}
```

#### ✅ 3. Add Page Transitions

**File:** `lib/src/shared/navigation/page_transitions.dart`

```dart
class SlideRightRoute extends PageRouteBuilder {
  final Widget page;

  SlideRightRoute({required this.page})
      : super(
          pageBuilder: (context, animation, secondaryAnimation) => page,
          transitionsBuilder: (context, animation, secondaryAnimation, child) {
            const begin = Offset(1.0, 0.0);
            const end = Offset.zero;
            const curve = Curves.easeInOut;

            var tween = Tween(begin: begin, end: end).chain(
              CurveTween(curve: curve),
            );

            return SlideTransition(
              position: animation.drive(tween),
              child: child,
            );
          },
        );
}

// Usage
Navigator.of(context).push(
  SlideRightRoute(page: ConflictResolutionScreen()),
);
```

---

## Part 3: Integration Tests

### Test Structure

```
test/
├── integration/
│   ├── offline_flow_test.dart
│   ├── conflict_resolution_test.dart
│   ├── audit_submission_test.dart
│   └── sync_queue_test.dart
├── widget/
│   ├── sync_status_indicator_test.dart
│   └── conflict_card_test.dart
└── unit/
    ├── repository_test.dart
    └── sync_queue_service_test.dart
```

#### ✅ 1. Offline Flow Test
**File:** `test/integration/offline_flow_test.dart`

```dart
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';

void main() {
  group('Offline Product Creation Flow', () {
    late ProductRepository repository;
    late MockConnectivityService connectivity;
    late MockSyncQueueService syncQueue;

    setUp(() {
      connectivity = MockConnectivityService();
      syncQueue = MockSyncQueueService();
      repository = ProductRepository(
        remote: MockRemoteDataSource(),
        local: MockLocalDataSource(),
        connectivity: connectivity,
      );
    });

    test('saves locally when offline', () async {
      // Arrange
      when(connectivity.isOnline).thenAnswer((_) async => false);

      final product = Product(
        productUuid: 'test-123',
        name: 'Test Product',
        category: Category.tradingCard,
      );

      // Act
      final result = await repository.create(product);

      // Assert
      expect(result.productUuid, 'test-123');
      verify(localDataSource.insert(product)).called(1);
      verifyNever(remoteDataSource.create(any));
    });

    test('syncs when coming back online', () async {
      // Arrange
      when(connectivity.isOnline).thenAnswer((_) async => true);
      when(localDataSource.getUnsynced()).thenAnswer(
        (_) async => [testProduct],
      );

      // Act
      final count = await repository.syncPendingChanges();

      // Assert
      expect(count, 1);
      verify(remoteDataSource.create(testProduct)).called(1);
      verify(localDataSource.markSynced(testProduct.productUuid)).called(1);
    });
  });
}
```

#### ✅ 2. Conflict Resolution Test
**File:** `test/integration/conflict_resolution_test.dart`

```dart
void main() {
  group('Conflict Resolution Flow', () {
    testWidgets('displays and resolves conflict', (tester) async {
      // Arrange
      final mockClient = MockApiClient();
      when(mockClient.getPendingConflicts()).thenAnswer(
        (_) async => [
          {
            'conflict_uuid': 'test-conflict',
            'resource_type': 'Product',
            'local_state': {'name': 'Local Name'},
            'remote_state': {'name': 'Remote Name'},
          }
        ],
      );

      // Act
      await tester.pumpWidget(
        MaterialApp(
          home: ConflictResolutionScreen(apiClient: mockClient),
        ),
      );
      await tester.pumpAndSettle();

      // Assert - Conflict card appears
      expect(find.text('Concurrent Modification'), findsOneWidget);
      expect(find.text('Local Name'), findsOneWidget);
      expect(find.text('Remote Name'), findsOneWidget);

      // Act - Resolve conflict
      await tester.tap(find.text('Use Remote'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Confirm'));
      await tester.pumpAndSettle();

      // Assert - API called
      verify(
        mockClient.resolveConflict('test-conflict', 'RemoteWins'),
      ).called(1);

      // Assert - Success message
      expect(find.text('Conflict resolved'), findsOneWidget);
    });
  });
}
```

---

## Part 4: Performance Optimization

### Checklist

#### ✅ 1. Use const Constructors
```dart
// Before
return Text('Hello');

// After
return const Text('Hello');
```

**Find and fix:**
```bash
grep -r "return Text(" lib/ | grep -v "const"
```

#### ✅ 2. Optimize ListView Rendering
```dart
// Use ListView.builder (already done)
ListView.builder(
  itemCount: items.length,
  itemBuilder: (context, index) => ItemWidget(items[index]),
)

// Add cacheExtent for better scrolling
ListView.builder(
  cacheExtent: 500, // Preload 500px ahead
  itemCount: items.length,
  itemBuilder: (context, index) => ItemWidget(items[index]),
)
```

#### ✅ 3. Add RepaintBoundary for Complex Widgets
```dart
@override
Widget build(BuildContext context) {
  return RepaintBoundary(
    child: ComplexWidget(...),
  );
}
```

#### ✅ 4. Profile Performance

Run DevTools performance profiler:
```bash
flutter run --profile
# Open DevTools
# Navigate to Performance tab
# Look for janky frames (>16ms)
```

---

## Implementation Order

### Day 1: Error Messages & Loading States
1. ✅ Create `error_messages.dart` helper
2. ✅ Create `enhanced_snackbar.dart` widget
3. ✅ Update all catch blocks to use new error handling
4. ✅ Create `shimmer_loading.dart`
5. ✅ Add skeleton screens to all list views
6. ✅ Add page transitions

### Day 2: Integration Tests
1. ✅ Set up test infrastructure
2. ✅ Write offline flow test
3. ✅ Write conflict resolution test
4. ✅ Write audit submission test
5. ✅ Write sync queue test
6. ✅ Run all tests and fix failures

### Day 3: Performance & Final Polish
1. ✅ Add const constructors everywhere possible
2. ✅ Optimize ListView rendering
3. ✅ Add RepaintBoundary to complex widgets
4. ✅ Run performance profiler
5. ✅ Fix any identified issues
6. ✅ Final code review
7. ✅ Update documentation

---

## Acceptance Criteria

### Error Handling
- [ ] All API calls have specific error handling
- [ ] Error messages are actionable
- [ ] Network errors show offline message
- [ ] Auth errors navigate to login

### Loading States
- [ ] All lists show skeleton loading
- [ ] No blank screens during load
- [ ] Smooth transitions between states
- [ ] Loading indicators for actions

### Testing
- [ ] 80%+ code coverage
- [ ] All critical paths tested
- [ ] Integration tests pass
- [ ] No memory leaks

### Performance
- [ ] 60 FPS on target devices
- [ ] < 100ms response to user input
- [ ] Smooth scrolling on large lists
- [ ] No janky frames

---

## Completion Checklist

- [ ] Error messages enhanced
- [ ] Loading states added
- [ ] Integration tests written
- [ ] Performance profiled and optimized
- [ ] All tests passing
- [ ] Code reviewed
- [ ] Documentation updated
- [ ] Ready for production deployment

---

**When all items are checked, Phase 6 is COMPLETE and the frontend is production-ready!** 🎉
