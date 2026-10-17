use g3_route_transitions::RouteTransitions;

#[derive(PartialEq, RouteTransitions)]
enum Route {
    Home {},
    Detail(u8),
}

fn main() {}
