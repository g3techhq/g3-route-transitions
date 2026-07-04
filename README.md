# dx-route-transitions

Route-owned View Transition helpers for Dioxus Router.

This crate keeps route animation rules next to your `Routable` enum, then exposes navigation helpers and explicit snapshot marker components:

- `#[route_transitions]` derives transition metadata from route variants.
- `animated_navigate(route)` computes the animation from the current route and pushes the next route.
- `RouteTransitionProvider` imports the default View Transition stylesheet.
- `RouteTransitionRoot` wraps the app shell with the provider and cover marker.
- `RouteTransitionBase`, `RouteTransitionCover`, and `RouteTransitionSegment` mark named snapshot regions explicitly.

## Install

```toml
[dependencies]
dioxus = { version = "0.7.9", features = ["router"] }
dx-route-transitions = "0.1"
```

The Rust crate name is `dx_route_transitions`:

```rust
use dx_route_transitions::{animated_navigate, route_transitions, RouteTransitionRoot};
```

## Define Route Metadata

Add `#[route_transitions]` to the same enum that derives `Routable`. Use `#[transition(...)]` on variants that need non-default behavior.

```rust,ignore
use dioxus::prelude::*;
use dx_route_transitions::route_transitions;

#[route_transitions]
#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[transition(base, push(group = tabs, order = tab))]
    #[route("/items?:tab")]
    Items { tab: ItemsTab },

    #[transition(cover)]
    #[route("/items/new")]
    NewItem {},

    #[transition(cover, push(group = item_details, key = item_id, order = tab))]
    #[route("/items/:item_id?:tab")]
    ItemDetails { item_id: String, tab: ItemTab },
}
```

Transition rules:

- `base` marks a normal page. It is the default.
- `cover` marks a sheet or modal route. Base-to-cover uses `CoverUp`; cover-to-base uses `UncoverDown`.
- `push(group = name, order = field)` marks ordered peer routes.
- `key = field` or `key = (field_a, field_b)` scopes a push group to one logical entity.
- Routes with no more specific match fall back to `Fade`.

## Mark Snapshot Regions

`RouteTransitionRoot` is the common app-shell wrapper. It loads the transition CSS and marks the shell as the cover snapshot.

```rust,ignore
use dx_route_transitions::RouteTransitionRoot;

#[component]
fn App() -> Element {
    rsx! {
        RouteTransitionRoot {
            Router::<Route> {}
        }
    }
}
```

For more explicit layouts, use the marker components:

```rust,ignore
use dx_route_transitions::{RouteTransitionBase, RouteTransitionSegment};

rsx! {
    RouteTransitionBase { nav { "Tabs or base page" } }
    RouteTransitionSegment { main { Outlet::<Route> {} } }
}
```

The marker class constants are also public for libraries that need to place the marker on an existing element without adding a wrapper:

- `ROUTE_TRANSITION_BASE_CLASS`
- `ROUTE_TRANSITION_COVER_CLASS`
- `ROUTE_TRANSITION_SEGMENT_CLASS`

## Navigate

```rust,ignore
button {
    onclick: move |_| spawn(async move {
        animated_navigate(Route::NewItem {}).await;
    }),
    "New item"
}
```

If the browser does not support `document.startViewTransition` or the user prefers reduced motion, navigation falls back to a normal router push.