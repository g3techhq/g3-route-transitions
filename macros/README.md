# g3-route-transitions-macros

Proc-macro crate for
[`g3-route-transitions`](https://github.com/g3techhq/g3-route-transitions).

Depend on `g3-route-transitions` instead of this crate. It re-exports the
derive next to the trait it implements:

```rust,ignore
use g3_route_transitions::RouteTransitions;

#[derive(Clone, PartialEq, Routable, RouteTransitions)]
enum Route {
    #[transition(layer = stack_root)]
    #[route("/")]
    Home {},
}
```

The derive implements `g3_route_transitions::RouteTransitions` and registers
the per-variant `#[transition(...)]` helper attribute: route layers (`base`,
`stack_root`, `stack_page`, `sheet`), `forward_to`, `peers(...)`,
`history = replace`, and `handoff_from`.

The full option reference and the rule table that decides which transition
runs are documented on the derive itself, so rust-analyzer shows them when
you hover `RouteTransitions` in a derive list. They are also published on
[docs.rs](https://docs.rs/g3-route-transitions/latest/g3_route_transitions/derive.RouteTransitions.html).
