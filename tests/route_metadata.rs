use g3_route_transitions::{NavigationAnimation, RouteTransitions, route_transitions};
#[route_transitions]
#[derive(Clone, PartialEq)]
enum Route {
    #[transition(root, replace)]
    Home { filter: u8 },
    #[transition(root)]
    Profile {},
    #[transition(cover)]
    Editor {},
    #[transition(pushed, forward = (Comments, Person))]
    Detail { id: u8 },
    #[transition(pushed)]
    Comments { id: u8 },
    #[transition(pushed, replace(key = id))]
    Ratings { id: u8, filter: u8 },
    #[transition(pushed, forward = Detail)]
    Person { id: u8 },
}
#[test]
fn roots_push_full_screen_pages_and_fade_between_peers() {
    let home = Route::Home { filter: 0 };
    let detail = Route::Detail { id: 7 };
    assert_eq!(home.transition_to(&detail), NavigationAnimation::PushLeft);
    assert_eq!(detail.transition_to(&home), NavigationAnimation::PushRight);
    assert_eq!(
        home.transition_to(&Route::Profile {}),
        NavigationAnimation::Fade
    );
}
#[test]
fn directed_drill_down_routes_push_and_reverse() {
    let detail = Route::Detail { id: 7 };
    let comments = Route::Comments { id: 7 };
    assert_eq!(
        detail.transition_to(&comments),
        NavigationAnimation::PushLeft
    );
    assert_eq!(
        comments.transition_to(&detail),
        NavigationAnimation::PushRight
    );
}
#[test]
fn in_place_variants_replace_history_with_optional_identity_keys() {
    let home = Route::Home { filter: 0 };
    let filtered_home = Route::Home { filter: 1 };
    assert_eq!(
        home.transition_to(&filtered_home),
        NavigationAnimation::None
    );
    assert!(home.replaces_history(&filtered_home));
    let ratings = Route::Ratings { id: 7, filter: 0 };
    let filtered_ratings = Route::Ratings { id: 7, filter: 1 };
    let other_person = Route::Ratings { id: 8, filter: 1 };
    assert!(ratings.replaces_history(&filtered_ratings));
    assert!(!ratings.replaces_history(&other_person));
}
#[test]
fn browser_back_uses_current_route_layer_semantics() {
    let pushed = Route::Person { id: 7 };
    let cover = Route::Editor {};
    assert_eq!(
        pushed.transition_to(&Route::Detail { id: 7 }),
        NavigationAnimation::PushLeft,
    );
    assert_eq!(pushed.transition_back(), NavigationAnimation::PushRight);
    assert_eq!(cover.transition_back(), NavigationAnimation::UncoverDown);
}
