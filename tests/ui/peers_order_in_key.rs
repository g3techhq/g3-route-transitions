use g3_route_transitions::RouteTransitions;

#[derive(PartialEq, RouteTransitions)]
enum Route {
    #[transition(peers(group = tabs, key = (id, tab), order = tab))]
    Home { id: u8, tab: u8 },
}

fn main() {}
