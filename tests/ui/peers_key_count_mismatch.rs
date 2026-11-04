use g3_route_transitions::RouteTransitions;

#[derive(PartialEq, RouteTransitions)]
enum Route {
    #[transition(peers(group = tabs, key = id, order = tab))]
    Details { id: u32, tab: u8 },
    #[transition(peers(group = tabs, order = tab))]
    Overview { tab: u8 },
}

fn main() {}
