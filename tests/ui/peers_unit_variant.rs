use g3_route_transitions::RouteTransitions;

#[derive(PartialEq, RouteTransitions)]
enum Route {
    #[transition(peers(group = tabs, order = tab))]
    Home,
}

fn main() {}
