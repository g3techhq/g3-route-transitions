use g3_route_transitions::{NavigationTransition, RouteTransitions};
#[derive(Clone, PartialEq, RouteTransitions)]
enum Route {
    #[transition(layer = stack_root, history = replace)]
    Home { filter: u8 },
    #[transition(layer = stack_root)]
    Profile {},
    #[transition(layer = sheet)]
    Editor {},
    #[transition(layer = stack_page, forward_to = (Comments, Person))]
    Detail { id: u8 },
    #[transition(layer = stack_page)]
    Comments { id: u8 },
    #[transition(layer = stack_page, history = replace(key = id))]
    Ratings { id: u8, filter: u8 },
    #[transition(layer = stack_page, forward_to = Detail)]
    Person { id: u8 },
    #[transition(
        layer = sheet,
        history = replace(key = id),
        peers(group = tabs, key = id, order = tab),
    )]
    Tabs { id: u8, tab: u8 },
    #[transition(layer = sheet)]
    Inbox {},
    #[transition(layer = sheet, handoff_from = Inbox)]
    Message { id: u8 },
}
/// A sheet that hands off to another sheet leaves no entry behind: Back from the
/// second returns to the page under both. The hand-off still rises like any
/// sheet, instead of taking the cross-fade between unrelated sheets.
#[test]
fn a_sheet_handing_off_to_another_sheet_replaces_it_and_still_rises() {
    let inbox = Route::Inbox {};
    let message = Route::Message { id: 3 };
    assert_eq!(
        inbox.transition_to(&message),
        NavigationTransition::PresentSheet
    );
    assert!(inbox.replaces_history(&message));
    assert_eq!(
        message.transition_back(),
        NavigationTransition::DismissSheet
    );

    // Directed: going the other way is ordinary navigation between sheets.
    assert_eq!(
        message.transition_to(&inbox),
        NavigationTransition::CrossFade
    );
    assert!(!message.replaces_history(&inbox));

    // Only a listed route hands off.
    assert_eq!(
        Route::Editor {}.transition_to(&message),
        NavigationTransition::CrossFade
    );
    assert!(!Route::Editor {}.replaces_history(&message));
    let home = Route::Home { filter: 0 };
    assert_eq!(
        home.transition_to(&message),
        NavigationTransition::PresentSheet
    );
    assert!(!home.replaces_history(&message));
}
#[test]
fn roots_push_full_screen_pages_and_fade_between_peers() {
    let home = Route::Home { filter: 0 };
    let detail = Route::Detail { id: 7 };
    assert_eq!(home.transition_to(&detail), NavigationTransition::Forward);
    assert_eq!(detail.transition_to(&home), NavigationTransition::Backward);
    assert_eq!(
        home.transition_to(&Route::Profile {}),
        NavigationTransition::CrossFade
    );
}
#[test]
fn directed_drill_down_routes_push_and_reverse() {
    let detail = Route::Detail { id: 7 };
    let comments = Route::Comments { id: 7 };
    assert_eq!(
        detail.transition_to(&comments),
        NavigationTransition::Forward
    );
    assert_eq!(
        comments.transition_to(&detail),
        NavigationTransition::Backward
    );
}
#[test]
fn in_place_variants_replace_history_with_optional_identity_keys() {
    let home = Route::Home { filter: 0 };
    let filtered_home = Route::Home { filter: 1 };
    assert_eq!(
        home.transition_to(&filtered_home),
        NavigationTransition::None
    );
    assert!(home.replaces_history(&filtered_home));
    let ratings = Route::Ratings { id: 7, filter: 0 };
    let filtered_ratings = Route::Ratings { id: 7, filter: 1 };
    let other_person = Route::Ratings { id: 8, filter: 1 };
    assert!(ratings.replaces_history(&filtered_ratings));
    assert!(!ratings.replaces_history(&other_person));
}

/// A segmented control wants both halves: the body slides toward the tab the
/// user picked, and Back leaves the screen instead of retracing every tab they
/// touched. History replacement decides how history is written; when the same
/// variant also declares `peers`, the ordering still selects the animation.
#[test]
fn segmented_peers_slide_while_still_replacing_history() {
    let first = Route::Tabs { id: 7, tab: 0 };
    let second = Route::Tabs { id: 7, tab: 1 };

    assert_eq!(first.transition_to(&second), NavigationTransition::Forward);
    assert_eq!(second.transition_to(&first), NavigationTransition::Backward);
    assert!(first.replaces_history(&second));
    assert!(second.replaces_history(&first));

    // The peer group is keyed, so tabs of a different record are not peers and
    // neither slide nor replace.
    let other_record = Route::Tabs { id: 8, tab: 1 };
    assert_eq!(
        first.transition_to(&other_record),
        NavigationTransition::CrossFade
    );
    assert!(!first.replaces_history(&other_record));
}
#[test]
fn browser_back_uses_current_route_layer_semantics() {
    let pushed = Route::Person { id: 7 };
    let sheet = Route::Editor {};
    assert_eq!(
        pushed.transition_to(&Route::Detail { id: 7 }),
        NavigationTransition::Forward,
    );
    assert_eq!(pushed.transition_back(), NavigationTransition::Backward);
    assert_eq!(sheet.transition_back(), NavigationTransition::DismissSheet);
}

mod dioxus_router_integration {
    use dioxus::prelude::*;
    use g3_route_transitions::{NavigationTransition, RouteTransitions};

    #[derive(Clone, PartialEq, Routable, RouteTransitions)]
    enum AppRoute {
        #[route("/")]
        #[transition(layer = stack_root)]
        Home {},

        #[route("/details")]
        #[transition(layer = stack_page)]
        Details {},
    }

    #[component]
    fn Home() -> Element {
        rsx! { "Home" }
    }

    #[component]
    fn Details() -> Element {
        rsx! { "Details" }
    }

    #[test]
    fn derive_coexists_with_dioxus_routable_and_its_route_helper() {
        assert_eq!(
            AppRoute::Home {}.transition_to(&AppRoute::Details {}),
            NavigationTransition::Forward,
        );
    }
}

/// Pins the derive's documented rule table and Back table.
mod documented_rules {
    use g3_route_transitions::{NavigationTransition as T, RouteTransitions};

    #[derive(Clone, PartialEq, RouteTransitions)]
    enum Route {
        Plain {},
        Other {},
        #[transition(forward_to = Other)]
        Linked {},
        #[transition(layer = stack_root)]
        RootA {},
        #[transition(layer = stack_root)]
        RootB {},
        #[transition(layer = stack_page, forward_to = Mutual)]
        PageA {},
        #[transition(layer = stack_page)]
        PageB {},
        #[transition(layer = stack_page, forward_to = PageA)]
        Mutual {},
        #[transition(layer = sheet)]
        SheetA {},
        #[transition(layer = sheet)]
        SheetB {},
        #[transition(layer = stack_page, handoff_from = SheetA)]
        HandedPage {},
        #[transition(handoff_from = SheetA)]
        HandedBase {},
        #[transition(peers(group = tabs, order = tab))]
        TabA {
            tab: u8,
        },
        #[transition(peers(group = tabs, order = tab))]
        TabB {
            tab: u8,
        },
    }

    #[test]
    fn layer_pairs_without_a_rule_cross_fade() {
        for (from, to) in [
            (Route::Plain {}, Route::Other {}),
            (Route::Plain {}, Route::RootA {}),
            (Route::Plain {}, Route::PageA {}),
            (Route::PageA {}, Route::Plain {}),
            (Route::RootA {}, Route::RootB {}),
            (Route::PageA {}, Route::PageB {}),
            (Route::SheetA {}, Route::SheetB {}),
        ] {
            assert_eq!(from.transition_to(&to), T::CrossFade);
            assert!(!from.replaces_history(&to));
        }
    }

    #[test]
    fn sheets_present_and_dismiss_from_any_other_layer() {
        for page in [Route::Plain {}, Route::RootA {}, Route::PageA {}] {
            assert_eq!(page.transition_to(&Route::SheetA {}), T::PresentSheet);
            assert_eq!(Route::SheetA {}.transition_to(&page), T::DismissSheet);
        }
    }

    #[test]
    fn handoff_motion_follows_the_destination_layer_and_replaces() {
        let sheet = Route::SheetA {};
        assert_eq!(sheet.transition_to(&Route::HandedPage {}), T::Forward);
        assert_eq!(sheet.transition_to(&Route::HandedBase {}), T::CrossFade);
        assert!(sheet.replaces_history(&Route::HandedPage {}));
        assert!(sheet.replaces_history(&Route::HandedBase {}));
    }

    #[test]
    fn peers_compare_order_across_variants() {
        let a = Route::TabA { tab: 1 };
        assert_eq!(a.transition_to(&Route::TabB { tab: 2 }), T::Forward);
        assert_eq!(a.transition_to(&Route::TabB { tab: 0 }), T::Backward);
        assert_eq!(a.transition_to(&Route::TabB { tab: 1 }), T::None);
        assert!(!a.replaces_history(&Route::TabB { tab: 2 }));
    }

    #[test]
    fn forward_to_is_directional_and_mutual_links_are_forward_both_ways() {
        assert_eq!(Route::Linked {}.transition_to(&Route::Other {}), T::Forward);
        assert_eq!(
            Route::Other {}.transition_to(&Route::Linked {}),
            T::Backward
        );
        // `Mutual` and `PageA` list each other, so the first rule wins both ways.
        assert_eq!(Route::Mutual {}.transition_to(&Route::PageA {}), T::Forward);
        assert_eq!(Route::PageA {}.transition_to(&Route::Mutual {}), T::Forward);
    }

    /// Back looks only at the layer being left, so it does not always mirror
    /// the forward transition.
    #[test]
    fn back_depends_only_on_the_layer_being_left() {
        assert_eq!(Route::SheetA {}.transition_back(), T::DismissSheet);
        assert_eq!(Route::PageB {}.transition_back(), T::Backward);
        assert_eq!(Route::RootA {}.transition_back(), T::CrossFade);
        // Forward from `Linked` slides, but Back from the base-layer `Other`
        // cross-fades.
        assert_eq!(Route::Other {}.transition_back(), T::CrossFade);
        // `PageA` → `PageB` cross-fades, but Back from `PageB` slides.
        assert_eq!(Route::PageB {}.transition_back(), T::Backward);
    }
}
