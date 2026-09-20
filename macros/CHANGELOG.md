# Changelog

All notable changes to `g3-route-transitions-macros` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-09-16

### Changed

- Replaced the `route_transitions` attribute macro with the
  `RouteTransitions` derive and registered `transition` as its helper
  attribute. The derive's rustdoc contains the full option reference for editor
  hover help.
- Replaced ambiguous flat metadata with `layer = ...`, `peers(...)`,
  `forward_to`, `history = replace`, and `handoff_from`. Routed sheets use
  `layer = sheet`; the unused morph/zoom layer was removed.
- Generated helper methods are now prefixed with `__g3_route_` and hidden
  from docs, so they cannot collide with methods on the route enum.
- A repeated `#[transition(...)]` or `peers(...)` argument is now a compile
  error instead of silently overriding the earlier value.
- `forward_to` naming its own variant is now a compile error.

### Documentation

- The derive reference now lists the exact rule order, the history
  behavior of each rule, layer pairs that cross-fade, and the Back table.
- The README links to the runtime crate by absolute URL so it resolves on
  crates.io.

## [0.3.0] - 2026-09-13

### Added

- Route metadata accepts `replaces = Route` handoff rules, allowing one route
  to replace another route's history entry while using the destination's
  transition layer.

## [0.1.0] - 2026-09-06

Initial release.

- Added `root` and `pushed` route layers.
- Added directed `forward` route relationships.
- Added unkeyed and identity-keyed `replace` rules for in-place routes.
- Generated layer-aware back-transition behavior.
- `route_transitions` attribute macro, generating a `RouteTransitions`
  implementation from per-route animation declarations.

This crate is an implementation detail of `g3-route-transitions` and carries
no stability guarantee of its own.
