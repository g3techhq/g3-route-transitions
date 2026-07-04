//! Generic View Transition helpers for Dioxus route navigation.
//!
//! This crate is intentionally independent of any app component library. It provides:
//!
//! - [`route_transitions`], an attribute macro for deriving route transition behavior from
//!   route-variant metadata.
//! - [`RouteTransitions`], the trait implemented by the macro and consumed by
//!   [`animated_navigate`].
//! - [`NavigationAnimation`], the animation vocabulary used by the generated route method and the
//!   JS bridge.
//! - [`RouteTransitionProvider`], a small Dioxus provider component that imports the default View
//!   Transition CSS.
//! - [`animated_navigate`], a Dioxus router helper that wraps route changes in
//!   `document.startViewTransition` without waiting for DOM mutations.
//!
//! # Route transition attributes
//!
//! Put `#[route_transitions]` on the same enum that derives Dioxus `Routable`. Then place
//! `#[transition(...)]` on variants that need non-default behavior.
//!
//! ```rust,ignore
//! use dioxus::prelude::*;
//! use dx_route_transitions::route_transitions;
//!
//! #[route_transitions]
//! #[derive(Clone, Routable, PartialEq)]
//! enum Route {
//!     #[transition(base, push(group = sections, order = tab))]
//!     #[route("/sections?:tab")]
//!     Sections { tab: SectionTab },
//!
//!     #[transition(cover)]
//!     #[route("/items/new")]
//!     NewItem {},
//!
//!     #[transition(cover, push(group = item_details, key = item_id, order = tab))]
//!     #[route("/items/:item_id?:tab")]
//!     ItemDetails { item_id: String, tab: ItemTab },
//! }
//! ```
//!
//! Attribute rules:
//!
//! - `base` marks a normal page. It is the default when no transition attribute exists.
//! - `cover` marks a sheet/modal-like route. Navigating `base -> cover` returns
//!   [`NavigationAnimation::CoverUp`]; `cover -> base` returns
//!   [`NavigationAnimation::UncoverDown`].
//! - `push(group = name, order = field)` marks ordered peers inside a static group. The `order`
//!   field must implement [`Ord`]. Moving to a greater order returns `PushLeft`; moving lower
//!   returns `PushRight`.
//! - `key = field` scopes a push group to a route parameter, such as an item id.
//! - `key = (field_a, field_b)` scopes a push group to multiple route parameters.
//! - If two routes are not equivalent, not a cover/uncover pair, and not matching push peers, the
//!   generated method returns [`NavigationAnimation::Fade`].
//!
//! # CSS provider and snapshot markers
//!
//! Wrap the router in [`RouteTransitionProvider`] to load the default CSS. Mark the parts of your
//! app that should participate in named snapshots with these generic markers:
//!
//! - `route-transition-base`: the stable base page under covers.
//! - `route-transition-cover`: the app shell that should slide over or off the base page.
//! - `route-transition-segment`: the body area that should push left/right for peer routes.
//!
//! The runtime names `route-transition-cover` only during cover/uncover transitions so normal
//! rendering and peer-route pushes do not create an extra named snapshot.

use dioxus::{document::eval, prelude::*};
use manganis::{Asset, asset};

pub use dx_route_transitions_macros::route_transitions;

pub static ROUTE_TRANSITIONS_CSS: Asset = asset!("/assets/route_transitions.css");

pub const ROUTE_TRANSITION_BASE_CLASS: &str = "route-transition-base";
pub const ROUTE_TRANSITION_COVER_CLASS: &str = "route-transition-cover";
pub const ROUTE_TRANSITION_SEGMENT_CLASS: &str = "route-transition-segment";

fn merge_transition_class(base: &'static str, extra: Option<&str>) -> String {
    match extra {
        Some(extra) if !extra.is_empty() => format!("{base} {extra}"),
        _ => base.to_string(),
    }
}

#[component]
pub fn RouteTransitionBase(children: Element, class: Option<String>) -> Element {
    let class = merge_transition_class(ROUTE_TRANSITION_BASE_CLASS, class.as_deref());

    rsx! {
        div { class, {children} }
    }
}

#[component]
pub fn RouteTransitionCover(children: Element, class: Option<String>) -> Element {
    let class = merge_transition_class(ROUTE_TRANSITION_COVER_CLASS, class.as_deref());

    rsx! {
        div { class, {children} }
    }
}

#[component]
pub fn RouteTransitionSegment(children: Element, class: Option<String>) -> Element {
    let class = merge_transition_class(ROUTE_TRANSITION_SEGMENT_CLASS, class.as_deref());

    rsx! {
        div { class, {children} }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationAnimation {
    None,
    #[default]
    Fade,
    PushLeft,
    PushRight,
    CoverUp,
    UncoverDown,
}

impl NavigationAnimation {
    pub fn data_value(self) -> &'static str {
        match self {
            NavigationAnimation::None => "none",
            NavigationAnimation::Fade => "fade",
            NavigationAnimation::PushLeft => "push-left",
            NavigationAnimation::PushRight => "push-right",
            NavigationAnimation::CoverUp => "cover-up",
            NavigationAnimation::UncoverDown => "uncover-down",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RouteTransitionLayer {
    #[default]
    Base,
    Cover,
}

pub trait RouteTransitions: PartialEq {
    fn transition_to(&self, next: &Self) -> NavigationAnimation;
}

#[component]
pub fn RouteTransitionProvider(children: Element) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: ROUTE_TRANSITIONS_CSS }
        {children}
    }
}

#[component]
pub fn RouteTransitionRoot(children: Element, class: Option<String>) -> Element {
    let class = merge_transition_class(ROUTE_TRANSITION_COVER_CLASS, class.as_deref());

    rsx! {
        RouteTransitionProvider {
            div { class, {children} }
        }
    }
}

const VIEW_TRANSITION_NAVIGATE: &str = r#"
const animation = await dioxus.recv();
const prefersReducedMotion = window.matchMedia?.("(prefers-reduced-motion: reduce)")?.matches ?? false;
const dxRouteTransitionNextFrame = () => new Promise((resolve) => {
    const raf = window.requestAnimationFrame ?? ((callback) => window.setTimeout(callback, 16));
    raf(() => resolve());
});
const dxRouteTransitionMutationOrFrame = () => new Promise((resolve) => {
    let settled = false;
    const finish = () => {
        if (settled) return;
        settled = true;
        observer?.disconnect?.();
        resolve();
    };
    const observer = typeof MutationObserver === "undefined" ? null : new MutationObserver(() => finish());
    observer?.observe?.(document.body ?? document.documentElement, {
        childList: true,
        subtree: true,
        attributes: true,
    });
    window.setTimeout(() => finish(), 120);
    dxRouteTransitionNextFrame().then(() => dxRouteTransitionNextFrame()).then(() => finish());
});

try {
    if (!document.startViewTransition || prefersReducedMotion) {
        dioxus.send("navigate");
        dioxus.send("done");
    } else {
        const ua = navigator.userAgent?.toLowerCase?.() ?? "";
        const coarsePointer = window.matchMedia?.("(hover: none), (pointer: coarse), (any-pointer: coarse)")?.matches ?? false;
        const nativeMobile = /android|iphone|ipad|ipod/.test(ua);

        document.documentElement.dataset.routeTransition = animation;
        document.documentElement.dataset.routeTransitionPlatform = nativeMobile || coarsePointer ? "mobile" : "web";

        const transition = document.startViewTransition(async () => {
            dioxus.send("navigate");

            const routeCommit = await dioxus.recv();
            if (routeCommit !== "navigated") {
                throw new Error(`unexpected route transition ack: ${routeCommit}`);
            }

            await dxRouteTransitionMutationOrFrame();
        });

        try {
            await transition.ready;
        } catch (_) {
        }

        try {
            await transition.finished;
        } catch (_) {
        }

        dioxus.send("done");
    }
} catch (_) {
    dioxus.send("fallback");
} finally {
    delete document.documentElement.dataset.routeTransition;
    delete document.documentElement.dataset.routeTransitionPlatform;
}
"#;

pub async fn animated_navigate<Route>(route: Route)
where
    Route: Clone + ToString + RouteTransitions + Routable + 'static,
{
    let current_route = router().current::<Route>().clone();
    let animation = current_route.transition_to(&route);
    let navigator = use_navigator();
    let route = route.to_string();

    if animation == NavigationAnimation::None {
        _ = navigator.push(route);
        return;
    }

    let mut transition = eval(VIEW_TRANSITION_NAVIGATE);
    _ = transition.send(animation.data_value());

    loop {
        match transition.recv::<String>().await.as_deref() {
            Ok("navigate") => {
                _ = navigator.push(route.clone());
                _ = transition.send("navigated");
            }
            Ok("done") => break,
            Ok("fallback") | Err(_) => {
                _ = navigator.push(route);
                break;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cover_transitions_dim_the_full_base_snapshot() {
        let stylesheet = include_str!("../assets/route_transitions.css");

        assert!(stylesheet.contains("::view-transition-old(base)"));
        assert!(stylesheet.contains("cover-up\"]::view-transition-new(cover)"));
        assert!(stylesheet.contains("uncover-down\"]::view-transition-old(cover)"));
        assert!(!stylesheet.contains("[data-route-transition-layer=\"sheet\"]"));
        assert!(stylesheet.contains("route-transition-dim-base"));
        assert!(stylesheet.contains("filter: brightness"));
        assert!(stylesheet.contains("cover-up\"]::view-transition-old(cover)"));
        assert!(stylesheet.contains("uncover-down\"]::view-transition-new(cover)"));
        assert!(stylesheet.contains("opacity: 0"));
    }

    #[test]
    fn mobile_cover_transitions_use_shared_cover_contract_without_platform_css_override() {
        let stylesheet = include_str!("../assets/route_transitions.css");

        assert!(stylesheet.contains("cover-up\"]::view-transition-old(base)"));
        assert!(stylesheet.contains("uncover-down\"]::view-transition-new(base)"));
        assert!(
            !stylesheet
                .contains("data-route-transition-platform=\"mobile\"]::view-transition-old(base)")
        );
        assert!(!stylesheet.contains("route-transition-mobile-dim-base"));
        assert!(!stylesheet.contains("route-transition-mobile-undim-base"));
    }

    #[test]
    fn cover_transitions_do_not_apply_debug_offsets_to_snapshots() {
        let stylesheet = include_str!("../assets/route_transitions.css");

        assert!(!stylesheet.contains("--route-transition-debug-peek"));
        assert!(!stylesheet.contains("--route-transition-cover-x"));
        assert!(!stylesheet.contains("--route-transition-base-x"));
        assert!(stylesheet.contains("transform: translateY(100vh)"));
        assert!(stylesheet.contains("transform: translateY(0vh)"));
        assert!(!stylesheet.contains("translateX(var(--route-transition-base-x))"));
    }

    #[test]
    fn cover_transitions_force_active_snapshots_to_paint_above_base() {
        let stylesheet = include_str!("../assets/route_transitions.css");

        assert!(stylesheet.contains("cover-up\"]::view-transition-new(cover),"));
        assert!(stylesheet.contains("uncover-down\"]::view-transition-old(cover)"));
        assert!(stylesheet.contains("mix-blend-mode: normal"));
        assert!(stylesheet.contains("opacity: 1"));
        assert!(stylesheet.contains("z-index: 2"));
        assert!(stylesheet.contains("cover-up\"]::view-transition-old(base),"));
        assert!(stylesheet.contains("uncover-down\"]::view-transition-new(base)"));
        assert!(stylesheet.contains("z-index: 1"));
    }
    #[test]
    fn snapshot_marker_components_are_public_contract() {
        let source = include_str!("lib.rs");
        let production_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes tests");

        assert!(production_source.contains("pub const ROUTE_TRANSITION_BASE_CLASS"));
        assert!(production_source.contains("pub const ROUTE_TRANSITION_COVER_CLASS"));
        assert!(production_source.contains("pub const ROUTE_TRANSITION_SEGMENT_CLASS"));
        assert!(production_source.contains("pub fn RouteTransitionBase"));
        assert!(production_source.contains("pub fn RouteTransitionCover"));
        assert!(production_source.contains("pub fn RouteTransitionSegment"));
        assert!(production_source.contains("merge_transition_class"));
    }
    #[test]
    fn route_transition_root_wraps_provider_and_cover_marker() {
        let source = include_str!("lib.rs");

        assert!(source.contains("pub fn RouteTransitionRoot"));
        assert!(source.contains("RouteTransitionProvider"));
        assert!(source.contains("ROUTE_TRANSITION_COVER_CLASS"));
        assert!(source.contains("merge_transition_class(ROUTE_TRANSITION_COVER_CLASS"));
        assert!(source.contains("div { class, {children} }"));
    }
    #[test]
    fn view_transition_update_waits_for_native_route_commit_without_blocking_on_raf() {
        assert!(VIEW_TRANSITION_NAVIGATE.contains("document.startViewTransition(async () =>"));
        assert!(VIEW_TRANSITION_NAVIGATE.contains("dioxus.send(\"navigate\")"));
        assert!(VIEW_TRANSITION_NAVIGATE.contains("const routeCommit = await dioxus.recv()"));
        assert!(VIEW_TRANSITION_NAVIGATE.contains("routeCommit !== \"navigated\""));
        assert!(!VIEW_TRANSITION_NAVIGATE.contains("await dxRouteTransitionNextFrame();"));
        assert!(VIEW_TRANSITION_NAVIGATE.contains("await dxRouteTransitionMutationOrFrame()"));
        assert!(VIEW_TRANSITION_NAVIGATE.contains("await transition.ready"));
        assert!(!VIEW_TRANSITION_NAVIGATE.contains("console.info"));
        assert!(!VIEW_TRANSITION_NAVIGATE.contains("g3RouteTransition"));
    }

    #[test]
    fn rust_side_acknowledges_navigation_after_router_push() {
        let source = include_str!("lib.rs");
        let production_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes tests");

        assert!(production_source.contains("Ok(\"navigate\")"));
        assert!(production_source.contains("transition.send(\"navigated\")"));
        assert!(!production_source.contains("eprintln!"));
    }

    #[test]
    fn route_transition_css_uses_production_durations() {
        let stylesheet = include_str!("../assets/route_transitions.css");

        assert!(stylesheet.contains("--route-transition-cover-duration: 0.6s"));
        assert!(stylesheet.contains("--route-transition-push-duration: 260ms"));
        assert!(!stylesheet.contains("route-transition-duration-debug"));
        assert!(!stylesheet.contains("route-transition-mobile-dim-base"));
        assert!(!stylesheet.contains("route-transition-mobile-undim-base"));
    }
    #[test]
    fn animation_data_values_match_css_contract() {
        assert_eq!(NavigationAnimation::None.data_value(), "none");
        assert_eq!(NavigationAnimation::Fade.data_value(), "fade");
        assert_eq!(NavigationAnimation::PushLeft.data_value(), "push-left");
        assert_eq!(NavigationAnimation::PushRight.data_value(), "push-right");
        assert_eq!(NavigationAnimation::CoverUp.data_value(), "cover-up");
        assert_eq!(
            NavigationAnimation::UncoverDown.data_value(),
            "uncover-down"
        );
    }
}
