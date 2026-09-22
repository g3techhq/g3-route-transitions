//! Browser Back and Forward, animated.
//!
//! The router learns about history traversals only through
//! `History::updater`. [`use_browser_history_transitions`] provides a history
//! that wraps the platform one and holds that update back until a View
//! Transition has captured the outgoing page, then releases it from inside the
//! transition's update callback, exactly as [`animated_navigate`] does for a
//! push.
//!
//! [`animated_navigate`]: crate::animated_navigate
use crate::{NavigationTransition, RouteTransitions};
use dioxus::prelude::*;

/// Which way a history traversal moved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) enum Traversal {
    Back,
    Forward,
    /// Neither the browser nor the recorded entries could tell.
    Unknown,
}

impl Traversal {
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn parse(value: &str) -> Self {
        match value {
            "back" => Traversal::Back,
            "forward" => Traversal::Forward,
            _ => Traversal::Unknown,
        }
    }
}

/// The routes this document has pushed, and which one is showing.
///
/// Only a fallback for browsers without the Navigation API, which reports the
/// direction itself. It starts from the entry the app was loaded on, so it
/// cannot see entries from before a reload; a traversal it cannot place resets
/// it to the destination.
#[derive(Debug)]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) struct Entries {
    routes: Vec<String>,
    cursor: usize,
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
impl Entries {
    pub(crate) fn new(route: String) -> Self {
        Self {
            routes: vec![route],
            cursor: 0,
        }
    }

    pub(crate) fn push(&mut self, route: String) {
        self.routes.truncate(self.cursor + 1);
        self.routes.push(route);
        self.cursor = self.routes.len() - 1;
    }

    pub(crate) fn replace(&mut self, route: String) {
        self.routes[self.cursor] = route;
    }

    /// Move to `route`, returning the direction taken.
    ///
    /// `reported` is what the browser said, and wins when it is known. The
    /// nearest matching entry in that direction becomes current, so a
    /// multi-step `history.go(-n)` lands on the right one.
    pub(crate) fn traverse(&mut self, route: &str, reported: Traversal) -> Traversal {
        let behind = (0..self.cursor).rev().find(|&i| self.routes[i] == route);
        let ahead = (self.cursor + 1..self.routes.len()).find(|&i| self.routes[i] == route);
        let direction = match (reported, behind, ahead) {
            (Traversal::Back | Traversal::Forward, _, _) => reported,
            (Traversal::Unknown, Some(b), Some(a)) if self.cursor - b <= a - self.cursor => {
                Traversal::Back
            }
            (Traversal::Unknown, _, Some(_)) => Traversal::Forward,
            (Traversal::Unknown, Some(_), None) => Traversal::Back,
            (Traversal::Unknown, None, None) => Traversal::Unknown,
        };
        match (direction, behind, ahead) {
            (Traversal::Back, Some(index), _) | (Traversal::Forward, _, Some(index)) => {
                self.cursor = index;
            }
            _ => *self = Self::new(route.to_string()),
        }
        direction
    }
}

/// The transition for a browser traversal from `from` to `to`.
///
/// Back uses the same rule as [`try_animated_back`](crate::try_animated_back),
/// so the browser button and an in-app Back button look alike. Forward replays
/// the push that created the entry.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) fn traversal_transition<Route: RouteTransitions>(
    from: &Route,
    to: &Route,
    traversal: Traversal,
) -> NavigationTransition {
    if from == to {
        return NavigationTransition::None;
    }
    match traversal {
        Traversal::Back => from.transition_back(),
        Traversal::Forward => from.transition_to(to),
        Traversal::Unknown => NavigationTransition::CrossFade,
    }
}

/// Key of the bridge's handle on `window`, which the transition script also
/// uses to restore scroll between the two snapshots.
#[cfg(test)]
pub(crate) const BROWSER_HISTORY_STATE_KEY: &str = "g3-route-transitions.browser-history";

/// Reports each `popstate` to Rust as `direction|ua`, where `ua` is `1` when
/// the browser already animated the traversal (for example an iOS edge swipe).
///
/// The direction comes from the Navigation API where available: the entry
/// index is tracked across pushes and compared on each `popstate`, which does
/// not depend on whether `currententrychange` fires before or after it.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) const BROWSER_HISTORY_BRIDGE: &str = r#"
const stateKey = Symbol.for("g3-route-transitions.browser-history");
window[stateKey]?.dispose?.();

const navigationApi = window.navigation;
const entryIndex = () => navigationApi?.currentEntry?.index ?? -1;
let knownIndex = entryIndex();

// Dioxus's web history stores the scroll position of each entry as `[x, y]`.
let pendingScroll = null;
const readScroll = (state) =>
    Array.isArray(state) && state.length === 2 && state.every(Number.isFinite) ? state : null;
const restoreScroll = () => {
    const scroll = pendingScroll;
    pendingScroll = null;
    if (scroll) window.scrollTo(scroll[0], scroll[1]);
};

const onEntryChange = (event) => {
    // Traversals are measured in the popstate handler instead.
    if (event.navigationType !== "traverse") knownIndex = entryIndex();
};
const onPopState = (event) => {
    pendingScroll = readScroll(event.state);
    const index = entryIndex();
    let direction = "unknown";
    if (index >= 0 && knownIndex >= 0 && index !== knownIndex) {
        direction = index < knownIndex ? "back" : "forward";
    }
    knownIndex = index;
    dioxus.send(`${direction}|${event.hasUAVisualTransition ? 1 : 0}`);
};

navigationApi?.addEventListener?.("currententrychange", onEntryChange);
window.addEventListener("popstate", onPopState);

let disposed = false;
window[stateKey] = {
    restoreScroll,
    dispose() {
        disposed = true;
        navigationApi?.removeEventListener?.("currententrychange", onEntryChange);
        window.removeEventListener("popstate", onPopState);
    },
};

while (!disposed) {
    let message;
    try {
        message = await dioxus.recv();
    } catch (_) {
        break;
    }
    if (message === "restore-scroll") restoreScroll();
}
"#;

/// Animate the browser's Back and Forward buttons (and `history.back()` from
/// other code) with the same transitions as [`try_animated_back`] and
/// [`animated_navigate`].
///
/// Call it once in the component that renders `Router::<Route>`, before the
/// router. It is opt-in and needs no other setup:
///
/// ```rust,no_run
/// # use dioxus::prelude::*;
/// # use g3_route_transitions::{RouteTransitionApp, RouteTransitions, use_browser_history_transitions};
/// # #[derive(Clone, PartialEq, Routable, RouteTransitions)]
/// # enum Route { #[route("/")] Home {} }
/// # #[component] fn Home() -> Element { VNode::empty() }
/// #[component]
/// fn App() -> Element {
///     use_browser_history_transitions::<Route>();
///     rsx! { RouteTransitionApp { Router::<Route> {} } }
/// }
/// ```
///
/// Back uses `current.transition_back()` and Forward uses
/// `current.transition_to(&next)`. The direction comes from the Navigation
/// API, or, in browsers without it, from the entries pushed since the page
/// loaded; a traversal it cannot place cross-fades.
///
/// No transition runs when the browser reports that it already animated the
/// traversal (`hasUAVisualTransition`, set for example by Safari's edge
/// swipe), so the two animations never stack. Traversals started by
/// [`try_animated_back`] are not animated a second time.
///
/// Only web builds are affected; elsewhere the hook does nothing. It wraps the
/// history the renderer provides, so a base path, hash routing, and scroll
/// restoration keep working.
///
/// A component that re-renders for an unrelated reason in the brief window
/// between the traversal and the snapshot sees the new route early, since the
/// URL has already changed by then.
///
/// [`try_animated_back`]: crate::try_animated_back
/// [`animated_navigate`]: crate::animated_navigate
pub fn use_browser_history_transitions<Route>()
where
    Route: Clone + ToString + RouteTransitions + Routable + 'static,
{
    #[cfg(target_arch = "wasm32")]
    web::use_browser_history_transitions::<Route>();
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::{BROWSER_HISTORY_BRIDGE, Entries, Traversal, traversal_transition};
    use crate::{NavigationTransition, RouteTransitions, run_animated_navigation};
    use dioxus::{
        document::eval,
        history::{History, history},
        prelude::*,
    };
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
        sync::Arc,
    };

    /// How long after our own `go_back`/`go_forward` a `popstate` is taken to
    /// be theirs. The caller is already animating those.
    const PROGRAMMATIC_WINDOW_MS: f64 = 1000.0;

    type RouterUpdate = Arc<dyn Fn() + Send + Sync>;

    struct Shared {
        inner: Rc<dyn History>,
        router_update: RefCell<Option<RouterUpdate>>,
        current: RefCell<String>,
        entries: RefCell<Entries>,
        programmatic_at: Cell<Option<f64>>,
    }

    impl Shared {
        fn record_current(&self, pushed: bool) {
            let route = self.inner.current_route();
            if *self.current.borrow() == route {
                return;
            }
            if pushed {
                self.entries.borrow_mut().push(route.clone());
            } else {
                self.entries.borrow_mut().replace(route.clone());
            }
            *self.current.borrow_mut() = route;
        }

        fn mark_programmatic(&self) {
            self.programmatic_at.set(now());
        }

        fn take_programmatic(&self) -> bool {
            match (self.programmatic_at.take(), now()) {
                (Some(at), Some(now)) => now - at <= PROGRAMMATIC_WINDOW_MS,
                (Some(_), None) => true,
                _ => false,
            }
        }
    }

    fn now() -> Option<f64> {
        Some(web_sys::window()?.performance()?.now())
    }

    /// The renderer's history, with the router's traversal update held back.
    struct TransitionHistory(Rc<Shared>);

    impl History for TransitionHistory {
        fn current_route(&self) -> String {
            self.0.inner.current_route()
        }

        fn current_prefix(&self) -> Option<String> {
            self.0.inner.current_prefix()
        }

        fn can_go_back(&self) -> bool {
            self.0.inner.can_go_back()
        }

        fn go_back(&self) {
            self.0.mark_programmatic();
            self.0.inner.go_back();
        }

        fn can_go_forward(&self) -> bool {
            self.0.inner.can_go_forward()
        }

        fn go_forward(&self) {
            self.0.mark_programmatic();
            self.0.inner.go_forward();
        }

        fn push(&self, route: String) {
            self.0.inner.push(route);
            self.0.record_current(true);
        }

        fn replace(&self, path: String) {
            self.0.inner.replace(path);
            self.0.record_current(false);
        }

        fn external(&self, url: String) -> bool {
            self.0.inner.external(url)
        }

        // The inner history's own popstate listener is never installed; the
        // bridge replaces it, scroll restoration included.
        fn updater(&self, callback: RouterUpdate) {
            *self.0.router_update.borrow_mut() = Some(callback);
        }

        fn include_prevent_default(&self) -> bool {
            self.0.inner.include_prevent_default()
        }
    }

    pub(super) fn use_browser_history_transitions<Route>()
    where
        Route: Clone + ToString + RouteTransitions + Routable + 'static,
    {
        let shared = use_hook(|| {
            let inner = history();
            let route = inner.current_route();
            let shared = Rc::new(Shared {
                inner,
                router_update: RefCell::new(None),
                current: RefCell::new(route.clone()),
                entries: RefCell::new(Entries::new(route)),
                programmatic_at: Cell::new(None),
            });
            provide_context(Rc::new(TransitionHistory(shared.clone())) as Rc<dyn History>);
            shared
        });
        use_future(move || {
            let shared = shared.clone();
            async move {
                let mut bridge = eval(BROWSER_HISTORY_BRIDGE);
                while let Ok(message) = bridge.recv::<String>().await {
                    let (direction, ua_animated) =
                        message.split_once('|').unwrap_or((&message, "0"));
                    let Some(update) = shared.router_update.borrow().clone() else {
                        continue;
                    };
                    let programmatic = shared.take_programmatic();
                    let to = shared.inner.current_route();
                    let from = shared.current.replace(to.clone());
                    let traversal = shared
                        .entries
                        .borrow_mut()
                        .traverse(&to, Traversal::parse(direction));
                    let animation = if programmatic || ua_animated == "1" {
                        NavigationTransition::None
                    } else {
                        match (Route::from_str(&from), Route::from_str(&to)) {
                            (Ok(from), Ok(to)) => traversal_transition(&from, &to, traversal),
                            _ => NavigationTransition::None,
                        }
                    };
                    if animation == NavigationTransition::None {
                        update();
                        _ = bridge.send("restore-scroll");
                        continue;
                    }
                    // Spawned so a quick second press is still heard; its
                    // transition skips this one.
                    spawn(async move {
                        run_animated_navigation(animation, &from, Some(&to), move || update())
                            .await;
                    });
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VIEW_TRANSITION_NAVIGATE;

    fn entries(routes: &[&str], cursor: usize) -> Entries {
        Entries {
            routes: routes.iter().map(|route| route.to_string()).collect(),
            cursor,
        }
    }

    #[test]
    fn pushing_after_going_back_drops_the_forward_entries() {
        let mut history = entries(&["/", "/a", "/b"], 1);
        history.push("/c".into());
        assert_eq!(history.routes, ["/", "/a", "/c"]);
        assert_eq!(history.cursor, 2);
        history.replace("/d".into());
        assert_eq!(history.routes, ["/", "/a", "/d"]);
    }

    #[test]
    fn the_reported_direction_wins_and_moves_to_the_nearest_match() {
        let mut history = entries(&["/", "/a", "/", "/a"], 3);
        assert_eq!(history.traverse("/", Traversal::Back), Traversal::Back);
        assert_eq!(history.cursor, 2);
        assert_eq!(history.traverse("/a", Traversal::Back), Traversal::Back);
        assert_eq!(history.cursor, 1);
        assert_eq!(
            history.traverse("/a", Traversal::Forward),
            Traversal::Forward
        );
        assert_eq!(history.cursor, 3);
    }

    #[test]
    fn an_unreported_direction_is_inferred_from_the_entries() {
        let mut history = entries(&["/", "/a", "/b"], 1);
        assert_eq!(
            history.traverse("/b", Traversal::Unknown),
            Traversal::Forward
        );
        assert_eq!(history.traverse("/", Traversal::Unknown), Traversal::Back);
        assert_eq!(history.cursor, 0);
        // A route on both sides resolves to the closer one.
        let mut history = entries(&["/x", "/", "/a", "/x", "/x"], 2);
        assert_eq!(
            history.traverse("/x", Traversal::Unknown),
            Traversal::Forward
        );
        assert_eq!(history.cursor, 3);
    }

    #[test]
    fn an_entry_that_cannot_be_placed_resets_the_record() {
        let mut history = entries(&["/", "/a"], 1);
        assert_eq!(
            history.traverse("/old", Traversal::Unknown),
            Traversal::Unknown
        );
        assert_eq!(history.routes, ["/old"]);
        // Reported, but from before the page loaded.
        let mut history = entries(&["/"], 0);
        assert_eq!(history.traverse("/old", Traversal::Back), Traversal::Back);
        assert_eq!(
            (history.routes.as_slice(), history.cursor),
            (&["/old".to_string()][..], 0)
        );
    }

    #[derive(PartialEq)]
    enum Page {
        Root,
        Child,
    }

    impl RouteTransitions for Page {
        fn transition_to(&self, next: &Self) -> NavigationTransition {
            match next {
                Page::Child => NavigationTransition::Forward,
                Page::Root => NavigationTransition::Backward,
            }
        }

        fn transition_back(&self) -> NavigationTransition {
            NavigationTransition::DismissSheet
        }
    }

    #[test]
    fn back_matches_in_app_back_and_forward_replays_the_push() {
        use NavigationTransition as T;
        assert_eq!(
            traversal_transition(&Page::Child, &Page::Root, Traversal::Back),
            T::DismissSheet
        );
        assert_eq!(
            traversal_transition(&Page::Root, &Page::Child, Traversal::Forward),
            T::Forward
        );
        assert_eq!(
            traversal_transition(&Page::Root, &Page::Child, Traversal::Unknown),
            T::CrossFade
        );
        assert_eq!(
            traversal_transition(&Page::Root, &Page::Root, Traversal::Back),
            T::None
        );
    }

    #[test]
    fn the_bridge_reports_direction_and_browser_animation() {
        assert_eq!(Traversal::parse("back"), Traversal::Back);
        assert_eq!(Traversal::parse("forward"), Traversal::Forward);
        assert_eq!(Traversal::parse("unknown"), Traversal::Unknown);
        let bridge = BROWSER_HISTORY_BRIDGE;
        assert!(bridge.contains("window.addEventListener(\"popstate\", onPopState)"));
        assert!(bridge.contains("event.hasUAVisualTransition"));
        assert!(bridge.contains("navigationApi?.currentEntry?.index"));
        assert!(bridge.contains("dioxus.send(`${direction}|"));
        assert!(bridge.contains("window[stateKey]?.dispose?.()"));
        // Traversals are measured only when popstate arrives, whatever order
        // the browser fires the two events in.
        assert!(bridge.contains("event.navigationType !== \"traverse\""));
    }

    #[test]
    fn the_transition_restores_scroll_before_the_new_snapshot() {
        let key = format!("Symbol.for(\"{BROWSER_HISTORY_STATE_KEY}\")");
        assert!(BROWSER_HISTORY_BRIDGE.contains(&key));
        assert!(VIEW_TRANSITION_NAVIGATE.contains(&key));
        let rendered = VIEW_TRANSITION_NAVIGATE.find("await rendered;").unwrap();
        let restored = VIEW_TRANSITION_NAVIGATE.find("restoreScroll?.()").unwrap();
        let captured = VIEW_TRANSITION_NAVIGATE
            .find("dxRouteTransitionIncomingPaintContext();\n        });")
            .unwrap();
        assert!(rendered < restored && restored < captured);
    }

    /// The router must hear about traversals only through the transition, and
    /// the renderer's own popstate listener must never be installed.
    #[test]
    fn the_router_update_is_held_back() {
        let source = include_str!("browser_history.rs");
        let web = source.split("mod web {").nth(1).unwrap();
        let web = web.split("#[cfg(test)]").next().unwrap();
        assert!(!web.contains("inner.updater("));
        assert!(web.contains("*self.0.router_update.borrow_mut() = Some(callback)"));
        assert!(
            web.contains("run_animated_navigation(animation, &from, Some(&to), move || update())")
        );
        assert!(web.contains("provide_context(Rc::new(TransitionHistory("));
        // Our own go_back is already animated by its caller.
        assert!(web.contains("self.0.mark_programmatic();\n            self.0.inner.go_back();"));
    }
}
