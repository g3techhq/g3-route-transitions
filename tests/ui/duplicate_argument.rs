use g3_route_transitions::RouteTransitions;

#[derive(PartialEq, RouteTransitions)]
enum Route {
    #[transition(layer = sheet, layer = stack_page)]
    Home {},
}

fn main() {}
