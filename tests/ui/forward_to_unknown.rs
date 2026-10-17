use g3_route_transitions::RouteTransitions;

#[derive(PartialEq, RouteTransitions)]
enum Route {
    #[transition(forward_to = Missing)]
    Home {},
}

fn main() {}
