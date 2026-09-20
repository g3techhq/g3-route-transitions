//! Proc-macro support for [`g3-route-transitions`](https://docs.rs/g3-route-transitions).
//!
//! This crate is an implementation detail. Import the documented
//! `g3_route_transitions::RouteTransitions` derive instead of depending on this
//! crate directly.
#![warn(missing_docs)]
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use std::collections::{BTreeMap, BTreeSet};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Error, Fields, Ident, ItemEnum, Result, Token, parenthesized, parse_macro_input};
/// Derives route-owned View Transition behavior for a Dioxus `Routable` enum.
///
/// Add `RouteTransitions` to the same derive list as `Routable`, then add
/// `#[transition(...)]` to variants that need more than the default. Hover
/// `RouteTransitions` in an editor backed by rust-analyzer to see this
/// reference without leaving the route declaration.
///
/// # Options
///
/// Each option may appear at most once per variant.
///
/// | Option | Meaning |
/// |---|---|
/// | `layer = base` | Ordinary page. This is the default. |
/// | `layer = stack_root` | Root of a navigation stack, such as a bottom-tab destination. |
/// | `layer = stack_page` | Full-screen page pushed above a stack root. |
/// | `layer = sheet` | Routed sheet presented over whatever page was showing. |
/// | `forward_to = Route` / `forward_to = (A, B)` | Directed drill-down: moving to a listed variant is `Forward`, and moving from it back to this variant is `Backward`. |
/// | `peers(group = g, order = field)` | Ordered siblings, such as tabs. Moving to a greater `order` is `Forward`, to a smaller one `Backward`, to an equal one `None`. |
/// | `peers(group = g, key = field, order = field)` | As above, but only routes whose `key` fields are equal are peers. `key = (a, b)` compares several fields. |
/// | `history = replace` | Navigating between two values of this variant replaces the current history entry. |
/// | `history = replace(key = field)` | As above, but only when the `key` fields are equal. |
/// | `handoff_from = Route` / `handoff_from = (A, B)` | Arriving here from a listed variant replaces that variant's history entry, so Back skips it. |
///
/// # Forward navigation
///
/// [`animated_navigate`] asks the current route for `transition_to(next)`.
/// The first matching rule wins:
///
/// | # | Rule | Transition | History |
/// |---|---|---|---|
/// | 1 | `current == next` | `None` (navigation is skipped entirely) | unchanged |
/// | 2 | `current` lists `next`'s variant in `forward_to` | `Forward` | push |
/// | 3 | `next` lists `current`'s variant in `forward_to` | `Backward` | push |
/// | 4 | `next` lists `current`'s variant in `handoff_from` | by `next`'s layer: `sheet` → `PresentSheet`, `stack_page` → `Forward`, otherwise `CrossFade` | **replace** |
/// | 5 | `current` is not a `sheet`, `next` is a `sheet` | `PresentSheet` | push* |
/// | 6 | `current` is a `sheet`, `next` is not | `DismissSheet` | push* |
/// | 7 | `stack_root` → `stack_page` | `Forward` | push* |
/// | 8 | `stack_page` → `stack_root` | `Backward` | push* |
/// | 9 | both in the same `peers` group (and `key`s match) | `Forward` / `Backward` / `None` by `order` | push* |
/// | 10 | same variant with a matching `history = replace` | `None` | **replace** |
/// | 11 | anything else | `CrossFade` | push* |
///
/// \* History is replaced instead whenever rule 4's or rule 10's condition
/// also holds, even if an earlier rule chose the animation. For example, tabs
/// declared with both `peers(...)` and `history = replace` slide *and*
/// replace history.
///
/// Layer pairs not listed in rules 5–8 have no built-in relationship:
/// `base` ↔ anything non-sheet, `stack_root` ↔ `stack_root`,
/// `stack_page` ↔ `stack_page`, and `sheet` ↔ `sheet` all fall through to
/// `peers`, `history`, and finally `CrossFade`. Use `forward_to` to make two
/// stack pages slide.
///
/// If two variants list each other in `forward_to`, rule 2 applies in both
/// directions, so both moves are `Forward`.
///
/// # Back navigation
///
/// [`try_animated_back`] and [`animated_back_or_navigate`] pop real router
/// history. The router does not reveal the destination until after the old
/// page has been captured, so `transition_back()` looks **only at the layer of
/// the route being left**:
///
/// | Leaving a… | Back transition |
/// |---|---|
/// | `sheet` | `DismissSheet` |
/// | `stack_page` | `Backward` |
/// | `base` or `stack_root` | `CrossFade` |
///
/// This means Back does not always mirror the forward animation. For example,
/// `forward_to` between two `base` routes slides forward but cross-fades on
/// Back, and two unrelated `stack_page` routes cross-fade forward but slide
/// on Back. Give drill-down destinations `layer = stack_page` (or
/// `layer = sheet`) when Back should mirror the forward motion.
///
/// # Requirements
///
/// - Variants must be unit variants or have named fields. Tuple variants are
///   rejected.
/// - `forward_to` and `handoff_from` must name other variants of the same
///   enum.
/// - Fields named by `key` must implement `PartialEq`, and `order` fields must
///   implement `Ord`. Every variant in a `peers` group must have fields with
///   those names and compatible types, because values are compared across
///   variants.
/// - `peers(...)` and keyed `history = replace(key = ...)` require named
///   fields.
///
/// # Example
///
/// ```ignore
/// #[derive(Clone, Routable, PartialEq, RouteTransitions)]
/// enum Route {
///     // Tab root. Changing `tab` swaps content in place without a history entry.
///     #[transition(layer = stack_root, history = replace)]
///     #[route("/items?:tab")]
///     Items { tab: String },
///
///     // Slides in over `Items` (rule 7) and slides back out (Back from a stack page).
///     // `forward_to` is needed because two stack pages otherwise cross-fade.
///     #[transition(layer = stack_page, forward_to = ItemComments)]
///     #[route("/items/:item_id")]
///     ItemDetails { item_id: String },
///
///     #[transition(layer = stack_page)]
///     #[route("/items/:item_id/comments")]
///     ItemComments { item_id: String },
///
///     // Rises over whichever page is showing and drops away on Back.
///     #[transition(layer = sheet)]
///     #[route("/items/new")]
///     NewItem {},
/// }
/// ```
///
/// [`animated_navigate`]: https://docs.rs/g3-route-transitions/latest/g3_route_transitions/fn.animated_navigate.html
/// [`try_animated_back`]: https://docs.rs/g3-route-transitions/latest/g3_route_transitions/fn.try_animated_back.html
/// [`animated_back_or_navigate`]: https://docs.rs/g3-route-transitions/latest/g3_route_transitions/fn.animated_back_or_navigate.html
#[proc_macro_derive(RouteTransitions, attributes(transition))]
pub fn derive_route_transitions(item: TokenStream) -> TokenStream {
    let route_enum = parse_macro_input!(item as ItemEnum);
    let mut route_variants = Vec::new();
    for variant in &route_enum.variants {
        let transition = match parse_transition_attr(variant) {
            Ok(transition) => transition,
            Err(error) => return error.to_compile_error().into(),
        };
        route_variants.push(RouteVariant {
            ident: variant.ident.clone(),
            fields: variant.fields.clone(),
            transition: transition.unwrap_or_default(),
        });
    }
    let enum_ident = &route_enum.ident;
    let layer_arms = match build_layer_arms(enum_ident, &route_variants) {
        Ok(arms) => arms,
        Err(error) => return error.to_compile_error().into(),
    };
    let peer_arms = match build_peer_ordering_arms(enum_ident, &route_variants) {
        Ok(arms) => arms,
        Err(error) => return error.to_compile_error().into(),
    };
    let forward_arms = match build_forward_arms(enum_ident, &route_variants) {
        Ok(arms) => arms,
        Err(error) => return error.to_compile_error().into(),
    };
    let replace_arms = match build_replace_arms(enum_ident, &route_variants) {
        Ok(arms) => arms,
        Err(error) => return error.to_compile_error().into(),
    };
    let hand_off_arms = match build_hand_off_arms(enum_ident, &route_variants) {
        Ok(arms) => arms,
        Err(error) => return error.to_compile_error().into(),
    };
    quote! {
        #[doc(hidden)] impl # enum_ident { fn __g3_route_transition_layer(&self) ->
        ::g3_route_transitions::RouteTransitionLayer { match self { # (# layer_arms),* }
        } fn __g3_route_transition_peer_ordering_to(& self, next : &# enum_ident) -> Option <
        std::cmp::Ordering > { match (self, next) { # (# peer_arms,) * _ => None, } } fn
        __g3_route_transition_is_forward_to(& self, next : &# enum_ident) -> bool { match (self,
        next) { # (# forward_arms,) * _ => false, } } fn __g3_route_transition_replaces_history_to(& self,
        next : &# enum_ident) -> bool { match (self, next) { # (# replace_arms,) * _ =>
        false, } } fn __g3_route_transition_hands_off_to(& self, next : &# enum_ident) -> bool { match
        (self, next) { # (# hand_off_arms,) * _ => false, } } } impl
        ::g3_route_transitions::RouteTransitions for # enum_ident { fn transition_to(&
        self, next : &# enum_ident,) -> ::g3_route_transitions::NavigationTransition { if
        self == next { return ::g3_route_transitions::NavigationTransition::None; } if self
        .__g3_route_transition_is_forward_to(next) { return
        ::g3_route_transitions::NavigationTransition::Forward; } if next
        .__g3_route_transition_is_forward_to(self) { return
        ::g3_route_transitions::NavigationTransition::Backward; } if self
        .__g3_route_transition_hands_off_to(next) { return match next.__g3_route_transition_layer() {
        ::g3_route_transitions::RouteTransitionLayer::Sheet => {
        ::g3_route_transitions::NavigationTransition::PresentSheet }
        ::g3_route_transitions::RouteTransitionLayer::StackPage => {
        ::g3_route_transitions::NavigationTransition::Forward }
        _ => ::g3_route_transitions::NavigationTransition::CrossFade, }; } match (self
        .__g3_route_transition_layer(), next.__g3_route_transition_layer()) { (current,
        ::g3_route_transitions::RouteTransitionLayer::Sheet,) if current !=
        ::g3_route_transitions::RouteTransitionLayer::Sheet => { return
        ::g3_route_transitions::NavigationTransition::PresentSheet },
        (::g3_route_transitions::RouteTransitionLayer::Sheet, next,) if next !=
        ::g3_route_transitions::RouteTransitionLayer::Sheet => { return
        ::g3_route_transitions::NavigationTransition::DismissSheet },
        (::g3_route_transitions::RouteTransitionLayer::StackRoot,
        ::g3_route_transitions::RouteTransitionLayer::StackPage,) => return
        ::g3_route_transitions::NavigationTransition::Forward,
        (::g3_route_transitions::RouteTransitionLayer::StackPage,
        ::g3_route_transitions::RouteTransitionLayer::StackRoot,) => return
        ::g3_route_transitions::NavigationTransition::Backward, _ => {} } if let
        Some(ordering) = self.__g3_route_transition_peer_ordering_to(next) { return match ordering {
        std::cmp::Ordering::Greater => {
        ::g3_route_transitions::NavigationTransition::Forward } std::cmp::Ordering::Less
        => { ::g3_route_transitions::NavigationTransition::Backward }
        std::cmp::Ordering::Equal => { ::g3_route_transitions::NavigationTransition::None
        } }; } if self.__g3_route_transition_replaces_history_to(next) { return
        ::g3_route_transitions::NavigationTransition::None; }
        ::g3_route_transitions::NavigationTransition::CrossFade } fn replaces_history(&
        self, next : &# enum_ident) -> bool { self.__g3_route_transition_replaces_history_to(next) || self
        .__g3_route_transition_hands_off_to(next) } fn
        transition_back(& self) -> ::g3_route_transitions::NavigationTransition { match
        self.__g3_route_transition_layer() { ::g3_route_transitions::RouteTransitionLayer::Sheet =>
        { ::g3_route_transitions::NavigationTransition::DismissSheet }
        ::g3_route_transitions::RouteTransitionLayer::StackPage => {
        ::g3_route_transitions::NavigationTransition::Backward }
        _ => ::g3_route_transitions::NavigationTransition::CrossFade, } } }
    }
    .into()
}
fn parse_transition_attr(variant: &syn::Variant) -> Result<Option<TransitionArgs>> {
    let mut transition = None;
    for attr in &variant.attrs {
        if attr.path().is_ident("transition") {
            if transition.is_some() {
                return Err(Error::new_spanned(attr, "duplicate transition attribute"));
            }
            transition = Some(attr.parse_args::<TransitionArgs>()?);
        }
    }
    Ok(transition)
}
fn build_layer_arms(
    enum_ident: &Ident,
    route_variants: &[RouteVariant],
) -> Result<Vec<TokenStream2>> {
    route_variants
        .iter()
        .map(|variant| {
            let pattern = build_layer_pattern(enum_ident, &variant.ident, &variant.fields)?;
            let layer = match variant.transition.layer {
                RouteLayer::Base => {
                    quote! {
                        ::g3_route_transitions::RouteTransitionLayer::Base
                    }
                }
                RouteLayer::Sheet => {
                    quote! {
                        ::g3_route_transitions::RouteTransitionLayer::Sheet
                    }
                }
                RouteLayer::StackRoot => {
                    quote! {
                        ::g3_route_transitions::RouteTransitionLayer::StackRoot
                    }
                }
                RouteLayer::StackPage => {
                    quote! {
                        ::g3_route_transitions::RouteTransitionLayer::StackPage
                    }
                }
            };
            Ok(quote! {
                # pattern => # layer
            })
        })
        .collect()
}
fn build_forward_arms(
    enum_ident: &Ident,
    route_variants: &[RouteVariant],
) -> Result<Vec<TokenStream2>> {
    let variants_by_name = route_variants
        .iter()
        .map(|variant| (variant.ident.to_string(), variant))
        .collect::<BTreeMap<_, _>>();
    let mut arms = Vec::new();
    for from in route_variants {
        let from_pattern = build_layer_pattern(enum_ident, &from.ident, &from.fields)?;
        for target in &from.transition.forward_to {
            let Some(to) = variants_by_name.get(&target.to_string()) else {
                return Err(Error::new_spanned(
                    target,
                    "forward_to target is not a route variant",
                ));
            };
            if to.ident == from.ident {
                return Err(Error::new_spanned(
                    target,
                    "a route cannot move forward to itself; use `peers(...)` to order values of one variant",
                ));
            }
            let to_pattern = build_layer_pattern(enum_ident, &to.ident, &to.fields)?;
            arms.push(quote! {
                (# from_pattern, # to_pattern) => true
            });
        }
    }
    Ok(arms)
}
fn build_replace_arms(
    enum_ident: &Ident,
    route_variants: &[RouteVariant],
) -> Result<Vec<TokenStream2>> {
    let mut arms = Vec::new();
    for variant in route_variants {
        let Some(replace) = &variant.transition.history_replace else {
            continue;
        };
        validate_named_fields(variant, replace.key.iter().collect(), "history replacement")?;
        if replace.key.is_empty() {
            let from = build_layer_pattern(enum_ident, &variant.ident, &variant.fields)?;
            let to = build_layer_pattern(enum_ident, &variant.ident, &variant.fields)?;
            arms.push(quote! {
                (# from, # to) => true
            });
            continue;
        }
        let from_aliases = key_aliases("__route_transition_replace_from", &replace.key);
        let to_aliases = key_aliases("__route_transition_replace_to", &replace.key);
        let from = build_key_pattern(enum_ident, &variant.ident, &variant.fields, &from_aliases)?;
        let to = build_key_pattern(enum_ident, &variant.ident, &variant.fields, &to_aliases)?;
        let guard = from_aliases
            .iter()
            .zip(&to_aliases)
            .map(|((_, from), (_, to))| {
                quote! {
                    # from == # to
                }
            })
            .collect::<Vec<_>>();
        arms.push(quote! {
            (# from, # to) if # (# guard) &&* => true
        });
    }
    Ok(arms)
}
fn build_hand_off_arms(
    enum_ident: &Ident,
    route_variants: &[RouteVariant],
) -> Result<Vec<TokenStream2>> {
    let variants_by_name = route_variants
        .iter()
        .map(|variant| (variant.ident.to_string(), variant))
        .collect::<BTreeMap<_, _>>();
    let mut arms = Vec::new();
    for to in route_variants {
        let to_pattern = build_layer_pattern(enum_ident, &to.ident, &to.fields)?;
        for source in &to.transition.handoff_from {
            let Some(from) = variants_by_name.get(&source.to_string()) else {
                return Err(Error::new_spanned(
                    source,
                    "handoff_from source is not a route variant",
                ));
            };
            if from.ident == to.ident {
                return Err(Error::new_spanned(
                    source,
                    "a route cannot hand off from itself; use `history = replace` for same-variant updates",
                ));
            }
            let from_pattern = build_layer_pattern(enum_ident, &from.ident, &from.fields)?;
            arms.push(quote! {
                (# from_pattern, # to_pattern) => true
            });
        }
    }
    Ok(arms)
}
fn build_peer_ordering_arms(
    enum_ident: &Ident,
    route_variants: &[RouteVariant],
) -> Result<Vec<TokenStream2>> {
    let mut groups: BTreeMap<String, Vec<&RouteVariant>> = BTreeMap::new();
    for variant in route_variants {
        if let Some(peers) = &variant.transition.peers {
            validate_peer_fields(variant, peers)?;
            groups
                .entry(peers.group.to_string())
                .or_default()
                .push(variant);
        }
    }
    let mut arms = Vec::new();
    for variants in groups.values() {
        for from in variants {
            for to in variants {
                let from_peers = from
                    .transition
                    .peers
                    .as_ref()
                    .expect("grouped peer variant");
                let to_peers = to.transition.peers.as_ref().expect("grouped peer variant");
                let from_aliases = FieldAliases::new("__route_transition_from", from_peers);
                let to_aliases = FieldAliases::new("__route_transition_to", to_peers);
                let from_pattern =
                    build_peer_pattern(enum_ident, &from.ident, &from.fields, &from_aliases)?;
                let to_pattern =
                    build_peer_pattern(enum_ident, &to.ident, &to.fields, &to_aliases)?;
                let key_guard = build_key_guard(&from_aliases, &to_aliases);
                let from_order = from_aliases.order_alias();
                let to_order = to_aliases.order_alias();
                arms.push(quote! {
                    (# from_pattern, # to_pattern) if # key_guard => Some(# to_order
                    .cmp(# from_order))
                });
            }
        }
    }
    Ok(arms)
}
fn validate_peer_fields(variant: &RouteVariant, peers: &PeerArgs) -> Result<()> {
    validate_named_fields(variant, peers.used_fields(), "peers")
}
fn validate_named_fields(
    variant: &RouteVariant,
    used_fields: Vec<&Ident>,
    transition_name: &str,
) -> Result<()> {
    if used_fields.is_empty() {
        return Ok(());
    }
    let Fields::Named(fields) = &variant.fields else {
        return Err(Error::new_spanned(
            &variant.ident,
            format!("{transition_name} transitions require named route fields"),
        ));
    };
    let field_names = fields
        .named
        .iter()
        .filter_map(|field| field.ident.as_ref().map(ToString::to_string))
        .collect::<BTreeSet<_>>();
    for field in used_fields {
        if !field_names.contains(&field.to_string()) {
            return Err(Error::new_spanned(
                field,
                format!("transition {transition_name} field is not present on this route variant",),
            ));
        }
    }
    Ok(())
}
fn key_aliases(prefix: &str, fields: &[Ident]) -> Vec<(Ident, Ident)> {
    fields
        .iter()
        .cloned()
        .map(|field| {
            let alias = format_ident!("{}_{}", prefix, field);
            (field, alias)
        })
        .collect()
}
fn build_key_pattern(
    enum_ident: &Ident,
    variant_ident: &Ident,
    fields: &Fields,
    aliases: &[(Ident, Ident)],
) -> Result<TokenStream2> {
    match fields {
        Fields::Named(_) => {
            let bindings = aliases
                .iter()
                .map(|(field, alias)| {
                    quote! {
                        # field : # alias
                    }
                })
                .collect::<Vec<_>>();
            Ok(quote! {
                # enum_ident::# variant_ident { # (# bindings),*, .. }
            })
        }
        Fields::Unnamed(_) | Fields::Unit => Err(Error::new_spanned(
            variant_ident,
            "keyed history replacement requires named route fields",
        )),
    }
}
fn build_layer_pattern(
    enum_ident: &Ident,
    variant_ident: &Ident,
    fields: &Fields,
) -> Result<TokenStream2> {
    match fields {
        Fields::Named(_) => Ok(quote! {
            # enum_ident::# variant_ident { .. }
        }),
        Fields::Unnamed(_) => Err(Error::new_spanned(
            variant_ident,
            "route transition macro only supports named or unit route variants",
        )),
        Fields::Unit => Ok(quote! {
            # enum_ident::# variant_ident
        }),
    }
}
fn build_peer_pattern(
    enum_ident: &Ident,
    variant_ident: &Ident,
    fields: &Fields,
    aliases: &FieldAliases,
) -> Result<TokenStream2> {
    match fields {
        Fields::Named(_) => {
            let bindings = aliases.bindings();
            Ok(quote! {
                # enum_ident::# variant_ident { # (# bindings),*, .. }
            })
        }
        Fields::Unnamed(_) => Err(Error::new_spanned(
            variant_ident,
            "route transition macro only supports named or unit route variants",
        )),
        Fields::Unit => Err(Error::new_spanned(
            variant_ident,
            "ordered peer transitions require named route fields",
        )),
    }
}
fn build_key_guard(from_aliases: &FieldAliases, to_aliases: &FieldAliases) -> TokenStream2 {
    let comparisons = from_aliases
        .key_aliases()
        .into_iter()
        .zip(to_aliases.key_aliases())
        .map(|(from, to)| {
            quote! {
                # from == # to
            }
        })
        .collect::<Vec<_>>();
    if comparisons.is_empty() {
        quote! {
            true
        }
    } else {
        quote! {
            # (# comparisons) &&*
        }
    }
}
#[derive(Clone)]
struct RouteVariant {
    ident: Ident,
    fields: Fields,
    transition: TransitionArgs,
}
#[derive(Clone)]
struct TransitionArgs {
    layer: RouteLayer,
    peers: Option<PeerArgs>,
    forward_to: Vec<Ident>,
    history_replace: Option<ReplaceHistoryArgs>,
    handoff_from: Vec<Ident>,
}
impl Default for TransitionArgs {
    fn default() -> Self {
        Self {
            layer: RouteLayer::Base,
            peers: None,
            forward_to: Vec::new(),
            history_replace: None,
            handoff_from: Vec::new(),
        }
    }
}
impl Parse for TransitionArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut args = Self::default();
        let mut seen = BTreeSet::new();
        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            if !seen.insert(ident.to_string()) {
                return Err(Error::new_spanned(
                    &ident,
                    format!("duplicate transition argument `{ident}`"),
                ));
            }
            match ident.to_string().as_str() {
                "layer" => {
                    input.parse::<Token![=]>()?;
                    let layer: Ident = input.parse()?;
                    args.layer = match layer.to_string().as_str() {
                        "base" => RouteLayer::Base,
                        "sheet" => RouteLayer::Sheet,
                        "stack_root" => RouteLayer::StackRoot,
                        "stack_page" => RouteLayer::StackPage,
                        _ => {
                            return Err(Error::new_spanned(
                                layer,
                                "unknown transition layer; expected base, stack_root, stack_page, or sheet",
                            ));
                        }
                    };
                }
                "peers" => {
                    let content;
                    parenthesized!(content in input);
                    args.peers = Some(content.parse()?);
                }
                "forward_to" => {
                    input.parse::<Token![=]>()?;
                    args.forward_to = parse_ident_list(input)?;
                }
                "history" => {
                    input.parse::<Token![=]>()?;
                    let action: Ident = input.parse()?;
                    if action != "replace" {
                        return Err(Error::new_spanned(
                            action,
                            "unknown history action; expected replace",
                        ));
                    }
                    args.history_replace = if input.peek(syn::token::Paren) {
                        let content;
                        parenthesized!(content in input);
                        Some(content.parse()?)
                    } else {
                        Some(ReplaceHistoryArgs::default())
                    };
                }
                "handoff_from" => {
                    input.parse::<Token![=]>()?;
                    args.handoff_from = parse_ident_list(input)?;
                }
                _ => return Err(Error::new_spanned(ident, "unknown transition argument")),
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(args)
    }
}
#[derive(Clone, Copy)]
enum RouteLayer {
    Base,
    Sheet,
    StackRoot,
    StackPage,
}
#[derive(Clone, Default)]
struct ReplaceHistoryArgs {
    key: Vec<Ident>,
}
impl Parse for ReplaceHistoryArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let name: Ident = input.parse()?;
        if name != "key" {
            return Err(Error::new_spanned(
                name,
                "unknown history replacement argument",
            ));
        }
        input.parse::<Token![=]>()?;
        let key = parse_key(input)?;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
        if !input.is_empty() {
            return Err(input.error("history replacement accepts only a key argument"));
        }
        Ok(Self { key })
    }
}
#[derive(Clone)]
struct PeerArgs {
    group: Ident,
    key: Vec<Ident>,
    order: Ident,
}
impl PeerArgs {
    fn used_fields(&self) -> Vec<&Ident> {
        let mut fields = self.key.iter().collect::<Vec<_>>();
        if !fields.contains(&&self.order) {
            fields.push(&self.order);
        }
        fields
    }
}
impl Parse for PeerArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut group = None;
        let mut key = Vec::new();
        let mut order = None;
        let mut seen = BTreeSet::new();
        while !input.is_empty() {
            let name: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            if !seen.insert(name.to_string()) {
                return Err(Error::new_spanned(
                    &name,
                    format!("duplicate peers argument `{name}`"),
                ));
            }
            match name.to_string().as_str() {
                "group" => group = Some(input.parse()?),
                "key" => key = parse_key(input)?,
                "order" => order = Some(input.parse()?),
                _ => return Err(Error::new_spanned(name, "unknown peers argument")),
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Self {
            group: group.ok_or_else(|| input.error("missing peers group"))?,
            key,
            order: order.ok_or_else(|| input.error("missing peers order"))?,
        })
    }
}
fn parse_key(input: ParseStream<'_>) -> Result<Vec<Ident>> {
    if input.peek(syn::token::Paren) {
        let content;
        parenthesized!(content in input);
        Ok(Punctuated::<Ident, Token![,]>::parse_terminated(&content)?
            .into_iter()
            .collect())
    } else {
        Ok(vec![input.parse()?])
    }
}
fn parse_ident_list(input: ParseStream<'_>) -> Result<Vec<Ident>> {
    if input.peek(syn::token::Paren) {
        let content;
        parenthesized!(content in input);
        Ok(Punctuated::<Ident, Token![,]>::parse_terminated(&content)?
            .into_iter()
            .collect())
    } else {
        Ok(vec![input.parse()?])
    }
}
struct FieldAliases {
    aliases: Vec<(Ident, Ident)>,
    key_len: usize,
}
impl FieldAliases {
    fn new(prefix: &str, peers: &PeerArgs) -> Self {
        let mut fields = peers.key.clone();
        if !fields.iter().any(|field| field == &peers.order) {
            fields.push(peers.order.clone());
        }
        let aliases = fields
            .into_iter()
            .map(|field| {
                let alias = format_ident!("{}_{}", prefix, field);
                (field, alias)
            })
            .collect();
        Self {
            aliases,
            key_len: peers.key.len(),
        }
    }
    fn bindings(&self) -> Vec<TokenStream2> {
        self.aliases
            .iter()
            .map(|(field, alias)| {
                quote! {
                    # field : # alias
                }
            })
            .collect()
    }
    fn key_aliases(&self) -> Vec<&Ident> {
        self.aliases
            .iter()
            .take(self.key_len)
            .map(|(_, alias)| alias)
            .collect()
    }
    fn order_alias(&self) -> &Ident {
        &self.aliases.last().expect("push aliases include order").1
    }
}
