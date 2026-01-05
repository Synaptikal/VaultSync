# VaultSync Design System

## 1. Visual Identity

VaultSync employs a "Premium Collector" aesthetic, inspired by graded card slabs, high-security vaults, and exclusive membership clubs.

### Color Palette

| Name | Hex | Usage |
|------|-----|-------|
| **Vault Navy** | `#1A237E` | Primary Brand, App Bars, Buttons |
| **Collector Gold** | `#FFD700` | Accents, High-value items, Icons |
| **Mythic Purple** | `#9C27B0` | Special Actions, Reports |
| **Slab White** | `#FFFFFF` | Card Backgrounds (Light Mode) |
| **Vault Dark** | `#121212` | Backgrounds (Dark Mode) |

### Typography

*   **Headlines:** Sans-serif, Bold weights (600-700). Used for page titles and stat values.
*   **Body:** Sans-serif, Regular/Medium weights (400-500). Optimized for readability.
*   **Labels:** All-caps, tracked out (letter-spacing: 1.2). Used for "Slab" headers.

## 2. Component Library

### Slab Stat Card
A unique widget mimicking a graded collectible case.
- **Header:** Colored strip with Label (left) and Grade (right box).
- **Body:** White/Dark surface with icon and large value text.
- **Usage:** Dashboard statistics, Inventory highlights.

### Action Card
Square or rectangular buttons with soft pastel backgrounds and deep-colored icons.
- **Interaction:** Ink ripple on tap.
- **Usage:** Dashboard Quick Actions.

### Navigation
- **Desktop:** Left-side `NavigationRail` with "Vault" branding.
- **Mobile:** Bottom `NavigationBar`.

## 3. Responsive Layouts

The application uses a unified `MainLayout` shell that adapts:
- **Mobile (< 800px):** Bottom Navigation, Single column views.
- **Desktop (> 800px):** Left Rail, Grid/Multi-column views.

## 4. Accessibility (WCAG 2.1 AA)

- **Contrast:** All text meets 4.5:1 contrast ratio against backgrounds.
- **Touch Targets:** All interactive elements are at least 48x48dp.
- **Semantics:** Icons are paired with labels or have tooltips.

## 5. Animations

- **Page Transitions:** Standard platform transitions (Slide on iOS, Fade/Zoom on Android).
- **Hero:** Used for product images moving from list to details.
- **Feedback:** Standard Material ripples for touch feedback.
