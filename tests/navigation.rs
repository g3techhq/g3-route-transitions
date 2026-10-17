//! The navigation functions, run beneath a real router.
//!
//! There is no renderer here, so `eval` is unsupported and every animated
//! navigation takes the script's failure path. That is what this checks: the
//! route still changes, and history is pushed, replaced or popped by the rules.
// Clippy asks for `const` thread-local initializers that already are, when
// linting for Android.
#![allow(clippy::missing_const_for_thread_local)]
use dioxus::{
    core::NoOpMutations,
    history::{History, MemoryHistory},
    prelude::*,
};
use g3_route_transitions::{
    NavigationTransition, RouteTransitions, animated_back_or_navigate, animated_navigate,
    animated_navigate_with, try_animated_back,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

#[derive(Clone, Debug, PartialEq, Routable, RouteTransitions)]
#[rustfmt::skip]
enum Route {
    #[layout(Harness)]
        #[transition(layer = stack_root, history = replace)]
        #[route("/home/:filter")]
        Home { filter: u8 },
        #[transition(layer = stack_page)]
        #[route("/details/:id")]
        Details { id: u8 },
        #[transition(layer = sheet, handoff_from = Details)]
        #[route("/compose")]
        Compose {},
}

#[component]
fn Home(filter: u8) -> Element {
    rsx! { "home {filter}" }
}

#[component]
fn Details(id: u8) -> Element {
    rsx! { "details {id}" }
}

#[component]
fn Compose() -> Element {
    rsx! { "compose" }
}

thread_local! {
    static STEPS: RefCell<Vec<(String, bool)>> = const { RefCell::new(Vec::new()) };
    static BACK_RESULTS: RefCell<Vec<bool>> = const { RefCell::new(Vec::new()) };
    static DONE: Cell<bool> = const { Cell::new(false) };
}

/// Where the router is and whether it can go back.
fn record() {
    let route = router().current::<Route>().to_string();
    let can_go_back = navigator().can_go_back();
    STEPS.with(|steps| steps.borrow_mut().push((route, can_go_back)));
}

/// A layout that stays mounted across every route, so its task survives the
/// navigations it starts.
#[component]
fn Harness() -> Element {
    use_hook(|| {
        spawn(async {
            record();
            // Rule 1: the current route is a no-op.
            animated_navigate(Route::Home { filter: 0 }).await;
            record();
            // Rule 10: same variant with `history = replace`.
            animated_navigate(Route::Home { filter: 1 }).await;
            record();
            // Rule 7: animated, and pushed even though the script failed.
            animated_navigate(Route::Details { id: 1 }).await;
            record();
            // Rule 4: a hand-off replaces the entry it leaves.
            animated_navigate(Route::Compose {}).await;
            record();
            let popped = try_animated_back::<Route>().await;
            BACK_RESULTS.with(|results| results.borrow_mut().push(popped));
            record();
            let popped = try_animated_back::<Route>().await;
            BACK_RESULTS.with(|results| results.borrow_mut().push(popped));
            record();
            // Nothing to pop, so the fallback is navigated to.
            animated_back_or_navigate(Route::Details { id: 2 }).await;
            record();
            animated_navigate_with(Route::Details { id: 3 }, NavigationTransition::CrossFade).await;
            record();
            // An explicit `None` still writes history by the route rules.
            animated_navigate_with(Route::Home { filter: 1 }, NavigationTransition::None).await;
            record();
            DONE.with(|done| done.set(true));
        })
    });
    rsx! { Outlet::<Route> {} }
}

#[component]
fn App() -> Element {
    use_context_provider(|| {
        Rc::new(MemoryHistory::with_initial_path("/home/0")) as Rc<dyn History>
    });
    rsx! { Router::<Route> {} }
}

#[tokio::test]
async fn navigation_writes_history_by_the_route_rules() {
    let mut dom = VirtualDom::new(App);
    dom.rebuild_in_place();
    tokio::time::timeout(Duration::from_secs(10), async {
        while !DONE.with(Cell::get) {
            dom.wait_for_work().await;
            dom.render_immediate(&mut NoOpMutations);
        }
    })
    .await
    .expect("the navigation script finishes");

    let steps = STEPS.with(|steps| steps.borrow().clone());
    let expected = [
        ("/home/0", false),
        ("/home/0", false),
        ("/home/1", false),
        ("/details/1", true),
        ("/compose", true),
        ("/home/1", false),
        ("/home/1", false),
        ("/details/2", true),
        ("/details/3", true),
        ("/home/1", true),
    ];
    let expected = expected
        .iter()
        .map(|(route, back)| (route.to_string(), *back))
        .collect::<Vec<_>>();
    assert_eq!(steps, expected);
    assert_eq!(
        BACK_RESULTS.with(|results| results.borrow().clone()),
        [true, false]
    );
}

/// The script's failure path is the one exercised above; the rules it relied
/// on are these.
#[test]
fn the_routes_used_above_follow_the_documented_rules() {
    let home = Route::Home { filter: 1 };
    let details = Route::Details { id: 1 };
    assert_eq!(home.transition_to(&details), NavigationTransition::Forward);
    assert!(!home.replaces_history(&details));
    assert!(details.replaces_history(&Route::Compose {}));
    assert!(home.replaces_history(&Route::Home { filter: 2 }));
}
