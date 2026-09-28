# g3-route-transitions

[![CI](https://github.com/g3techhq/g3-route-transitions/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/g3techhq/g3-route-transitions/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/g3-route-transitions.svg)](https://crates.io/crates/g3-route-transitions)
[![docs.rs](https://docs.rs/g3-route-transitions/badge.svg)](https://docs.rs/g3-route-transitions)
[![License](https://img.shields.io/crates/l/g3-route-transitions.svg)](#license)

Native-feeling page transitions for Dioxus Router, declared next to your routes.

<p align="center">
  <a href="https://github.com/g3techhq/g3-ui/raw/main/docs/media/route-transitions-demo.mp4">
    <img
      src="https://raw.githubusercontent.com/g3techhq/g3-ui/main/docs/media/route-transitions-demo.webp"
      alt="g3-route-transitions running inside the g3-ui mobile device frame"
      width="390"
    >
  </a>
</p>

A [live demo](https://g3ui.g3tech.net/transitions) runs inside the `g3-ui`
components these transitions were designed for.

## How it fits together

You need all three pieces before every transition is visible:

1. **Route metadata** decides *which* transition runs.
   `#[derive(RouteTransitions)]` reads `#[transition(...)]` on your `Routable`
   enum.
2. **Snapshot regions** decide *what moves*. Wrapper components mark the
   parts of the page that each transition animates.
3. **Animated navigation** captures the old page *before* the route changes:
   `animated_navigate`, `try_animated_back`, and `animated_back_or_navigate`.
   Calling `navigator().push(...)` or `go_back()` directly skips the
   animation.

The transitions use the browser's
[View Transitions API](https://developer.mozilla.org/docs/Web/API/View_Transition_API).
Where it is unavailable, or the user prefers reduced motion, routes change
instantly.

## Install

```toml
[dependencies]
dioxus = { version = "0.7.9", features = ["router"] }
g3-route-transitions = "0.4"
```

Enable `native-back` to animate the native Back action on Android and iOS:

```toml
g3-route-transitions = { version = "0.4", features = ["native-back"] }
```

## Quick start

```rust,ignore
use dioxus::prelude::*;
use g3_route_transitions::{
    RouteTransitionApp, RouteTransitionBaseRegion, RouteTransitionPage, RouteTransitions,
    animated_back_or_navigate, animated_navigate,
};

#[derive(Clone, PartialEq, Routable, RouteTransitions)]
enum Route {
    #[transition(layer = stack_root)]
    #[route("/")]
    Home {},

    #[transition(layer = stack_page)]
    #[route("/details/:id")]
    Details { id: u32 },

    #[transition(layer = sheet)]
    #[route("/compose")]
    Compose {},
}

#[component]
fn App() -> Element {
    // Loads the stylesheet and provides the overlay region that sheets use.
    rsx! { RouteTransitionApp { Router::<Route> {} } }
}

// Ordinary pages: a base region (stays put, dims under sheets) around a page
// region (slides for Forward/Backward).
#[component]
fn Home() -> Element {
    rsx! {
        RouteTransitionBaseRegion {
            RouteTransitionPage {
                button {
                    onclick: move |_| async move { animated_navigate(Route::Details { id: 1 }).await },
                    "Open details"
                }
                button {
                    onclick: move |_| async move { animated_navigate(Route::Compose {}).await },
                    "Compose"
                }
            }
            // Persistent chrome such as a tab bar goes here, outside the page.
        }
    }
}

#[component]
fn Details(id: u32) -> Element {
    rsx! {
        RouteTransitionBaseRegion {
            RouteTransitionPage {
                button {
                    onclick: move |_| async move { animated_back_or_navigate(Route::Home {}).await },
                    "Back"
                }
                "Details {id}"
            }
        }
    }
}

// Sheets: no base or page region, so the whole sheet rises with the overlay.
#[component]
fn Compose() -> Element {
    rsx! {
        button {
            onclick: move |_| async move { animated_back_or_navigate(Route::Home {}).await },
            "Close"
        }
    }
}
```

This example is compiled as part of the crate documentation. The result:

| You navigate | You see | Back shows |
|---|---|---|
| Home → Details | `Forward`: Details slides in from the right | `Backward` |
| Home or Details → Compose | `PresentSheet`: Compose rises and the page underneath dims | `DismissSheet` |
| Details 1 → Details 2 | `CrossFade`: nothing relates two `stack_page` values | `Backward` (Back only looks at the route being left) |

## Declaring routes

Derive `RouteTransitions` next to `Routable`, and add `#[transition(...)]` only
where a route needs more than the default. Hover `RouteTransitions` in
rust-analyzer to see the full reference inline.

### Options

| Option | Meaning |
|---|---|
| *(none)* / `layer = base` | Ordinary page with no stack or sheet role. |
| `layer = stack_root` | Root of a navigation stack, such as a bottom-tab destination. |
| `layer = stack_page` | Full-screen page pushed above a stack root. |
| `layer = sheet` | Routed sheet presented over whatever page was showing. |
| `forward_to = Route` or `(A, B)` | Directed drill-down. Moving to a listed variant is `Forward`, and moving from it back to this variant is `Backward`. |
| `peers(group = g, order = field)` | Ordered siblings such as tabs. A greater `order` is `Forward`, a smaller one `Backward`, an equal one `None`. |
| `peers(group = g, key = field, order = field)` | As above, but only routes whose `key` fields are equal are peers. `key = (a, b)` compares several fields. |
| `history = replace` | Moving between two values of this variant replaces the history entry. |
| `history = replace(key = field)` | As above, but only when the `key` fields are equal. |
| `handoff_from = Route` or `(A, B)` | Arriving here from a listed variant replaces its history entry, so Back skips it. |

Each option may appear once per variant. `forward_to` and `handoff_from` must
name *other* variants. `key` fields need `PartialEq`, and `order` fields need
`Ord`. Every variant in a `peers` group needs fields with those names and
compatible types. Tuple variants are not supported.

### Which transition will run?

`animated_navigate(next)` checks these rules in order, and the first match
wins:

| # | When | Transition | History |
|---|---|---|---|
| 1 | `next == current` | none; navigation is skipped | unchanged |
| 2 | `current` lists `next` in `forward_to` | `Forward` | push |
| 3 | `next` lists `current` in `forward_to` | `Backward` | push |
| 4 | `next` lists `current` in `handoff_from` | `PresentSheet` if `next` is a `sheet`, `Forward` if it is a `stack_page`, otherwise `CrossFade` | **replace** |
| 5 | non-sheet → `sheet` | `PresentSheet` | push* |
| 6 | `sheet` → non-sheet | `DismissSheet` | push* |
| 7 | `stack_root` → `stack_page` | `Forward` | push* |
| 8 | `stack_page` → `stack_root` | `Backward` | push* |
| 9 | same `peers` group (matching `key`) | `Forward` / `Backward` / `None` by `order` | push* |
| 10 | same variant with matching `history = replace` | `None` (instant) | **replace** |
| 11 | anything else | `CrossFade` | push* |

\* History is replaced instead when rule 4's or rule 10's condition also holds.
For example, tabs with both `peers(...)` and `history = replace` slide *and*
replace history, so Back leaves the screen instead of replaying every tab.

Some consequences of this order:

- **These layer pairs have no built-in motion and cross-fade:**
  `base` ↔ any non-sheet layer, `stack_root` ↔ `stack_root`,
  `stack_page` ↔ `stack_page`, and `sheet` ↔ `sheet`. Add `forward_to` to
  make one stack page slide to another.
- **Mutual `forward_to`:** if two variants list each other, both directions
  are `Forward`.
- **`history = replace` alone** makes same-variant updates (filters, query
  params) instant.

### Back

`try_animated_back()` and `animated_back_or_navigate(fallback)` pop real
router history. The router does not reveal the destination until after the
old page is captured, so **Back looks only at the layer of the route being
left**:

| Leaving a… | Back transition |
|---|---|
| `sheet` | `DismissSheet` |
| `stack_page` | `Backward` |
| `base` or `stack_root` | `CrossFade` |

So Back does not always mirror the forward transition. `forward_to` between
two `base` routes slides forward but cross-fades back. Two unrelated
`stack_page`s cross-fade forward but slide back. Give drill-down
destinations `layer = stack_page` (or `sheet`) when Back should match.

When `animated_back_or_navigate` has no history to pop (after a deep link,
for example), it navigates to `fallback` using the forward rules above.

## What moves: snapshot regions

A transition only animates the regions it knows about. Everything else
belongs to the root snapshot.

| Component | Snapshot name | Role |
|---|---|---|
| `RouteTransitionApp` | `overlay` | App-level wrapper: links the stylesheet and provides the overlay region. |
| `RouteTransitionOverlayRegion` | `overlay` | The part that rises or falls for sheets. Use it directly only in a hand-built shell. |
| `RouteTransitionBaseRegion` | `base` | A non-sheet page's shell. It stays put during navigation and dims under sheets. |
| `RouteTransitionPage` | `page` | A page's header and body, captured as one image. This is the part that slides for `Forward`/`Backward`. |
| `RouteTransitionSegment` | `segment` | Content that slides by itself, such as tab bodies under a fixed header. |
| `RouteTransitionPersistent` | `persistent` | Chrome that never moves and paints above everything, including a rising sheet, such as a desktop navigation rail. |
| `ROUTE_TRANSITION_PAGE_FRAME_CLASS` | `page`, under sheets | A shell around a page that holds chrome a sheet should cover, such as a phone's bottom tab bar. During sheet transitions the frame, not the bare page, is what dims under the sheet. |
| `RouteTransitionStyles` | – | Only links the stylesheet. Use it with a hand-built overlay region. |

| Transition | What animates | What does not |
|---|---|---|
| `CrossFade` | root and `base` cross-fade | – |
| `Forward` / `Backward` | `page` slides with platform motion; a `segment` outside any page slides full-width | root and `base` switch instantly |
| `PresentSheet` / `DismissSheet` | `overlay` rises or falls; `base` and `page` underneath dim (and scale on iOS) | root is hidden |
| `None` | nothing; no View Transition runs | – |

In every transition, `persistent` stays exactly in place above the other snapshots. It fades only if one of the two routes does not render it.

### Layout rules

- **Ordinary pages:** `RouteTransitionBaseRegion { RouteTransitionPage { header, body }, tab_bar }`.
  The page slides and the tab bar stays still.
- **Sheet routes:** render *no* base or page region. Those are captured
  separately from the overlay, so the sheet's content would be missing from
  the rising image.
- **Segmented content:** wrap the moving body in `RouteTransitionSegment`
  *without* a surrounding `RouteTransitionPage`. Inside a page, segments are
  suppressed and the whole page moves instead.
- **Chrome a sheet covers:** put the page and the chrome in one shell with
  `ROUTE_TRANSITION_PAGE_FRAME_CLASS` (g3-ui's `TabLayout` has it). The page
  still slides alone for pushes, and the tab bar stays under a rising sheet
  instead of vanishing.
- **Persistent chrome:** render it on *both* sides of a transition, sheet
  routes included, in the same place. For example, a sheet route on desktop
  renders the same navigation rail as the page beneath it.
- **One of each:** render at most one of each region at a time. Duplicate
  `view-transition-name`s make the browser skip the animation.
- **`RouteTransitionApp` alone gives only cross-fades.** Stack motion needs
  a page or segment region, and sheets need a base region to rise over.

```rust,ignore
// Tabs whose bodies slide under a fixed header:
rsx! {
    RouteTransitionBaseRegion {
        Header { SegmentedControl {} }
        RouteTransitionSegment { TabBody {} }
        TabBar {}
    }
}
```

Component libraries can place the markers on their own elements with the
public class constants: `ROUTE_TRANSITION_BASE_REGION_CLASS`,
`ROUTE_TRANSITION_OVERLAY_REGION_CLASS`, `ROUTE_TRANSITION_PAGE_CLASS`,
`ROUTE_TRANSITION_SEGMENT_CLASS`, `ROUTE_TRANSITION_PERSISTENT_CLASS`, and
`ROUTE_TRANSITION_PAGE_FRAME_CLASS`. For a
region that should only apply at some breakpoints, such as a rail that is a
bottom bar on phones, set `view-transition-name: persistent` inside your own
media or container query instead.

## How each transition looks

`NavigationTransition` names *what happened*. `Platform` decides how it looks.
Call `set_platform(Platform::Ios)` or `set_platform(Platform::Material)` at
startup, and again whenever the app's mode changes. Otherwise
`get_platform()` falls back to `detect_platform()`: `Ios` on iOS targets and
iPhone/iPad user agents, and `Material` everywhere else.

| Transition | iOS | Material |
|---|---|---|
| `Forward` | Navigation-controller push: the new page slides in from the right over the old one, which shifts 30% left and dims. | Shared Axis X: both pages move 30px left with a sequential fade-through. |
| `Backward` | The reverse pop. | The reverse, moving right. |
| `PresentSheet` | Page sheet: the sheet rises from the bottom while the page beneath scales to 93%, rounds its corners, and dims. | Bottom sheet: the sheet rises 20% while fading in; the page beneath dims in place. |
| `DismissSheet` | The reverse. | The reverse (350ms exit). |
| `CrossFade` | A quick 150ms dissolve. The new page stays opaque underneath, which avoids WebView backdrop flashes. | Same as iOS. |
| Segment `Forward`/`Backward` | A full-width filmstrip with both panes locked together. | Same as iOS. |

Directions are physical: `Forward` always enters from the right, including in
right-to-left documents.

The Material page motion follows the official
[Shared Axis X](https://github.com/material-components/material-components-android/blob/master/docs/theming/Motion.md)
spec: a 30dp slide with fade-through, 300ms, standard easing. The sheet
timings follow the M3 bottom-sheet
[enter](https://github.com/material-components/material-components-android/blob/master/lib/java/com/google/android/material/bottomsheet/res/anim/m3_bottom_sheet_slide_in.xml)
and [exit](https://github.com/material-components/material-components-android/blob/master/lib/java/com/google/android/material/bottomsheet/res/anim/m3_bottom_sheet_slide_out.xml)
resources. The small leading-edge shadow is an added elevation cue.
`CrossFade` and the segment filmstrip are deliberate product choices shared
by both platforms.

UIKit does not publish its animator curves, so the iOS motion reproduces the
visible structure of
[UIKit push/pop and page-sheet presentation](https://developer.apple.com/documentation/uikit/uipresentationcontroller)
rather than exact system timing.

Before each transition, the runtime copies the resolved app background and
the nearest rounded clipping ancestor onto `<html>`. This keeps motion inside
embedded app frames and keeps the iOS sheet backdrop correct in light and
dark mode.

## Styling

### Attributes on `<html>` during a transition

| Attribute | Value |
|---|---|
| `data-route-transition` | `cross-fade`, `forward`, `backward`, `present-sheet`, or `dismiss-sheet` (`NavigationTransition::data_value`) |
| `data-route-transition-platform` | `ios` or `material` (`Platform::data_value`) |
| `data-route-transition-from` | Path of the route being left |
| `data-route-transition-to` | Path of the route being entered. Absent on Back, because the destination is not known yet. |

Use these to lift extra elements into their own snapshot for specific
transitions:

```css
html[data-route-transition="present-sheet"][data-route-transition-to^="/watch/"] .player {
  view-transition-name: player;
}
```

To override the library's animations, target the snapshot names from the
region table, for example
`html[data-route-transition="forward"]::view-transition-new(page)`.

### Custom properties

Load overrides after the library stylesheet. Scope platform-specific values
with `html[data-route-transition-platform="ios"]` or `"material"`.

| Property | Default (iOS / Material) | Controls |
|---|---|---|
| `--route-transition-bg` | falls back to `--color-bg`, then `#f8f8f8` | Background of `html`, `body`, and the regions |
| `--route-transition-stack-duration` | `260ms` / `300ms` | `Forward` / `Backward` page motion |
| `--route-transition-segment-duration` | stack duration | Segment filmstrip |
| `--route-transition-sheet-duration` | `0.6s` / `400ms` | Sheet presentation (and iOS dismissal) |
| `--route-transition-material-sheet-dismiss-duration` | – / `350ms` | Material sheet dismissal |
| `--route-transition-fade-duration` | `150ms` | `CrossFade` |
| `--route-transition-material-shared-axis-distance` | `30px` | Material page travel |
| `--route-transition-ios-presentation-backdrop` | app surface mixed 72% with black | Area exposed around the scaled iOS page |

The runtime sets `--route-transition-document-bg`,
`--route-transition-surface-bg`, `--route-transition-incoming-surface-bg`, and
`--route-transition-clip-radius` during each transition. Do not set these
yourself.

### Global styles

The stylesheet also applies layout rules outside the transition itself:

- `html` and `body` get `min-height: 100%` and the `--route-transition-bg`
  background.
- The base and overlay regions are full-height flex columns
  (`min-height: 100dvh`) with that background.
- `RouteTransitionPage` is a `100dvh` flex column with `overflow: hidden`.

## Navigating

```rust,ignore
button {
    onclick: move |_| async move { animated_navigate(Route::Compose {}).await },
    "Compose"
}
```

- **`animated_navigate(route)`** applies the rule table. It does nothing if
  `route` is already current.
- **`animated_back_or_navigate(fallback)`** is for visible Back buttons. It
  pops history, or navigates to `fallback` at the history root.
- **`try_animated_back()`** pops history and returns `false` when there was
  nothing to pop, so platform Back handlers can keep their own root behavior.

All three must run beneath `Router::<Route>`, and their futures resolve when
the animation finishes. Internally, JavaScript captures the old page, asks
Rust to push, replace, or pop the route, waits for the new route to render,
and then lets the animation run.

## Browser Back and Forward

On the web, the browser's own Back and Forward buttons change the route
without a transition unless you opt in. Call one hook in the component that
renders the router, before `Router::<Route>`:

```rust,ignore
use g3_route_transitions::use_browser_history_transitions;

#[component]
fn App() -> Element {
    use_browser_history_transitions::<Route>();
    rsx! { RouteTransitionApp { Router::<Route> {} } }
}
```

The hook wraps the renderer's history, so a base path, hash routing, and
scroll restoration keep working. It delays the router's update until the old
page has been captured. Back uses the same transition as
`try_animated_back`, and Forward replays the original push.

- The Navigation API supplies the direction where the browser supports it.
  Otherwise the hook uses the entries pushed since the page loaded, and a
  traversal it cannot place cross-fades.
- When the browser has already animated the traversal (it sets
  `hasUAVisualTransition`, for example for Safari's edge swipe), the route
  changes without a second animation. Pages cannot turn off that browser
  animation.
- Pops started by `try_animated_back` are not animated twice.
- Builds other than web are unaffected.

## Native Back on Android and iOS

With `native-back` enabled, call one hook from a layout beneath the router:

```rust,ignore
use g3_route_transitions::use_native_back_navigation;

#[component]
fn AppLayout() -> Element {
    use_native_back_navigation::<Route>();
    rsx! { Outlet::<Route> {} }
}
```

The hook prepares `g3-native-plugins`, reusing an existing
`NativePluginsProvider` if the app has one. It takes over the native Back
action (Android's Back gesture or key, and iOS's swipe in from the left screen
edge) only while router history can be popped, and runs the same animated pop
as `try_animated_back`. At the root, Android Back leaves the app and an iOS
edge swipe does nothing, as usual. On the web the hook does nothing; use
`use_browser_history_transitions` there.

Dialogs, sheets, fullscreen players, and other higher-priority UI can claim
the cancelable `g3nativeback` window event by calling
`event.preventDefault()`. If such UI can be open at the root of history, use
`use_native_back_navigation_with_interception::<Route>(is_open)` so the event
still fires. When no layer claims the event and there is no history, the
action goes back to the platform: Android handles it, and iOS drops the swipe.
After an animated pop, the library dispatches `g3routebacktransitionend`
(`NATIVE_BACK_TRANSITION_FINISHED_EVENT`) for work such as restoring scroll
position.

## Migrating from 0.3

| 0.3 | 0.4 |
|---|---|
| `#[route_transitions]` | `#[derive(RouteTransitions)]` |
| `root` / `pushed` | `layer = stack_root` / `layer = stack_page` |
| `cover` | `layer = sheet` |
| `morph` | Removed; unrelated routes use `CrossFade` |
| `push(...)` | `peers(...)` |
| `forward = ...` | `forward_to = ...` |
| `replace(...)` | `history = replace(...)` |
| `replaces = ...` | `handoff_from = ...` |
| `NavigationAnimation` | `NavigationTransition` |
| `PushLeft` / `PushRight` | `Forward` / `Backward` |
| `CoverUp` / `UncoverDown` | `PresentSheet` / `DismissSheet` |
| `Fade` | `CrossFade` |
| `Platform::Md` | `Platform::Material` |
| `RouteTransitionRoot` | `RouteTransitionApp` |
| `RouteTransitionProvider` | `RouteTransitionStyles` |
| `RouteTransitionBase` | `RouteTransitionBaseRegion` |
| `RouteTransitionCover` | `RouteTransitionOverlayRegion` |
| `ROUTE_TRANSITION_BASE_CLASS` | `ROUTE_TRANSITION_BASE_REGION_CLASS` |
| `ROUTE_TRANSITION_COVER_CLASS` | `ROUTE_TRANSITION_OVERLAY_REGION_CLASS` |
| `animated_go_back` | `animated_back_or_navigate` |
| `try_animated_go_back` | `try_animated_back` |

If you wrote custom CSS against the stylesheet, update these too:

| 0.3 | 0.4 |
|---|---|
| `data-route-transition="fade"` | `"cross-fade"` |
| `"push-left"` / `"push-right"` | `"forward"` / `"backward"` |
| `"cover-up"` / `"uncover-down"` | `"present-sheet"` / `"dismiss-sheet"` |
| `"morph-in"` / `"morph-out"` | Removed |
| `data-route-transition-platform="md"` | `"material"` |
| `::view-transition-*(cover)` | `::view-transition-*(overlay)` |
| `.route-transition-base` / `.route-transition-cover` | `.route-transition-base-region` / `.route-transition-overlay-region` |

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
