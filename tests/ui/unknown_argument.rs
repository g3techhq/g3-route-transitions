use g3_route_transitions::RouteTransitions;

#[derive(PartialEq, RouteTransitions)]
enum Route {
    #[transition(motion = slide)]
    Home {},
}

fn main() {}
