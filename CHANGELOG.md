# Changelog

All notable changes to `dx-route-transitions` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

## [0.1.0] - 2026-08-20

Initial release.

- Route-owned View Transition helpers for Dioxus Router: `animated_navigate`,
  the `RouteTransitions` trait, and the `route_transitions` macro.
- Eight navigation animations (fade, push, cover/uncover, morph) that each
  render with iOS or Material motion depending on the active `Platform`.
- `RouteTransitionProvider` and `RouteTransitionRoot` for wiring the
  stylesheet and layer classes.
- Honors `prefers-reduced-motion`, falling back to an immediate navigation.
