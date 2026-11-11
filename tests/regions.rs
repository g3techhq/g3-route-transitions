//! Page and base regions publish the route they render, which the navigate
//! script compares with the destination to tell when it has landed.
use dioxus::{
    core::NoOpMutations,
    history::{History, MemoryHistory},
    prelude::*,
};
use g3_route_transitions::{RouteTransitionBaseRegion, RouteTransitionPageRegion};
use std::{rc::Rc, time::Duration};

#[derive(Clone, Debug, PartialEq, Routable)]
#[rustfmt::skip]
enum Route {
    #[layout(Shell)]
        #[route("/home")]
        Home {},
        #[route("/details/:id")]
        Details { id: u8 },
}

#[component]
fn Shell() -> Element {
    rsx! {
        RouteTransitionPageRegion {
            RouteTransitionBaseRegion { Outlet::<Route> {} }
        }
    }
}

/// Opens a details page once the first render is done.
#[component]
fn Home() -> Element {
    use_hook(|| {
        spawn(async {
            navigator().push(Route::Details { id: 7 });
        })
    });

    rsx! { "home" }
}

#[component]
fn Details(id: u8) -> Element {
    rsx! { "details {id}" }
}

#[component]
fn App() -> Element {
    use_context_provider(|| Rc::new(MemoryHistory::with_initial_path("/home")) as Rc<dyn History>);

    rsx! { Router::<Route> {} }
}

#[tokio::test]
async fn regions_publish_the_current_route_and_follow_it() {
    let mut dom = VirtualDom::new(App);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert_eq!(
        html.matches(r#"data-route-transition-route="/home""#)
            .count(),
        2,
        "{html}"
    );

    // The layout's regions stay mounted and follow the route.
    tokio::time::timeout(Duration::from_secs(5), async {
        while !dioxus_ssr::render(&dom).contains("details 7") {
            dom.wait_for_work().await;
            dom.render_immediate(&mut NoOpMutations);
        }
    })
    .await
    .expect("the route changes");
    let html = dioxus_ssr::render(&dom);
    assert_eq!(
        html.matches(r#"data-route-transition-route="/details/7""#)
            .count(),
        2,
        "{html}"
    );
}

#[test]
fn regions_outside_a_router_publish_nothing() {
    fn page() -> Element {
        rsx! {
            RouteTransitionPageRegion { "page" }
            RouteTransitionBaseRegion { "base" }
        }
    }
    let mut dom = VirtualDom::new(page);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("route-transition-page-region"), "{html}");
    assert!(!html.contains("data-route-transition-route"), "{html}");
}
