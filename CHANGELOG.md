# Changelog

All notable changes to `g3-route-transitions` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.2] - 2026-09-28

### Fixed

- Navigation no longer hangs on a page the browser does not paint: a
  background tab, or an occluded window or preview pane that still reports
  itself visible. The browser captures the old page on its next rendering
  opportunity and only then runs the update that changes the route, so
  neither happened and `animated_navigate` never returned. If the update has
  not started within a second, the transition is skipped, which still runs
  it: the route changes without motion.

## [0.4.1] - 2026-09-22

### Fixed

- Persistent chrome was stacked *below* a routed sheet, contradicting the
  documented promise that it "stays in place above all other snapshots, such
  as a desktop rail beside a rising sheet". A sheet covered a desktop
  navigation rail for the length of the transition, and the live rail then
  popped back on top the moment the transition ended. The sheet-specific
  `z-index` overrides are gone, so persistent chrome now outranks the overlay
  as documented. Chrome that a sheet *should* cover, such as a phone's bottom
  tab bar, must not be marked persistent; mark it only at the widths where it
  should stay reachable and let it fall back into the base region elsewhere.

## [0.4.0] - 2026-09-21

### Added

- `use_browser_history_transitions::<Route>()` animates the browser's Back and
  Forward buttons on the web. It is opt-in: call it in the component that
  renders the router. It wraps the renderer's history and delays the router
  update until the old page has been captured. Traversals the browser already
  animated (`hasUAVisualTransition`) are not animated again.
- `RouteTransitionPersistent` and `ROUTE_TRANSITION_PERSISTENT_CLASS` mark
  chrome that stays in place during every transition, such as a desktop
  navigation rail. It paints above a rising sheet instead of dimming with the
  page, and fades when only one route renders it. The `persistent` snapshot
  name can also be set directly inside a breakpoint.

### Changed

- `use_native_back_navigation` and
  `use_native_back_navigation_with_interception` now work on iOS as well as
  Android. With `native-back` enabled, an iOS swipe in from the left edge runs
  the same animated router pop as Android's Back, using the iOS bridge that
  `g3-native-plugins` provides. At the root the swipe does nothing, as before.
- Replaced the `#[route_transitions]` attribute with the idiomatic
  `#[derive(RouteTransitions)]`. The derive registers `#[transition(...)]` and
  carries the complete option reference so rust-analyzer can show it on hover.
- Rescoped route metadata by responsibility: `layer = stack_root`,
  `layer = stack_page`, `layer = sheet`, `peers(...)`,
  `forward_to`, `history = replace`, and `handoff_from` replace the ambiguous
  `root`, `pushed`, `cover`, `push`, `forward`, `replace`, and
  `replaces` spellings.
- Renamed `NavigationAnimation` to `NavigationTransition`; its variants are now
  semantic (`Forward`, `Backward`, `PresentSheet`, `DismissSheet`, and
  `CrossFade`) rather than encoding CSS direction or implementation details.
- Renamed route layers to `StackRoot`, `StackPage`, and `Sheet`, and
  renamed `Platform::Md` to `Platform::Material`. The corresponding platform
  data value is now `material` instead of `md`.
- Renamed the setup components to `RouteTransitionApp` and
  `RouteTransitionStyles`, and the explicit region components to
  `RouteTransitionBaseRegion` and `RouteTransitionOverlayRegion`. Their public
  class constants and CSS selectors now use the same region vocabulary.
- Renamed Back helpers to `try_animated_back` and
  `animated_back_or_navigate`, making the latter's fallback navigation explicit.
- Renamed public motion custom properties around the same vocabulary, including
  `--route-transition-stack-duration` and `--route-transition-sheet-duration`.
- Removed the unused route-level morph/zoom transition and its stylesheet.
- Renamed the `data-route-transition` values to match the enum: `fade`,
  `push-left`, `push-right`, `cover-up`, and `uncover-down` are now
  `cross-fade`, `forward`, `backward`, `present-sheet`, and `dismiss-sheet`.
  The sheet snapshot is now named `overlay` instead of `cover`, and internal
  `md-` keyframes use `material-`.
- `CrossFade` now gives the base region the same fade as the root snapshot
  (the outgoing image fades while the incoming one stays opaque) instead of
  the browser's default cross-fade.
- The derive rejects repeated transition arguments and `forward_to` pointing
  at its own variant.

### Documentation

- Rewrote the README and crate rustdoc as a consumer guide: the full rule
  order with history behavior, layer pairs that cross-fade, the Back table
  and where Back differs from forward navigation, which snapshot regions each
  transition animates, layout rules for pages, sheets, and segments, the CSS
  contract (attributes, values, snapshot names, custom properties, global
  styles), and a CSS migration table.
- The quick-start example is now a compiled doctest.

### Fixed

- Native Back only worked once per launch. Dioxus's Android and iOS renderers
  close an `eval` channel when its script returns, so the bridge never
  received Rust's reply to the first press and ignored every press after it.
  The bridge now stays alive until it is replaced.

- `animated_navigate` and `try_animated_back` no longer call the
  `use_navigator` hook from inside async tasks, which added a hook slot to the
  calling component on every navigation.

- iOS sheet dismissal now derives its exposed presentation backdrop from the
  incoming page, preventing a light outgoing sheet from flashing behind a
  dark-mode page during sheet dismissal.

## [0.3.0] - 2026-09-13

### Added

- `#[transition(replaces = Route)]` hands a route off to another. Navigating
  from a listed route replaces its history entry and animates by the
  destination's layer, so a cover opened from another cover rises instead of
  fading, and Back returns to the page under both instead of reopening the first.
- Transitions publish `data-route-transition-from` and
  `data-route-transition-to` on `<html>` for their duration, set before the
  outgoing snapshot and cleared afterwards. `to` is omitted on Back, whose
  destination is unknown until the router pops. App CSS can use them to scope
  snapshot naming to the routes a transition moves between.

## [0.2.0] - 2026-09-08

### Changed

- The executable transition showcase now lives in the `g3-ui` playground, so
  every recorded transition uses the real component library instead of a
  second set of standalone demo components.
- Spatial page and cover snapshots now clip to the app shell's resolved bounds,
  so transitions stay inside embedded frames such as the playground phone.
- The transition runtime now carries the outgoing document and route-surface
  colors into the document-level snapshot tree. Cover transitions retain the
  surrounding site's background, and iOS sheet scaling no longer exposes a
  white backdrop behind dark-mode pages.
- The route playground now uses the published `g3-ui` shell, theme, and
  segmented control while continuing to exercise this crate's low-level
  snapshot markers directly.
- `replace` now governs history only, not motion. It still yields
  `NavigationAnimation::None` on its own, but a variant that declares both
  `replace` and `push` slides left or right *and* replaces the history entry,
  instead of the replace short-circuit discarding the push ordering. This is
  what a segmented control needs: the body tracks the selected tab while Back
  leaves the screen rather than retracing every tab the user touched. Routes
  declaring `replace` without `push` are unaffected.

## [0.1.0] - 2026-09-06

Initial release.

- Added the optional `native-back` integration with `g3-native-plugins`,
  including hooks that connect Android system Back directly to animated Dioxus
  router history without app-specific Rust/JavaScript bridges.
- Added `try_animated_go_back`, which preserves the operating system's root
  behavior when no router history exists.
- Back animation selection now depends only on the current route layer rather
  than a fallback route that may not be the actual history destination.
- Added `root` and `pushed` route layers plus directed `forward` edges for
  navigation-stack transitions.
- Added `replace` route metadata so same-page query/filter updates replace
  browser history without animating.
- Added layer-aware back animations while continuing to pop the browser's
  actual previous entry through the Rust/JavaScript acknowledgement bridge.
- Added `RouteTransitionPage` for one stable full-viewport snapshot around
  shells with nested transition markers.
- Made iOS and Material peer fades the same faster cross-dissolve, and made
  Material sheet dismissal faster with full downward travel.
- Route-owned View Transition helpers for Dioxus Router: `animated_navigate`,
  the `RouteTransitions` trait, and the `route_transitions` macro.
- Eight navigation animations (fade, push, cover/uncover, morph) that each
  render with iOS or Material motion depending on the active `Platform`.
- `RouteTransitionProvider` and `RouteTransitionRoot` for wiring the
  stylesheet and layer classes.
- Honors `prefers-reduced-motion`, falling back to an immediate navigation.
