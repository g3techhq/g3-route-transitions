# AGENTS.md

Instructions for coding agents working in this repository (Claude Code, Codex,
Cursor, Copilot, and anything else that reads `AGENTS.md`). Useful for people
too. Follow these over your defaults.

## What this is

**g3-route-transitions** animates Dioxus router navigation with the View
Transitions API, in iOS or Material motion. An app declares each route's
motion once, on its `Route` enum (`#[derive(RouteTransitions)]` with
`#[transition(layer = .., history = .., handoff_from = .., forward_to = ..)]`),
and navigates with `animated_navigate` / `animated_back_or_navigate`. No
component contains animation code. It is part of the g3 stack; g3-ui's
`transitions` feature places most snapshot regions, and media-mancer,
greenside-partee, tawny and g3-stack use it.

| Piece | Version | Reference |
| --- | --- | --- |
| Dioxus (router) | 0.7.9 | [dioxuslabs.com/learn/0.7](https://dioxuslabs.com/learn/0.7/), and g3-stack's `docs/dioxus/patterns.md` |
| g3-native-plugins (optional, `native-back`) | 0.4 | Android Back and the iOS edge swipe |
| Rust | edition 2024 | `rust-toolchain.toml` |

**Your training data does not know this crate, and is probably wrong about
Dioxus 0.7.** The README is the user-facing reference and must stay true;
read it before changing behaviour.

## Map

```
src/lib.rs              Transition rules, animated navigation, snapshot-region
                        components, and the navigate script (VIEW_TRANSITION_NAVIGATE)
src/browser_history.rs  use_browser_history_transitions: animated browser Back/Forward
macros/src/lib.rs       #[derive(RouteTransitions)]: parses #[transition(..)] into metadata
tests/route_metadata.rs The rule table, tested against a real Route enum
assets/route_transitions.css  The transition keyframes and snapshot styling
README.md               The reference: options, rule table, regions, Back behaviour
CHANGELOG.md            Every user-visible change, under [Unreleased] until a release
```

The macros crate is published separately (`g3-route-transitions-macros`) and
versioned on its own.

## Commands

```bash
just check        # crate and macros
just test         # nextest + doc tests, crate and macros
just lint-strict  # clippy with warnings as errors, as CI runs it
just format       # rustfmt, crate and macros
just pre-push     # everything above plus typos
```

## Definition of done

1. `just pre-push` passes.
2. A user-visible change has a line under `## [Unreleased]` in `CHANGELOG.md`
   (and the macros' own changelog when the macros change).
3. A change to which transition runs has a test in `tests/route_metadata.rs`,
   and the rule table in the README still matches.
4. A change to the navigate script has a test asserting its ordering (the
   existing script tests in `src/lib.rs` show how), and was exercised in a
   consuming app through a `[patch.crates-io]` path override, removed
   afterwards.

If you could not do one of these, say which and why.

---

## Rules

### The public contract

- **The README's rule table is the specification.** `transition_to`,
  `transition_back` and `replaces_history` must agree with it, and the tests
  pin it. Change the table and the code together, never one alone.
- **Back looks only at the layer being left**; forward navigation applies the
  numbered rules in order. Keep that asymmetry documented where it surprises.
- Renames get a row in the README's migration table and in the CHANGELOG.

### The navigate script

The script runs in the web view and talks to Rust through `dioxus.send` /
`dioxus.recv`. Its hard-won constraints, each with a test:

- **Flush style before `startViewTransition`**, so names granted by the
  transition's attributes exist when the old page is captured.
- **Observe before asking for the route**: the router often renders while the
  ack is in flight, so an observer started afterwards misses the mutation.
- **No `requestAnimationFrame` inside the update callback**: rendering is
  paused there, so a frame never ticks.
- **A page that never paints must still navigate.** A background tab or an
  occluded window can report itself visible and never render, so the old page
  is never captured and the update callback never runs. The watchdog skips
  the transition, which still runs the update.
- Anything the script leaves on `document.documentElement` is removed in its
  `finally`.

### Dioxus

- Hooks run unconditionally, before any early return; `use_effect` keeps its
  first closure.
- No `use_reactive!`; a value a hook follows is read through a signal.
- Never hold a signal borrow across `.await`.
- The animated functions must run beneath `Router::<R>`; say so in any new
  public function's docs.
- Gate calls, never markup: server and web client must render the same tree.

### Releases

- Releases publish from `publish-crates.yml` (crates.io trusted publishing),
  which skips an unchanged macros version. Do not run `cargo publish` by hand,
  and do not release without the maintainer's go-ahead.
- Commits follow Conventional Commits; lefthook checks them.

---

## Verifying

Unit tests cover which transition runs; only a browser shows how it looks.
Patch an app to this checkout (`[patch.crates-io] g3-route-transitions =
{ path = "../g3-route-transitions" }`), run its Playwright tests, and look at
push, sheet and tab transitions in both modes at a phone width and a wide
one. Remove the patch and restore the app's `Cargo.lock` afterwards.

## Where to look

- `README.md`: options, the rule table, snapshot regions, Back on every platform
- `CHANGELOG.md`: what changed between versions
- g3-ui's `transitions` feature, for where the regions are placed in a real shell
