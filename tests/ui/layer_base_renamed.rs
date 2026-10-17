use g3_route_transitions::RouteTransitions;

#[derive(PartialEq, RouteTransitions)]
enum Route {
    #[transition(layer = base)]
    Home {},
}

fn main() {}
