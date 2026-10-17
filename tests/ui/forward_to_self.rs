use g3_route_transitions::RouteTransitions;

#[derive(PartialEq, RouteTransitions)]
enum Route {
    #[transition(forward_to = Home)]
    Home {},
}

fn main() {}
