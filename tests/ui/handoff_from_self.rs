use g3_route_transitions::RouteTransitions;

#[derive(PartialEq, RouteTransitions)]
enum Route {
    #[transition(handoff_from = Home)]
    Home {},
}

fn main() {}
