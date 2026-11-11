//! The mouse's Back and Forward side buttons, animated, for desktop apps.
//!
//! A browser turns those buttons into history traversals itself, which
//! [`use_browser_history_transitions`](crate::use_browser_history_transitions)
//! animates. A desktop web view keeps the router's history in memory, so the
//! buttons do nothing there until [`use_mouse_history_buttons`] connects them.
use crate::RouteTransitions;
use dioxus::prelude::*;

/// Routes left by going back, nearest first, so Forward knows where it is
/// going and can animate the way a push there would.
///
/// The router reveals only whether it can go forward, not to what. Any new
/// navigation discards the forward history, and the stack with it.
#[derive(Debug)]
pub(crate) struct ForwardStack<R> {
    routes: Vec<R>,
    /// A forward step is under way, so the next change is not a step back.
    forwarding: bool,
}

impl<R: Clone + PartialEq> ForwardStack<R> {
    pub(crate) fn new() -> Self {
        Self {
            routes: Vec::new(),
            forwarding: false,
        }
    }

    /// Records a route change: from `left` to the current route, after which
    /// the router `can_go_forward` or not.
    pub(crate) fn changed(&mut self, left: R, can_go_forward: bool) {
        if std::mem::take(&mut self.forwarding) {
            // Our own forward step: the route it took is already off the stack.
        } else if can_go_forward {
            // A step back: the page left is now ahead.
            self.routes.push(left);
        } else {
            // A push: nothing is ahead any more.
            self.routes.clear();
        }
    }

    /// Starts a forward step and returns the route it lands on, when known.
    #[cfg_attr(
        any(target_arch = "wasm32", target_os = "android", target_os = "ios"),
        allow(dead_code, reason = "Only desktop web views listen.")
    )]
    pub(crate) fn forward(&mut self) -> Option<R> {
        self.forwarding = true;
        self.routes.pop()
    }

    /// The forward step did not happen after all.
    #[cfg_attr(
        any(target_arch = "wasm32", target_os = "android", target_os = "ios"),
        allow(dead_code, reason = "Only desktop web views listen.")
    )]
    pub(crate) fn forward_cancelled(&mut self) {
        self.forwarding = false;
    }

    #[cfg(test)]
    pub(crate) fn peek(&self) -> Option<&R> {
        self.routes.last()
    }
}

/// Animate the mouse's Back and Forward side buttons on desktop.
///
/// Back pops history with [`try_animated_back`](crate::try_animated_back), so
/// it slides or fades exactly as a Back button would. Forward returns to the
/// page Back left, animated as navigating there would be; when that page is
/// unknown (history from before this hook mounted) it cross-fades.
///
/// Call it once from a layout beneath `Router<Route>`. It does nothing on the
/// web, where the browser already handles the buttons, and on Android and
/// iOS, which have no such buttons.
pub fn use_mouse_history_buttons<Route>()
where
    Route: Routable + RouteTransitions + PartialEq,
{
    use std::{cell::RefCell, rc::Rc};
    let route: Route = use_route();
    let forward = use_hook(|| Rc::new(RefCell::new(ForwardStack::<Route>::new())));
    // Plain cells, not signals: they are bookkeeping, and a signal must not
    // be written while rendering.
    let last = use_hook(|| Rc::new(RefCell::new(route.clone())));
    // Learn each route change as it renders: whether it was a step back
    // (something ahead) or a push.
    if *last.borrow() != route {
        let left = last.replace(route.clone());
        forward
            .borrow_mut()
            .changed(left, navigator().can_go_forward());
    }

    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    {
        let stack = forward.clone();
        use_hook(move || {
            spawn(async move {
                let mut buttons = document::eval(LISTEN_SCRIPT);
                while let Ok(button) = buttons.recv::<String>().await {
                    match button.as_str() {
                        "back" => {
                            crate::try_animated_back::<Route>().await;
                        }
                        "forward" => {
                            if !navigator().can_go_forward() {
                                continue;
                            }
                            let next = stack.borrow_mut().forward();
                            if !animated_forward::<Route>(next).await {
                                stack.borrow_mut().forward_cancelled();
                            }
                        }
                        _ => {}
                    }
                }
            });
        });
        use_drop(|| {
            document::eval(DISPOSE_SCRIPT);
        });
    }
    #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
    {
        let _ = &forward;
    }
}

/// Goes forward one entry, animated as a navigation to `next` would be, or
/// cross-faded when `next` is unknown. False when there was nowhere to go.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
async fn animated_forward<Route>(next: Option<Route>) -> bool
where
    Route: Routable + RouteTransitions,
{
    crate::pending::route_changes_landed().await;
    let navigator = navigator();
    if !navigator.can_go_forward() {
        return false;
    }
    let current = router().current::<Route>();
    let animation = match &next {
        Some(next) => current.transition_to(next),
        None => crate::NavigationTransition::CrossFade,
    };
    if animation == crate::NavigationTransition::Instant {
        navigator.go_forward();
        return true;
    }
    let from = current.to_string();
    let to = next.map(|next| next.to_string());
    crate::run_animated_navigation(animation, &from, to.as_deref(), || navigator.go_forward())
        .await;
    true
}

/// Reports side-button presses. The press is cancelled too, so the web view
/// does not also act on it.
#[cfg_attr(
    any(target_arch = "wasm32", target_os = "android", target_os = "ios"),
    allow(dead_code, reason = "Only desktop web views listen.")
)]
const LISTEN_SCRIPT: &str = r#"
const key = Symbol.for("g3-route-transitions.mouse-history");
window[key]?.dispose?.();
const side = (e) => e.button === 3 || e.button === 4;
const down = (e) => { if (side(e)) e.preventDefault(); };
const up = (e) => {
    if (!side(e)) return;
    e.preventDefault();
    dioxus.send(e.button === 3 ? "back" : "forward");
};
window.addEventListener("mousedown", down, true);
window.addEventListener("mouseup", up, true);
window[key] = {
    dispose() {
        window.removeEventListener("mousedown", down, true);
        window.removeEventListener("mouseup", up, true);
    },
};
"#;

#[cfg_attr(
    any(target_arch = "wasm32", target_os = "android", target_os = "ios"),
    allow(dead_code, reason = "Only desktop web views listen.")
)]
const DISPOSE_SCRIPT: &str =
    r#"window[Symbol.for("g3-route-transitions.mouse-history")]?.dispose?.();"#;

#[cfg(test)]
mod tests {
    use super::ForwardStack;

    #[test]
    fn going_back_remembers_the_page_left_and_forward_returns_to_it() {
        let mut stack = ForwardStack::new();
        // a -> b -> c, then back to b and back to a.
        stack.changed("a", false);
        stack.changed("b", false);
        stack.changed("c", true);
        stack.changed("b", true);
        assert_eq!(stack.peek(), Some(&"b"));
        assert_eq!(stack.forward(), Some("b"));
        assert_eq!(stack.forward(), Some("c"));
        assert_eq!(stack.forward(), None);
    }

    #[test]
    fn a_new_navigation_drops_what_was_ahead() {
        let mut stack = ForwardStack::new();
        stack.changed("b", true);
        assert_eq!(stack.peek(), Some(&"b"));
        stack.changed("a", false);
        assert_eq!(stack.forward(), None);
    }

    #[test]
    fn a_forward_step_is_not_mistaken_for_a_step_back() {
        let mut stack = ForwardStack::new();
        // Back from c to b, then back from b to a: [c, b] ahead.
        stack.changed("c", true);
        stack.changed("b", true);
        // Forward from a to b: c is still ahead, a is behind.
        assert_eq!(stack.forward(), Some("b"));
        stack.changed("a", true);
        assert_eq!(stack.peek(), Some(&"c"));
        // A later step back records again.
        stack.changed("b", true);
        assert_eq!(stack.peek(), Some(&"b"));
    }

    #[test]
    fn a_cancelled_forward_leaves_the_next_change_alone() {
        let mut stack = ForwardStack::new();
        stack.changed("b", true);
        assert_eq!(stack.forward(), Some("b"));
        stack.forward_cancelled();
        stack.changed("x", true);
        assert_eq!(stack.peek(), Some(&"x"));
    }
}
