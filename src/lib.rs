#![warn(missing_docs)]
//! Route-owned View Transition helpers for Dioxus Router.
//!
//! Using the crate takes three pieces, and all three are needed before every
//! transition is visible:
//!
//! 1. **Route metadata.** `#[derive(RouteTransitions)]` on your `Routable`
//!    enum decides *which* [`NavigationTransition`] runs between two routes.
//! 2. **Snapshot regions.** Wrapper components mark *which parts of the DOM*
//!    move. A transition only animates the regions it knows about (see
//!    [What moves](#what-moves)).
//! 3. **Animated navigation.** [`animated_navigate`], [`try_animated_back`],
//!    and [`animated_back_or_navigate`] capture the old page before changing
//!    the route. Calling `navigator().push(...)` directly skips the animation.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use dioxus::prelude::*;
//! use g3_route_transitions::{
//!     RouteTransitionApp, RouteTransitionBaseRegion, RouteTransitionPage, RouteTransitions,
//!     animated_back_or_navigate, animated_navigate,
//! };
//!
//! #[derive(Clone, PartialEq, Routable, RouteTransitions)]
//! enum Route {
//!     #[transition(layer = stack_root)]
//!     #[route("/")]
//!     Home {},
//!
//!     #[transition(layer = stack_page)]
//!     #[route("/details/:id")]
//!     Details { id: u32 },
//!
//!     #[transition(layer = sheet)]
//!     #[route("/compose")]
//!     Compose {},
//! }
//!
//! #[component]
//! fn App() -> Element {
//!     // The overlay region: the part that rises and falls for sheets.
//!     rsx! { RouteTransitionApp { Router::<Route> {} } }
//! }
//!
//! // Ordinary (non-sheet) pages sit in a base region and wrap their content in
//! // a page region. The base region stays put and dims under a sheet; the page
//! // region slides for `Forward`/`Backward`.
//! #[component]
//! fn Home() -> Element {
//!     rsx! {
//!         RouteTransitionBaseRegion {
//!             RouteTransitionPage {
//!                 h1 { "Home" }
//!                 button {
//!                     onclick: move |_| async move { animated_navigate(Route::Details { id: 1 }).await },
//!                     "Open details"
//!                 }
//!                 button {
//!                     onclick: move |_| async move { animated_navigate(Route::Compose {}).await },
//!                     "Compose"
//!                 }
//!             }
//!         }
//!     }
//! }
//!
//! #[component]
//! fn Details(id: u32) -> Element {
//!     rsx! {
//!         RouteTransitionBaseRegion {
//!             RouteTransitionPage {
//!                 button {
//!                     onclick: move |_| async move { animated_back_or_navigate(Route::Home {}).await },
//!                     "Back"
//!                 }
//!                 "Details {id}"
//!             }
//!         }
//!     }
//! }
//!
//! // Sheets render *without* a base or page region, so the whole sheet is
//! // captured as part of the overlay.
//! #[component]
//! fn Compose() -> Element {
//!     rsx! {
//!         button {
//!             onclick: move |_| async move { animated_back_or_navigate(Route::Home {}).await },
//!             "Close"
//!         }
//!     }
//! }
//!
//! fn main() {
//!     dioxus::launch(App);
//! }
//! ```
//!
//! With this setup:
//!
//! | Navigation | Transition |
//! |---|---|
//! | Home → Details | `Forward`, and Back is `Backward` |
//! | Home or Details → Compose | `PresentSheet`, and Back is `DismissSheet` |
//! | Details(1) → Details(2) | `CrossFade`, since nothing relates two `stack_page` values |
//!
//! The [`RouteTransitions` derive](derive@RouteTransitions) documents every
//! option and the exact rule order.
//!
//! # What moves
//!
//! Each transition animates specific snapshot regions. Anything outside them
//! is part of the root snapshot.
//!
//! | Transition | Animated regions | Everything else |
//! |---|---|---|
//! | `CrossFade` | root and [`RouteTransitionBaseRegion`] fade | – |
//! | `Forward` / `Backward` | [`RouteTransitionPage`] slides with platform motion; a [`RouteTransitionSegment`] *outside* any page slides as a full-width filmstrip | root and base region switch instantly |
//! | `PresentSheet` / `DismissSheet` | [`RouteTransitionOverlayRegion`] (added by [`RouteTransitionApp`]) rises or falls; the base region and page region underneath dim (and on iOS, scale) | root is hidden |
//! | `None` | nothing; the route changes without a View Transition | – |
//!
//! In every transition, [`RouteTransitionPersistent`] stays in place above
//! all other snapshots, such as a desktop rail beside a rising sheet.
//!
//! Consequences worth knowing:
//!
//! - [`RouteTransitionApp`] on its own only produces visible cross-fades.
//!   `Forward`/`Backward` need a [`RouteTransitionPage`] or
//!   [`RouteTransitionSegment`], and sheets need a base region to rise over.
//! - A sheet route must not render a base region or page region. Those are
//!   captured separately from the overlay, so the sheet's content would be
//!   missing from the rising image.
//! - A segment inside a [`RouteTransitionPage`] never moves by itself; the
//!   whole page moves instead. Use segments without a page wrapper for tabbed
//!   content that should slide under a fixed header.
//! - Persistent chrome only stays still if both routes render it in the same
//!   place, sheet routes included. Otherwise it fades.
//! - Each region may appear at most once in the document at a time. Duplicate
//!   `view-transition-name`s make the browser skip the animation.
//!
//! # Platform motion
//!
//! [`Platform`] picks the visual style for the same semantic transition.
//! Call [`set_platform`] at startup (and whenever the app's mode changes);
//! otherwise [`get_platform`] falls back to [`detect_platform`].
//!
//! Animations are skipped, and the route is changed directly, when the
//! browser lacks `document.startViewTransition` or the user prefers reduced
//! motion.
mod browser_history;

pub use browser_history::use_browser_history_transitions;
use dioxus::{document::eval, prelude::*};
pub use g3_route_transitions_macros::RouteTransitions;
use manganis::{Asset, asset};
/// The stylesheet backing every transition. Link it once via
/// [`RouteTransitionStyles`] (or [`RouteTransitionApp`]), or attach it
/// yourself if the app manages its own `document::Link` tags.
///
/// Besides the transition rules, the stylesheet gives `html` and `body`
/// `min-height: 100%` and a background of `--route-transition-bg` (falling
/// back to `--color-bg`, then `#f8f8f8`). The region classes also set layout
/// styles; see each class constant.
pub static ROUTE_TRANSITIONS_CSS: Asset = asset!("/assets/route_transitions.css");
/// Class applied by [`RouteTransitionBaseRegion`].
///
/// The element always has `view-transition-name: base`. It cross-fades in
/// `CrossFade`, switches instantly in `Forward`/`Backward`, and dims under a
/// sheet. It is styled as a full-height flex column
/// (`display: flex; flex-direction: column; min-height: 100dvh`) with an
/// opaque background. A base region that is a direct child of a
/// [`RouteTransitionPage`] is not named.
pub const ROUTE_TRANSITION_BASE_REGION_CLASS: &str = "route-transition-base-region";
/// Class applied by [`RouteTransitionOverlayRegion`] and
/// [`RouteTransitionApp`].
///
/// The element is named `overlay` only during `PresentSheet` and
/// `DismissSheet`, when it rises or falls above the base region. It is styled
/// as a full-height flex column with an opaque background.
pub const ROUTE_TRANSITION_OVERLAY_REGION_CLASS: &str = "route-transition-overlay-region";
/// Class applied by [`RouteTransitionSegment`].
///
/// The element is named `segment` only during `Forward` and `Backward`, when
/// it slides the full width of its own box as a filmstrip on both platforms.
/// Inside a [`RouteTransitionPage`] it is not named.
pub const ROUTE_TRANSITION_SEGMENT_CLASS: &str = "route-transition-segment";
/// Class applied by [`RouteTransitionPage`].
///
/// The element is named `page` during `Forward`, `Backward`, `PresentSheet`,
/// and `DismissSheet`. It slides with platform motion for the first two and
/// dims under a sheet for the others. It is styled as a viewport-height flex
/// column (`height: 100dvh; overflow: hidden`) with an opaque background.
pub const ROUTE_TRANSITION_PAGE_CLASS: &str = "route-transition-page";
/// Class applied by [`RouteTransitionPersistent`].
///
/// The element is always named `persistent` and never animates. It paints
/// above page snapshots and below a routed sheet. If only one of the two
/// routes renders it, it fades in or out instead. The stylesheet applies no
/// layout to it. Libraries that only want this behavior at some breakpoints
/// can set `view-transition-name: persistent` themselves instead of using the
/// class.
pub const ROUTE_TRANSITION_PERSISTENT_CLASS: &str = "route-transition-persistent";
fn merge_transition_class(base: &'static str, extra: Option<&str>) -> String {
    match extra {
        Some(extra) if !extra.is_empty() => format!("{base} {extra}"),
        _ => base.to_string(),
    }
}
/// Content that stays in place during navigation and dims underneath a
/// presented sheet: typically an ordinary page's shell, including a tab bar.
///
/// Render one in every non-sheet route. Sheet routes must not render one,
/// or the sheet's content is captured here instead of rising with the
/// overlay. See [What moves](crate#what-moves).
///
/// `class` is appended to [`ROUTE_TRANSITION_BASE_REGION_CLASS`].
#[component]
pub fn RouteTransitionBaseRegion(children: Element, class: Option<String>) -> Element {
    let class = merge_transition_class(ROUTE_TRANSITION_BASE_REGION_CLASS, class.as_deref());
    rsx! {
        div { class, {children} }
    }
}
/// Content that rises over the base region for `PresentSheet` and falls away
/// for `DismissSheet`.
///
/// [`RouteTransitionApp`] already provides one around the whole app, which is
/// what most applications want. Use this component directly only when the
/// app shell is assembled from [`RouteTransitionStyles`] by hand. Render at
/// most one at a time.
///
/// `class` is appended to [`ROUTE_TRANSITION_OVERLAY_REGION_CLASS`].
#[component]
pub fn RouteTransitionOverlayRegion(children: Element, class: Option<String>) -> Element {
    let class = merge_transition_class(ROUTE_TRANSITION_OVERLAY_REGION_CLASS, class.as_deref());
    rsx! {
        div { class, {children} }
    }
}
/// Content that slides by itself for `Forward` and `Backward`, such as the
/// body of a segmented control under a fixed header.
///
/// Both the outgoing and incoming segments travel the full width of the box
/// together, clipped to it. This is the same on iOS and Material. Everything
/// outside the segment switches instantly. A segment inside a
/// [`RouteTransitionPage`] does not move by itself, because the page moves
/// instead.
///
/// `class` is appended to [`ROUTE_TRANSITION_SEGMENT_CLASS`].
#[component]
pub fn RouteTransitionSegment(children: Element, class: Option<String>) -> Element {
    let class = merge_transition_class(ROUTE_TRANSITION_SEGMENT_CLASS, class.as_deref());
    rsx! {
        div { class, {children} }
    }
}
/// One routed page, captured as a single viewport-sized image for `Forward`
/// and `Backward` (platform push/pop motion) and dimmed under sheets.
///
/// This is the region that makes stack navigation visible. Place it inside
/// the page's [`RouteTransitionBaseRegion`], around the header and body, and
/// leave persistent chrome such as a tab bar outside it so that chrome stays
/// still. Segments and direct-child base regions inside the page are
/// suppressed so the page moves as one piece.
///
/// `class` is appended to [`ROUTE_TRANSITION_PAGE_CLASS`].
#[component]
pub fn RouteTransitionPage(children: Element, class: Option<String>) -> Element {
    let class = merge_transition_class(ROUTE_TRANSITION_PAGE_CLASS, class.as_deref());
    rsx! {
        div { class, {children} }
    }
}
/// Chrome that stays exactly where it is during every transition, such as a
/// desktop navigation rail beside the page.
///
/// It is captured on its own and painted above page snapshots, so it doesn't
/// slide with the page or dim under a sheet. Routed sheets paint above it.
/// For it to stay still, the routes on both sides of a transition must
/// render it in the same place, including sheet routes. If only one side
/// renders it, it fades. Render at most one at a time.
///
/// `class` is appended to [`ROUTE_TRANSITION_PERSISTENT_CLASS`].
#[component]
pub fn RouteTransitionPersistent(children: Element, class: Option<String>) -> Element {
    let class = merge_transition_class(ROUTE_TRANSITION_PERSISTENT_CLASS, class.as_deref());
    rsx! {
        div { class, {children} }
    }
}
/// A semantic navigation event chosen by [`RouteTransitions::transition_to`]
/// or [`RouteTransitions::transition_back`].
///
/// The variant says *what happened* (drill in, present a sheet, and so on).
/// The stylesheet decides how it looks for the current [`Platform`], and
/// which regions move is described in [What moves](crate#what-moves). While a
/// transition runs, [`data_value`](Self::data_value) is published on `<html>`
/// as `data-route-transition`.
///
/// Directions are physical. `Forward` always brings the new page in from the
/// right, including in right-to-left documents.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationTransition {
    /// No View Transition: the route changes immediately. The derive returns
    /// this for identical routes, equal peer `order`s, and same-variant
    /// `history = replace` updates.
    None,
    /// A quick cross-dissolve of the root and base region, the same on both
    /// platforms. This is what unrelated routes get.
    #[default]
    CrossFade,
    /// Drill in: the new page enters from the right. On iOS it slides over the
    /// old page, which shifts 30% left and dims (UINavigationController push).
    /// On Material both pages move 30px with a fade-through (Shared Axis X).
    /// A standalone segment slides full-width on both platforms.
    Forward,
    /// Drill out: the exact reverse of [`Forward`](Self::Forward).
    Backward,
    /// A routed sheet rises from the bottom over the current page, which dims
    /// in place. On iOS the page also scales down and rounds its corners. On
    /// Material the sheet moves 20% while fading in.
    PresentSheet,
    /// The sheet falls away to reveal the page beneath: the reverse of
    /// [`PresentSheet`](Self::PresentSheet).
    DismissSheet,
}
impl NavigationTransition {
    /// The `data-route-transition` attribute value the stylesheet keys on:
    /// `none`, `cross-fade`, `forward`, `backward`, `present-sheet`, or
    /// `dismiss-sheet`.
    pub fn data_value(self) -> &'static str {
        match self {
            NavigationTransition::None => "none",
            NavigationTransition::CrossFade => "cross-fade",
            NavigationTransition::Forward => "forward",
            NavigationTransition::Backward => "backward",
            NavigationTransition::PresentSheet => "present-sheet",
            NavigationTransition::DismissSheet => "dismiss-sheet",
        }
    }
}
/// Platform styling mode for the transition stylesheet - mirrors the
/// iOS-vs-Material split app component libraries (e.g. Ionic, g3-ui) already
/// use for widget styling, so the same signal can drive both.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    /// UINavigationController/UIKit-flavored motion: parallax push where the
    /// outgoing page slides back and dims rather than leaving the screen,
    /// page-sheet presentation, quick cross-dissolves.
    Ios,
    /// Material Design 3 motion: shared-axis slides, quick cross-dissolves,
    /// and bottom sheets with a scrim.
    Material,
}
impl Platform {
    /// The `data-route-transition-platform` attribute value the stylesheet
    /// keys on.
    pub fn data_value(self) -> &'static str {
        match self {
            Platform::Ios => "ios",
            Platform::Material => "material",
        }
    }
}
thread_local! {
    static GLOBAL_PLATFORM: std::cell::Cell<Option<Platform>> = const {
        std::cell::Cell::new(None)
    };
}
/// Explicitly set the platform used to pick a transition's visual style.
/// Call this once at startup and again whenever the app's platform mode
/// changes (e.g. a user-facing iOS/Material style toggle).
pub fn set_platform(platform: Platform) {
    GLOBAL_PLATFORM.with(|p| p.set(Some(platform)));
}
/// The platform currently used to style transitions. Falls back to
/// compile-time/runtime auto-detection if [`set_platform`] was never called.
pub fn get_platform() -> Platform {
    GLOBAL_PLATFORM
        .with(|p| p.get())
        .unwrap_or_else(detect_platform)
}
/// Detect a reasonable default platform from `cfg(target_os)` (native
/// mobile builds) or, on wasm, from the user agent.
pub fn detect_platform() -> Platform {
    #[cfg(target_os = "ios")]
    {
        Platform::Ios
    }
    #[cfg(target_os = "android")]
    {
        Platform::Material
    }
    #[cfg(all(
        target_arch = "wasm32",
        not(target_os = "ios"),
        not(target_os = "android")
    ))]
    {
        detect_platform_web()
    }
    #[cfg(not(any(target_os = "ios", target_os = "android", target_arch = "wasm32")))]
    {
        Platform::Material
    }
}
#[cfg(target_arch = "wasm32")]
fn detect_platform_web() -> Platform {
    let Some(window) = web_sys::window() else {
        return Platform::Material;
    };
    let navigator = window.navigator();
    let user_agent = navigator.user_agent().unwrap_or_default().to_lowercase();
    let platform = navigator.platform().unwrap_or_default().to_lowercase();
    let max_touch_points = navigator.max_touch_points();
    let is_iphone_or_ipod = user_agent.contains("iphone") || user_agent.contains("ipod");
    let is_ipad = user_agent.contains("ipad")
        || (platform.contains("mac") && max_touch_points > 1 && user_agent.contains("safari"));
    if is_iphone_or_ipod || is_ipad {
        Platform::Ios
    } else {
        Platform::Material
    }
}
/// Initialize the global platform from compile-time/runtime auto-detection.
/// Prefer calling [`set_platform`] directly when the host app already tracks
/// an explicit iOS/Material mode.
pub fn init_auto_platform() {
    set_platform(detect_platform());
}
/// The navigation role a route declares with `#[transition(layer = ...)]`.
///
/// Code generated by the [`RouteTransitions` derive](derive@RouteTransitions)
/// uses the layer to pick transitions. It does not choose which DOM element
/// moves; snapshot regions do that. See the derive documentation for the rules
/// each layer takes part in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RouteTransitionLayer {
    /// `layer = base`: an ordinary page with no stack or sheet role. This is
    /// the default.
    #[default]
    Base,
    /// `layer = sheet`: presented over the previous page with `PresentSheet`
    /// and removed with `DismissSheet`.
    Sheet,
    /// `layer = stack_root`: the root of a navigation stack, such as a
    /// bottom-tab destination. Moving to a `StackPage` is `Forward`.
    StackRoot,
    /// `layer = stack_page`: a full-screen page above a stack root. Moving to
    /// a `StackRoot` and Back are both `Backward`.
    StackPage,
}
/// Decides which [`NavigationTransition`] runs between two routes and how
/// history is written.
///
/// Usually generated by the [`RouteTransitions` derive](derive@RouteTransitions),
/// whose documentation lists the exact rules. Implement it by hand when the
/// choice depends on route data the derive cannot express.
pub trait RouteTransitions: PartialEq {
    /// The transition for [`animated_navigate`] from `self` to `next`.
    ///
    /// Returning [`NavigationTransition::None`] changes the route without a
    /// View Transition.
    fn transition_to(&self, next: &Self) -> NavigationTransition;
    /// Whether [`animated_navigate`] from `self` to `next` replaces the current
    /// history entry instead of pushing a new one.
    ///
    /// The derive returns `true` for matching `history = replace` rules and
    /// for `handoff_from` sources. The default always pushes.
    fn replaces_history(&self, _next: &Self) -> bool {
        false
    }
    /// The transition for [`try_animated_back`] and
    /// [`animated_back_or_navigate`] when leaving `self`.
    ///
    /// No destination is passed, because the router does not reveal the
    /// previous entry until after the old page has been captured. The derive
    /// uses only `self`'s layer: `DismissSheet` for a sheet, `Backward` for a
    /// stack page, and `CrossFade` otherwise. The default is `CrossFade`.
    fn transition_back(&self) -> NavigationTransition {
        NavigationTransition::CrossFade
    }
}
/// Links [`ROUTE_TRANSITIONS_CSS`] and renders its children unchanged.
///
/// Use it when building the app shell from
/// [`RouteTransitionOverlayRegion`] by hand. Otherwise use
/// [`RouteTransitionApp`], which includes it. It provides no context.
#[component]
pub fn RouteTransitionStyles(children: Element) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: ROUTE_TRANSITIONS_CSS }
        {children}
    }
}
/// The app-level wrapper: links the stylesheet and wraps its children in an
/// overlay region (the part that rises and falls for sheets). Place
/// `Router::<Route> {}` inside it.
///
/// On its own this only produces visible cross-fades. Pages also need a
/// [`RouteTransitionBaseRegion`] and a [`RouteTransitionPage`] (or
/// [`RouteTransitionSegment`]) for stack and sheet motion; see
/// [What moves](crate#what-moves).
///
/// `class` is appended to [`ROUTE_TRANSITION_OVERLAY_REGION_CLASS`].
#[component]
pub fn RouteTransitionApp(children: Element, class: Option<String>) -> Element {
    let class = merge_transition_class(ROUTE_TRANSITION_OVERLAY_REGION_CLASS, class.as_deref());
    rsx! {
        RouteTransitionStyles {
            div { class, {children} }
        }
    }
}
/// Placeholders substituted before the script is evaluated.
///
/// These used to arrive over the eval channel, which cost two round trips
/// between wasm and JS before the transition could start — dead time between
/// the tap and the first frame of the animation. Both values are known on the
/// Rust side already, so they are written into the script instead. The
/// animation and platform come from fixed enums, so there is nothing to escape;
/// the route paths are written as escaped string literals, placeholder quotes
/// included.
const ANIMATION_PLACEHOLDER: &str = "__DX_ROUTE_TRANSITION_ANIMATION__";
const PLATFORM_PLACEHOLDER: &str = "__DX_ROUTE_TRANSITION_PLATFORM__";
const FROM_PLACEHOLDER: &str = "\"__DX_ROUTE_TRANSITION_FROM__\"";
const TO_PLACEHOLDER: &str = "\"__DX_ROUTE_TRANSITION_TO__\"";
const VIEW_TRANSITION_NAVIGATE: &str = r#"
const animation = "__DX_ROUTE_TRANSITION_ANIMATION__";
const platform = "__DX_ROUTE_TRANSITION_PLATFORM__";
// The route being left, and the one being entered when it is known. Back does
// not know its destination until the router has popped, so it leaves `to`
// empty. Published on <html> so app CSS can scope snapshot naming to the routes
// involved - one sheet opening can want an element lifted that another does not.
const from = "__DX_ROUTE_TRANSITION_FROM__";
const to = "__DX_ROUTE_TRANSITION_TO__";
const prefersReducedMotion = window.matchMedia?.("(prefers-reduced-motion: reduce)")?.matches ?? false;

// View-transition snapshots are painted in a document-level pseudo tree. They
// do not inherit a theme that is scoped to an app shell, and they are not
// clipped by an embedded shell's rounded/overflow-hidden ancestor. Capture the
// resolved paint values for both routes and publish them on <html>, where the
// pseudo tree can inherit them.
const dxRouteTransitionOpaqueBackground = (element) => {
    if (!element) return null;
    const color = window.getComputedStyle(element).backgroundColor;
    return color && color !== "transparent" && color !== "rgba(0, 0, 0, 0)" ? color : null;
};
const dxRouteTransitionSurfaceBackground = () => {
    const root = document.documentElement;
    const surface = document.querySelector(".route-transition-page, .route-transition-base-region");
    return dxRouteTransitionOpaqueBackground(surface)
        ?? dxRouteTransitionOpaqueBackground(document.body)
        ?? dxRouteTransitionOpaqueBackground(root)
        ?? "Canvas";
};
const dxRouteTransitionPaintContext = () => {
    const root = document.documentElement;
    const surface = document.querySelector(".route-transition-page, .route-transition-base-region");
    const surfaceBackground = dxRouteTransitionSurfaceBackground();
    const documentBackground = dxRouteTransitionOpaqueBackground(document.body)
        ?? dxRouteTransitionOpaqueBackground(root)
        ?? surfaceBackground;

    let clipRadius = "0px";
    for (let current = surface; current && current !== document.body; current = current.parentElement) {
        const style = window.getComputedStyle(current);
        const clips = [style.overflow, style.overflowX, style.overflowY]
            .some((value) => value === "hidden" || value === "clip");
        const radius = style.borderTopLeftRadius;
        if (clips && radius && radius !== "0px") {
            clipRadius = radius;
            break;
        }
    }

    root.style.setProperty("--route-transition-surface-bg", surfaceBackground);
    root.style.setProperty("--route-transition-document-bg", documentBackground);
    root.style.setProperty("--route-transition-clip-radius", clipRadius);
};
const dxRouteTransitionIncomingPaintContext = () => {
    document.documentElement.style.setProperty(
        "--route-transition-incoming-surface-bg",
        dxRouteTransitionSurfaceBackground(),
    );
};
// Resolves once the router has replaced the page, or after a ceiling if it
// renders something indistinguishable.
//
// This must be started *before* the route is asked for, so the observer is
// already live when the update lands. Starting it afterwards meant the mutation
// had usually already happened, leaving nothing to observe and the ceiling as
// the de facto wait — measured at a flat ~133ms of doing nothing after the new
// route was on screen, on every navigation.
//
// There is deliberately no requestAnimationFrame fallback: rendering is paused
// inside a view transition's update callback, so frames do not tick and it
// could never fire.
const dxRouteTransitionRouteRendered = () => new Promise((resolve) => {
    let settled = false;
    const finish = () => {
        if (settled) return;
        settled = true;
        window.clearTimeout(ceiling);
        observer?.disconnect?.();
        resolve();
    };
    const observer = typeof MutationObserver === "undefined" ? null : new MutationObserver(() => finish());
    // Only structural changes: an attribute tick from a clock or a progress bar
    // is not the route arriving.
    observer?.observe?.(document.body ?? document.documentElement, {
        childList: true,
        subtree: true,
    });
    const ceiling = window.setTimeout(() => finish(), 120);
    if (!observer) finish();
});

try {
    if (!document.startViewTransition || prefersReducedMotion) {
        dioxus.send("navigate");
        dioxus.send("done");
    } else {
        document.documentElement.dataset.routeTransition = animation;
        document.documentElement.dataset.routeTransitionPlatform = platform;
        if (from) document.documentElement.dataset.routeTransitionFrom = from;
        if (to) document.documentElement.dataset.routeTransitionTo = to;
        dxRouteTransitionPaintContext();

        // The outgoing snapshot is taken synchronously inside
        // startViewTransition, so the attributes set above have to reach
        // computed style before that call. Without this flush an element whose
        // `view-transition-name` is granted by those attributes is still
        // unnamed when it is captured, and so gets no group at all.
        //
        // The failure is asymmetric and easy to miss: an overlay entering needs
        // its name only in the *new* state, which is styled later anyway and
        // works, while the same overlay leaving needs it in the old state and
        // silently drops out of the transition.
        void document.documentElement.offsetHeight;

        const transition = document.startViewTransition(async () => {
            // Watch first, then ask. The router usually renders while the ack
            // is still in flight, so an observer started afterwards has already
            // missed the only mutation it cares about.
            const rendered = dxRouteTransitionRouteRendered();

            dioxus.send("navigate");

            const routeCommit = await dioxus.recv();
            if (routeCommit !== "navigated") {
                throw new Error(`unexpected route transition ack: ${routeCommit}`);
            }

            await rendered;
            // A browser Back or Forward returns to a remembered scroll offset.
            // Restored here, so the new snapshot is taken where the page lands.
            window[Symbol.for("g3-route-transitions.browser-history")]?.restoreScroll?.();
            // Dismissal reveals the newly rendered route, not the outgoing
            // sheet. Capture its resolved theme color before the new snapshot
            // is taken so dark pages do not inherit a light sheet backdrop.
            dxRouteTransitionIncomingPaintContext();
        });

        try {
            await transition.ready;
        } catch (_) {
        }

        try {
            await transition.finished;
        } catch (_) {
        }

        dioxus.send("done");
    }
} catch (_) {
    dioxus.send("fallback");
} finally {
    delete document.documentElement.dataset.routeTransition;
    delete document.documentElement.dataset.routeTransitionPlatform;
    delete document.documentElement.dataset.routeTransitionFrom;
    delete document.documentElement.dataset.routeTransitionTo;
    document.documentElement.style.removeProperty("--route-transition-surface-bg");
    document.documentElement.style.removeProperty("--route-transition-incoming-surface-bg");
    document.documentElement.style.removeProperty("--route-transition-document-bg");
    document.documentElement.style.removeProperty("--route-transition-clip-radius");
}
"#;
/// `value` as a double-quoted JavaScript string literal.
///
/// Route paths carry their parameters, which can hold quotes, backslashes or
/// line separators; any of those written raw would end the literal early.
fn js_string_literal(value: &str) -> String {
    let mut literal = String::with_capacity(value.len() + 2);
    literal.push('"');
    for character in value.chars() {
        match character {
            '"' => literal.push_str("\\\""),
            '\\' => literal.push_str("\\\\"),
            '\n' => literal.push_str("\\n"),
            '\r' => literal.push_str("\\r"),
            '\u{2028}' => literal.push_str("\\u2028"),
            '\u{2029}' => literal.push_str("\\u2029"),
            character if character.is_control() => {
                literal.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => literal.push(character),
        }
    }
    literal.push('"');
    literal
}
async fn run_animated_navigation(
    animation: NavigationTransition,
    from: &str,
    to: Option<&str>,
    mut navigate: impl FnMut(),
) {
    let script = VIEW_TRANSITION_NAVIGATE
        .replace(ANIMATION_PLACEHOLDER, animation.data_value())
        .replace(PLATFORM_PLACEHOLDER, get_platform().data_value())
        .replace(FROM_PLACEHOLDER, &js_string_literal(from))
        .replace(TO_PLACEHOLDER, &js_string_literal(to.unwrap_or_default()));
    let mut transition = eval(&script);
    let mut navigated = false;
    loop {
        match transition.recv::<String>().await.as_deref() {
            Ok("navigate") => {
                if !navigated {
                    navigate();
                    navigated = true;
                }
                _ = transition.send("navigated");
            }
            Ok("done") => break,
            Ok("fallback") | Err(_) => {
                if !navigated {
                    navigate();
                }
                break;
            }
            _ => {}
        }
    }
}
/// Navigate to `route` with the transition chosen by
/// `current.transition_to(&route)`, pushing or replacing history according to
/// `current.replaces_history(&route)`.
///
/// The old page is captured before the router changes, which is why this must
/// be used instead of `navigator().push(...)`. The call does nothing if
/// `route` equals the current route. It changes the route without animation
/// when the transition is [`NavigationTransition::None`], the browser lacks
/// View Transitions, or the user prefers reduced motion. The future resolves
/// once the animation has finished.
///
/// Call it from an event handler, for example
/// `onclick: move |_| async move { animated_navigate(Route::Home {}).await }`.
/// It must run beneath `Router::<Route>`.
pub async fn animated_navigate<Route>(route: Route)
where
    Route: Clone + ToString + RouteTransitions + Routable + 'static,
{
    let current_route = router().current::<Route>().clone();
    if current_route == route {
        return;
    }
    let animation = current_route.transition_to(&route);
    let replace = current_route.replaces_history(&route);
    let navigator = navigator();
    let from = current_route.to_string();
    let route = route.to_string();
    if animation == NavigationTransition::None {
        if replace {
            _ = navigator.replace(route);
        } else {
            _ = navigator.push(route);
        }
        return;
    }
    run_animated_navigation(animation, &from, Some(&route.clone()), || {
        if replace {
            _ = navigator.replace(route.clone());
        } else {
            _ = navigator.push(route.clone());
        }
    })
    .await;
}

/// Navigate with an explicit transition while preserving the route's history
/// replacement rule. This is for UI state encoded in a route (for example a
/// segmented filter) whose visual relationship is known by the caller even
/// when the route taxonomy deliberately treats the update as `None`.
pub async fn animated_navigate_with<Route>(route: Route, animation: NavigationTransition)
where
    Route: Clone + ToString + RouteTransitions + Routable + 'static,
{
    let current_route = router().current::<Route>().clone();
    if current_route == route {
        return;
    }
    let replace = current_route.replaces_history(&route);
    let navigator = navigator();
    let from = current_route.to_string();
    let route = route.to_string();
    if animation == NavigationTransition::None {
        if replace {
            _ = navigator.replace(route);
        } else {
            _ = navigator.push(route);
        }
        return;
    }
    run_animated_navigation(animation, &from, Some(&route.clone()), || {
        if replace {
            _ = navigator.replace(route.clone());
        } else {
            _ = navigator.push(route.clone());
        }
    })
    .await;
}
/// Pop real router history with `current.transition_back()`.
///
/// Calling `navigator().go_back()` directly changes the route before the old
/// page can be captured. This helper captures it first, then pops.
///
/// Returns `false` without doing anything when there is no previous entry, so
/// callers such as platform Back handlers can keep their own root behavior.
pub async fn try_animated_back<Route>() -> bool
where
    Route: Clone + ToString + RouteTransitions + Routable + 'static,
{
    let navigator = navigator();
    if !navigator.can_go_back() {
        return false;
    }
    let current_route = router().current::<Route>().clone();
    let animation = current_route.transition_back();
    if animation == NavigationTransition::None {
        navigator.go_back();
        return true;
    }
    let from = current_route.to_string();
    run_animated_navigation(animation, &from, None, || navigator.go_back()).await;
    true
}
/// Pop real router history like [`try_animated_back`], or, when there is no
/// previous entry (for example after a deep link), navigate to `fallback` with
/// [`animated_navigate`].
///
/// Use this for visible Back buttons. When falling back, the transition comes
/// from the forward rules (`current.transition_to(&fallback)`), so a stack
/// page returning to its root still animates `Backward`.
pub async fn animated_back_or_navigate<Route>(fallback: Route)
where
    Route: Clone + ToString + RouteTransitions + Routable + 'static,
{
    if !try_animated_back::<Route>().await {
        animated_navigate(fallback).await;
    }
}
#[cfg(feature = "native-back")]
/// Window event emitted after native Back has completed an animated router pop.
///
/// Apps normally do not need this. It is available for route-adjacent work
/// such as restoring scroll after the destination has rendered.
pub const NATIVE_BACK_TRANSITION_FINISHED_EVENT: &str = "g3routebacktransitionend";
#[cfg(all(feature = "native-back", any(target_os = "android", target_os = "ios")))]
const NATIVE_BACK_EVENT_PLACEHOLDER: &str = "__G3_NATIVE_BACK_EVENT__";
#[cfg(all(feature = "native-back", any(target_os = "android", target_os = "ios")))]
const NATIVE_BACK_FINISHED_EVENT_PLACEHOLDER: &str = "__G3_NATIVE_BACK_FINISHED_EVENT__";
#[cfg(all(
    feature = "native-back",
    any(target_os = "android", target_os = "ios", test)
))]
const NATIVE_BACK_NAVIGATION_BRIDGE: &str = r#"
const nativeBackEvent = "__G3_NATIVE_BACK_EVENT__";
const transitionFinishedEvent = "__G3_NATIVE_BACK_FINISHED_EVENT__";
const stateKey = Symbol.for("g3-route-transitions.native-back");

window[stateKey]?.dispose?.();

const state = { pending: false };
const defer = window.queueMicrotask?.bind(window)
    ?? ((callback) => Promise.resolve().then(callback));
const onNativeBack = (event) => {
    // Give dialogs, sheets, menus, and fullscreen players the rest of the
    // current dispatch to claim this cancelable event first.
    defer(async () => {
        if (event.defaultPrevented || state.pending) return;

        state.pending = true;
        event.preventDefault();
        dioxus.send("back");

        try {
            const outcome = await dioxus.recv();
            if (outcome === "navigated") {
                window.dispatchEvent(new Event(transitionFinishedEvent));
            }
        } finally {
            state.pending = false;
        }
    });
};

window.addEventListener(nativeBackEvent, onNativeBack);

// Native renderers close an eval's channel as soon as its script returns, after
// which Rust's replies never arrive: the first press would leave `pending` set
// and every later press would be swallowed. Stay alive until replaced.
let release;
const released = new Promise((resolve) => {
    release = resolve;
});
window[stateKey] = {
    dispose() {
        window.removeEventListener(nativeBackEvent, onNativeBack);
        release();
    },
};
await released;
"#;
/// Connect native Back from `g3-native-plugins` to [`try_animated_back`]:
/// Android's system Back gesture or key, and iOS's swipe in from the left
/// screen edge.
///
/// Call this hook once from a layout rendered beneath `Router<Route>`. It
/// subscribes to the current route, enables native interception only while the
/// router can go back, and uses the route's generated reverse animation. At a
/// root route interception is off, so Android Back exits normally and an iOS
/// edge swipe does nothing, as it would in any other app. On other targets the
/// hook does nothing; see [`use_browser_history_transitions`] for the web.
///
/// Enable the crate's `native-back` feature to use this hook. It reuses the
/// nearest `NativePluginsProvider` when present and otherwise owns a standalone
/// Back plugin, so no handwritten Rust/JavaScript connector is required.
///
/// Higher-priority UI may claim the cancelable `g3nativeback` window event by
/// synchronously calling `preventDefault()`. See
/// [`use_native_back_navigation_with_interception`] when such UI can be open
/// without router history.
#[cfg(feature = "native-back")]
pub fn use_native_back_navigation<Route>()
where
    Route: Clone + ToString + RouteTransitions + Routable + 'static,
{
    use_native_back_navigation_with_interception::<Route>(false);
}
/// The configurable form of [`use_native_back_navigation`].
///
/// Set `intercept_without_history` while a non-route UI layer is open at the
/// root. That layer must synchronously call `preventDefault()` on the
/// `g3nativeback` event after dismissing itself. If no layer claims the event
/// and no router history exists, the hook passes that press back to the
/// platform: Android handles it (usually by leaving the app), and iOS drops the
/// swipe.
#[cfg(feature = "native-back")]
pub fn use_native_back_navigation_with_interception<Route>(intercept_without_history: bool)
where
    Route: Clone + ToString + RouteTransitions + Routable + 'static,
{
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        use g3_native_plugins::{BackButton, NativePlugins};
        let _current_route: Route = use_route();
        let navigator = navigator();
        let standalone = use_signal(BackButton::new);
        let mut back_button = try_consume_context::<NativePlugins>()
            .map(|plugins| plugins.back_button)
            .unwrap_or(standalone);
        let intercepting = navigator.can_go_back() || intercept_without_history;
        {
            let mut back_button = back_button.write();
            let _ = back_button.prepare();
            let _ = back_button.set_intercepting(intercepting);
        }
        use_future(move || async move {
            let script = NATIVE_BACK_NAVIGATION_BRIDGE
                .replace(
                    NATIVE_BACK_EVENT_PLACEHOLDER,
                    g3_native_plugins::NATIVE_BACK_EVENT,
                )
                .replace(
                    NATIVE_BACK_FINISHED_EVENT_PLACEHOLDER,
                    NATIVE_BACK_TRANSITION_FINISHED_EVENT,
                );
            let mut bridge = eval(&script);
            loop {
                if !matches!(bridge.recv::<String>().await.as_deref(), Ok("back")) {
                    break;
                }
                let navigated = try_animated_back::<Route>().await;
                if !navigated {
                    let _ = back_button.write().fall_through();
                }
                let outcome = if navigated { "navigated" } else { "ignored" };
                if bridge.send(outcome).is_err() {
                    break;
                }
            }
        });
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let _ = intercept_without_history;
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sheet_transitions_dim_the_full_base_snapshot_per_platform() {
        let stylesheet = include_str!("../assets/route_transitions.css");
        assert!(stylesheet.contains("present-sheet\"]::view-transition-new(overlay)"));
        assert!(stylesheet.contains("dismiss-sheet\"]::view-transition-old(overlay)"));
        assert!(!stylesheet.contains("[data-route-transition-layer=\"sheet\"]"));
        assert!(stylesheet.contains("route-transition-ios-dim-base"));
        assert!(stylesheet.contains("route-transition-material-dim-base"));
        assert!(stylesheet.contains("filter: brightness"));
        assert!(stylesheet.contains("--route-transition-ios-presentation-backdrop"));
        assert!(stylesheet.contains("box-shadow: 0 -14px 20px -14px"));
        assert!(stylesheet.contains("present-sheet\"]::view-transition-old(overlay)"));
        assert!(stylesheet.contains("dismiss-sheet\"]::view-transition-new(overlay)"));
        assert!(stylesheet.contains("opacity: 0"));
    }
    #[test]
    fn sheet_transitions_are_scoped_by_platform_attribute() {
        let stylesheet = include_str!("../assets/route_transitions.css");
        assert!(
            stylesheet
                .contains(
                    "html[data-route-transition-platform=\"ios\"][data-route-transition=\"present-sheet\"]::view-transition-old(base)",
                ),
        );
        assert!(
            stylesheet
                .contains(
                    "html[data-route-transition-platform=\"material\"][data-route-transition=\"present-sheet\"]::view-transition-old(base)",
                ),
        );
        assert!(!stylesheet.contains("route-transition-mobile-dim-base"));
        assert!(!stylesheet.contains("route-transition-mobile-undim-base"));
        assert!(stylesheet.contains("border-radius: 12px"));
    }
    #[test]
    fn sheet_transitions_do_not_apply_debug_offsets_to_snapshots() {
        let stylesheet = include_str!("../assets/route_transitions.css");
        assert!(!stylesheet.contains("--route-transition-debug-peek"));
        assert!(!stylesheet.contains("--route-transition-cover-x"));
        assert!(!stylesheet.contains("--route-transition-base-x"));
        assert!(stylesheet.contains("transform: translateY(100%)"));
        assert!(stylesheet.contains("transform: translateY(0)"));
        assert!(!stylesheet.contains("translateX(var(--route-transition-base-x))"));
    }
    #[test]
    fn sheet_transitions_force_active_snapshots_to_paint_above_base() {
        let stylesheet = include_str!("../assets/route_transitions.css");
        assert!(stylesheet.contains("present-sheet\"]::view-transition-new(overlay),"));
        assert!(stylesheet.contains("dismiss-sheet\"]::view-transition-old(overlay)"));
        assert!(stylesheet.contains("mix-blend-mode: normal"));
        assert!(stylesheet.contains("opacity: 1"));
        assert!(stylesheet.contains("z-index: 2"));
        assert!(stylesheet.contains("present-sheet\"]::view-transition-old(base),"));
        assert!(stylesheet.contains("dismiss-sheet\"]::view-transition-new(base)"));
        assert!(stylesheet.contains("z-index: 1"));
    }
    #[test]
    fn hidden_base_pair_cannot_cover_a_full_page_snapshot() {
        let stylesheet = include_str!("../assets/route_transitions.css");
        let base_group = stylesheet
            .split("html[data-route-transition=\"present-sheet\"]::view-transition-group(base),")
            .nth(2)
            .and_then(|block| block.split('}').next())
            .expect("missing sheet base stacking rule");
        let page_group = stylesheet
            .split("html[data-route-transition=\"present-sheet\"]::view-transition-group(page),")
            .nth(2)
            .and_then(|block| block.split('}').next())
            .expect("missing sheet page stacking rule");

        assert!(base_group.contains("z-index: 0"));
        assert!(page_group.contains("z-index: 1"));
    }
    #[test]
    fn snapshot_marker_components_are_public_contract() {
        let source = include_str!("lib.rs");
        let production_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes tests");
        assert!(production_source.contains("pub const ROUTE_TRANSITION_BASE_REGION_CLASS"));
        assert!(production_source.contains("pub const ROUTE_TRANSITION_OVERLAY_REGION_CLASS"));
        assert!(production_source.contains("pub const ROUTE_TRANSITION_SEGMENT_CLASS"));
        assert!(production_source.contains("pub const ROUTE_TRANSITION_PAGE_CLASS"));
        assert!(production_source.contains("pub fn RouteTransitionBaseRegion"));
        assert!(production_source.contains("pub fn RouteTransitionOverlayRegion"));
        assert!(production_source.contains("pub fn RouteTransitionSegment"));
        assert!(production_source.contains("pub fn RouteTransitionPage"));
        assert!(production_source.contains("pub const ROUTE_TRANSITION_PERSISTENT_CLASS"));
        assert!(production_source.contains("pub fn RouteTransitionPersistent"));
        assert!(production_source.contains("merge_transition_class"));
    }
    /// Persistent chrome never moves, stays above page pushes but below routed
    /// sheets, and fades instead of popping when only one route renders it.
    #[test]
    fn persistent_chrome_is_static_and_sheets_cover_it() {
        let stylesheet = include_str!("../assets/route_transitions.css");
        let rule = |selector: &str| {
            stylesheet
                .split(&format!("{selector} {{"))
                .nth(1)
                .and_then(|block| block.split('}').next())
                .unwrap_or_else(|| panic!("missing {selector}"))
                .to_string()
        };
        assert!(rule(".route-transition-persistent").contains("view-transition-name: persistent"));
        let group = rule("html[data-route-transition]::view-transition-group(persistent)");
        assert!(group.contains("animation: none"));
        assert!(group.contains("z-index: 10000"));
        for transition in ["present-sheet", "dismiss-sheet"] {
            let selector = format!(
                "html[data-route-transition=\"{transition}\"]::view-transition-group(persistent)"
            );
            assert!(rule(&selector).contains("z-index: 9998"));
        }
        assert!(
            rule("html[data-route-transition]::view-transition-old(persistent):only-child")
                .contains("route-transition-fade-out")
        );
        assert!(
            rule("html[data-route-transition]::view-transition-new(persistent):only-child")
                .contains("route-transition-persistent-fade-in")
        );
        assert!(
            rule("html[data-route-transition]::view-transition-old(persistent):not(:only-child)")
                .contains("opacity: 0")
        );
    }
    #[test]
    fn full_page_snapshots_own_nested_stack_motion() {
        let stylesheet = include_str!("../assets/route_transitions.css");
        assert!(stylesheet.contains(".route-transition-page"));
        assert!(stylesheet.contains("view-transition-name: page"));
        assert!(stylesheet.contains(".route-transition-page > .route-transition-base-region"));
        assert!(stylesheet.contains(".route-transition-page .route-transition-segment"));
        assert!(stylesheet.contains("view-transition-old(page)"));
        assert!(stylesheet.contains("view-transition-new(page)"));
    }
    #[test]
    fn route_transition_app_wraps_styles_and_overlay_region() {
        let source = include_str!("lib.rs");
        assert!(source.contains("pub fn RouteTransitionApp"));
        assert!(source.contains("RouteTransitionStyles"));
        assert!(source.contains("ROUTE_TRANSITION_OVERLAY_REGION_CLASS"));
        assert!(source.contains("merge_transition_class(ROUTE_TRANSITION_OVERLAY_REGION_CLASS"));
        assert!(source.contains("div { class, {children} }"));
    }
    #[test]
    fn style_is_flushed_before_the_outgoing_snapshot_is_taken() {
        let script = VIEW_TRANSITION_NAVIGATE;
        let attributes = script
            .find("dataset.routeTransitionPlatform = platform")
            .expect("platform attribute is set");
        let flush = script
            .find("void document.documentElement.offsetHeight")
            .expect("style is flushed");
        let capture = script
            .find("document.startViewTransition(async () =>")
            .expect("transition is started");
        assert!(
            attributes < flush,
            "the flush must come after the attributes"
        );
        assert!(flush < capture, "the flush must come before the snapshot");
    }
    #[test]
    fn runtime_exports_the_shell_paint_context_to_the_snapshot_tree() {
        let script = VIEW_TRANSITION_NAVIGATE;
        let paint_context = script
            .find("dxRouteTransitionPaintContext();")
            .expect("paint context is captured");
        let capture = script
            .find("document.startViewTransition(async () =>")
            .expect("transition is started");

        assert!(
            paint_context < capture,
            "paint context must precede capture"
        );
        assert!(script.contains(".route-transition-page, .route-transition-base-region"));
        assert!(script.contains("--route-transition-surface-bg"));
        assert!(script.contains("--route-transition-incoming-surface-bg"));
        assert!(script.contains("--route-transition-document-bg"));
        assert!(script.contains("--route-transition-clip-radius"));
        assert!(script.contains("style.removeProperty(\"--route-transition-surface-bg\")"));
        assert!(
            script.contains("style.removeProperty(\"--route-transition-incoming-surface-bg\")"),
        );
    }
    #[test]
    fn runtime_captures_the_incoming_surface_before_the_new_snapshot() {
        let script = VIEW_TRANSITION_NAVIGATE;
        let callback = script
            .find("document.startViewTransition(async () =>")
            .expect("the transition is started");
        let body = &script[callback..];
        let rendered = body.find("await rendered;").expect("the new route renders");
        let incoming = body
            .find("dxRouteTransitionIncomingPaintContext();")
            .expect("the incoming paint context is captured");

        assert!(
            rendered < incoming,
            "the incoming color must be read from the newly rendered route",
        );
    }
    #[test]
    fn spatial_snapshot_groups_clip_to_embedded_shells() {
        let stylesheet = include_str!("../assets/route_transitions.css");

        for selector in [
            "[data-route-transition=\"forward\"]::view-transition-group(page)",
            "[data-route-transition=\"backward\"]::view-transition-group(page)",
            "[data-route-transition=\"forward\"]::view-transition-image-pair(page)",
            "[data-route-transition=\"backward\"]::view-transition-image-pair(page)",
            "[data-route-transition=\"present-sheet\"]::view-transition-group(overlay)",
            "[data-route-transition=\"dismiss-sheet\"]::view-transition-group(overlay)",
            "[data-route-transition=\"present-sheet\"]::view-transition-image-pair(overlay)",
            "[data-route-transition=\"dismiss-sheet\"]::view-transition-image-pair(overlay)",
        ] {
            assert!(stylesheet.contains(selector), "missing {selector}");
        }
        assert!(stylesheet.contains("overflow: hidden;"));
        assert!(stylesheet.contains("border-radius: var(--route-transition-clip-radius, 0px)"));
        assert!(
            stylesheet
                .contains("clip-path: inset(0 round var(--route-transition-clip-radius, 0px))",)
        );
    }
    #[test]
    fn sheet_backdrop_uses_the_resolved_theme_surface() {
        let stylesheet = include_str!("../assets/route_transitions.css");

        assert!(stylesheet.contains("--route-transition-document-bg"));
        assert!(stylesheet.contains("--route-transition-surface-bg"));
        assert!(stylesheet.contains("--route-transition-incoming-surface-bg"));
        assert!(stylesheet.contains(
            "[data-route-transition=\"present-sheet\"]::view-transition-image-pair(page)",
        ));
        assert!(stylesheet.contains(
            "[data-route-transition=\"dismiss-sheet\"]::view-transition-image-pair(base)",
        ));
    }
    #[test]
    fn view_transition_update_waits_for_native_route_commit_without_blocking_on_raf() {
        assert!(VIEW_TRANSITION_NAVIGATE.contains("document.startViewTransition(async () =>"),);
        assert!(VIEW_TRANSITION_NAVIGATE.contains("dioxus.send(\"navigate\")"));
        assert!(VIEW_TRANSITION_NAVIGATE.contains("const routeCommit = await dioxus.recv()"),);
        assert!(VIEW_TRANSITION_NAVIGATE.contains("routeCommit !== \"navigated\""));
        let callback = VIEW_TRANSITION_NAVIGATE
            .find("document.startViewTransition(async () =>")
            .expect("the transition is started");
        let body = &VIEW_TRANSITION_NAVIGATE[callback..];
        let armed = body
            .find("const rendered = dxRouteTransitionRouteRendered();")
            .expect("the wait is armed");
        let asked = body
            .find("dioxus.send(\"navigate\")")
            .expect("the route is asked for");
        assert!(
            armed < asked,
            "the observer must be live before the request"
        );
        assert!(VIEW_TRANSITION_NAVIGATE.contains("await rendered;"));
        assert!(!VIEW_TRANSITION_NAVIGATE.contains("dxRouteTransitionNextFrame"));
        assert!(VIEW_TRANSITION_NAVIGATE.contains("await transition.ready"));
        assert!(!VIEW_TRANSITION_NAVIGATE.contains("console.info"));
        assert!(!VIEW_TRANSITION_NAVIGATE.contains("g3RouteTransition"));
    }
    #[test]
    fn transitions_publish_the_routes_they_move_between_and_clear_them() {
        assert!(VIEW_TRANSITION_NAVIGATE.contains(FROM_PLACEHOLDER));
        assert!(VIEW_TRANSITION_NAVIGATE.contains(TO_PLACEHOLDER));
        // Set before the outgoing snapshot is taken, like the animation itself,
        // so CSS keyed on them names elements in the old state too.
        let set_from = VIEW_TRANSITION_NAVIGATE
            .find("dataset.routeTransitionFrom = from")
            .expect("from is published");
        let started = VIEW_TRANSITION_NAVIGATE
            .find("document.startViewTransition(async () =>")
            .expect("the transition is started");
        assert!(set_from < started);
        assert!(VIEW_TRANSITION_NAVIGATE.contains("dataset.routeTransitionTo = to"));
        let cleanup = VIEW_TRANSITION_NAVIGATE
            .split("} finally {")
            .nth(1)
            .expect("the script cleans up");
        assert!(cleanup.contains("delete document.documentElement.dataset.routeTransitionFrom;"));
        assert!(cleanup.contains("delete document.documentElement.dataset.routeTransitionTo;"));
    }
    #[test]
    fn route_paths_are_escaped_into_string_literals() {
        assert_eq!(js_string_literal("/watch/abc"), "\"/watch/abc\"");
        assert_eq!(js_string_literal("a\"b\\c"), "\"a\\\"b\\\\c\"");
        assert_eq!(js_string_literal("a\nb\u{2028}"), "\"a\\nb\\u2028\"");
        assert_eq!(js_string_literal(""), "\"\"");
    }
    #[test]
    fn rust_side_acknowledges_navigation_after_router_push() {
        let source = include_str!("lib.rs");
        let production_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes tests");
        assert!(production_source.contains("Ok(\"navigate\")"));
        assert!(production_source.contains("transition.send(\"navigated\")"));
        assert!(!production_source.contains("eprintln!"));
    }
    #[test]
    fn history_back_uses_the_view_transition_handshake() {
        let source = include_str!("lib.rs");
        let production_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes tests");
        let helper = production_source
            .split("pub async fn try_animated_back")
            .nth(1)
            .expect("try_animated_back is public");
        assert!(helper.contains("navigator.can_go_back()"));
        assert!(helper.contains("current_route.transition_back()"));
        assert!(helper.contains("run_animated_navigation(animation"));
        assert!(helper.contains("navigator.go_back()"));
        let fallback_helper = production_source
            .split("pub async fn animated_back_or_navigate")
            .nth(1)
            .expect("animated_back_or_navigate is public");
        assert!(fallback_helper.contains("try_animated_back::<Route>().await"));
        assert!(fallback_helper.contains("animated_navigate(fallback).await"));
    }
    #[cfg(feature = "native-back")]
    #[test]
    fn native_back_bridge_is_cancelable_prioritized_and_acknowledged() {
        let source = include_str!("lib.rs");
        assert!(NATIVE_BACK_NAVIGATION_BRIDGE.contains("event.defaultPrevented"));
        assert!(NATIVE_BACK_NAVIGATION_BRIDGE.contains("queueMicrotask"));
        assert!(NATIVE_BACK_NAVIGATION_BRIDGE.contains("dioxus.send(\"back\")"));
        assert!(NATIVE_BACK_NAVIGATION_BRIDGE.contains("await dioxus.recv()"));
        assert!(NATIVE_BACK_NAVIGATION_BRIDGE.contains("g3-route-transitions.native-back"),);
        assert_eq!(
            NATIVE_BACK_TRANSITION_FINISHED_EVENT,
            "g3routebacktransitionend"
        );
        assert!(source.contains("back_button.write().fall_through()"));
    }
    /// Native renderers close an eval channel when its script returns. A
    /// bridge that only registers a listener and returns would never hear
    /// Rust's reply, leaving the first press pending and swallowing the rest.
    #[cfg(feature = "native-back")]
    #[test]
    fn native_back_bridge_stays_alive_to_receive_replies() {
        let bridge = NATIVE_BACK_NAVIGATION_BRIDGE.trim_end();
        assert!(bridge.ends_with("await released;"));
        let dispose = bridge
            .split("dispose() {")
            .nth(1)
            .expect("the bridge can be disposed");
        assert!(dispose.contains("release();"));
        assert!(bridge.contains("state.pending = false"));
    }
    /// iOS raises the same `g3nativeback` event from its edge swipe, so the
    /// bridge must be compiled for both native platforms, not only Android.
    #[test]
    fn native_back_is_wired_on_android_and_ios() {
        let source = include_str!("lib.rs");
        let production_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes tests");
        let hook = production_source
            .split("pub fn use_native_back_navigation_with_interception")
            .nth(1)
            .expect("the configurable hook is public");
        assert!(hook.contains("#[cfg(any(target_os = \"android\", target_os = \"ios\"))]"));
        assert!(hook.contains("#[cfg(not(any(target_os = \"android\", target_os = \"ios\")))]"));
        assert!(!production_source.contains("native-back\", target_os = \"android\")"));
    }
    #[test]
    fn in_place_routes_replace_history_without_crossing_the_js_boundary() {
        let source = include_str!("lib.rs");
        let production_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source precedes tests");
        let helper = production_source
            .split("pub async fn animated_navigate")
            .nth(1)
            .expect("animated_navigate is public");
        assert!(helper.contains("current_route.replaces_history(&route)"));
        assert!(helper.contains("navigator.replace(route)"));
        assert!(helper.contains("animation == NavigationTransition::None"));
    }
    #[test]
    fn a_cross_dissolve_never_uncovers_what_is_behind_the_page() {
        let stylesheet = include_str!("../assets/route_transitions.css");
        assert!(
            !stylesheet.contains("route-transition-fade-in"),
            "nothing may fade in during a cross-dissolve",
        );
        assert!(
            stylesheet.contains("route-transition-fade-out"),
            "the outgoing page still fades out",
        );
    }
    #[test]
    fn route_transition_css_uses_production_durations() {
        let stylesheet = include_str!("../assets/route_transitions.css");
        assert!(stylesheet.contains("--route-transition-sheet-duration: 0.6s"));
        assert!(stylesheet.contains("--route-transition-stack-duration: 260ms"));
        assert!(stylesheet.contains("--route-transition-stack-duration: 300ms"));
        assert!(stylesheet.contains("--route-transition-fade-duration: 150ms"));
        assert!(stylesheet.contains("--route-transition-sheet-duration: 400ms"));
        assert!(stylesheet.contains("--route-transition-material-sheet-dismiss-duration: 350ms"),);
        assert!(stylesheet.contains("translateY(20%); opacity: 0"));
        assert!(stylesheet.contains("to { transform: translateY(100%); }"));
        assert!(!stylesheet.contains("route-transition-duration-debug"));
        assert!(!stylesheet.contains("route-transition-mobile-dim-base"));
        assert!(!stylesheet.contains("route-transition-mobile-undim-base"));
        assert!(!stylesheet.contains("morph"));
        assert!(!stylesheet.contains("zoom"));
    }
    #[test]
    fn stack_transitions_diverge_between_ios_parallax_and_material_shared_axis() {
        let stylesheet = include_str!("../assets/route_transitions.css");
        assert!(stylesheet.contains("route-transition-ios-push-out-left"));
        assert!(stylesheet.contains("translateX(-30%); filter: brightness(0.85)"));
        assert!(stylesheet.contains("route-transition-material-axis-out-left"));
        assert!(stylesheet.contains("route-transition-material-axis-in-left"));
        assert!(stylesheet.contains("route-transition-material-axis-out-right"));
        assert!(stylesheet.contains("route-transition-material-axis-in-right"));
        assert!(stylesheet.contains("--route-transition-material-spatial-ease"));
        assert!(stylesheet.contains("--route-transition-material-shared-axis-distance: 30px"));
        assert!(stylesheet.contains("35%, 100% { opacity: 0; }"));
        assert!(stylesheet.contains("box-shadow: -16px 0 16px -16px"));
    }

    /// A segment push is a filmstrip, not a page push. The two tab bodies are
    /// neighbouring frames of one strip: they travel the full width on one
    /// shared curve, so the gap between them stays exactly one pane wide for
    /// the whole transition and neither can appear on top of the other. The
    /// platform-specific parallax and shared-axis pairs stay on `page`, where
    /// one surface really is moving over another.
    #[test]
    fn peer_segments_slide_as_one_filmstrip_rather_than_a_stack_page() {
        let stylesheet = include_str!("../assets/route_transitions.css");

        assert!(stylesheet.contains("route-transition-segment-out-left"));
        assert!(stylesheet.contains("route-transition-segment-in-left"));
        assert!(stylesheet.contains("route-transition-segment-out-right"));
        assert!(stylesheet.contains("route-transition-segment-in-right"));

        // Keyframe bodies contain nested braces, so the block ends at the first
        // closing brace in column zero rather than the first one seen.
        let keyframe_block = |name: &str| {
            stylesheet
                .split(&format!("@keyframes {name} {{"))
                .nth(1)
                .and_then(|block| block.split("\n}").next())
                .unwrap_or_else(|| panic!("missing @keyframes {name}"))
                .to_string()
        };

        // Full-width travel in both directions, so the outgoing pane clears the
        // box exactly as the incoming one lands. A partial offset (the 30% a
        // page push uses) leaves the two panes overlapping mid-transition.
        for (name, from, to) in [
            (
                "route-transition-segment-out-left",
                "translateX(0)",
                "translateX(-100%)",
            ),
            (
                "route-transition-segment-in-left",
                "translateX(100%)",
                "translateX(0)",
            ),
            (
                "route-transition-segment-out-right",
                "translateX(0)",
                "translateX(100%)",
            ),
            (
                "route-transition-segment-in-right",
                "translateX(-100%)",
                "translateX(0)",
            ),
        ] {
            let block = keyframe_block(name);
            assert!(block.contains(from), "{name} should start at {from}");
            assert!(block.contains(to), "{name} should end at {to}");
            // No cross-fade: a filmstrip never dims or dissolves between frames.
            assert!(!block.contains("opacity"), "{name} should not fade");
            assert!(!block.contains("brightness"), "{name} should not dim");
        }

        // One shared curve for both panes, rather than the accelerate /
        // decelerate pair that makes a page push read as two separate moves.
        assert!(stylesheet.contains("--route-transition-segment-ease"));
        assert!(!stylesheet.contains(
            "\"material\"][data-route-transition=\"forward\"]::view-transition-old(segment)"
        ));

        // The user agent gives view-transition images `plus-lighter` for smooth
        // cross-fades; two opaque panes sliding across each other must
        // composite normally or the overlap ghosts.
        let segment_images = stylesheet
            .split("::view-transition-new(segment) {")
            .nth(1)
            .and_then(|block| block.split('}').next())
            .expect("missing segment image rule");
        assert!(segment_images.contains("mix-blend-mode: normal"));

        // Full-width travel has to be clipped to the segment's own box.
        assert!(stylesheet.contains("::view-transition-group(segment)"));
        assert!(stylesheet.contains("::view-transition-image-pair(segment)"));
        assert!(stylesheet.contains("clip-path: inset(0)"));
    }
    #[test]
    fn cross_fade_transitions_are_the_same_on_both_platforms() {
        let stylesheet = include_str!("../assets/route_transitions.css");
        assert!(
            stylesheet
                .contains("html[data-route-transition=\"cross-fade\"]::view-transition-old(root)",),
        );
        assert!(stylesheet.contains("route-transition-fade-out"));
        assert!(!stylesheet.contains("route-transition-material-fade-through"));
    }
    #[test]
    fn animation_data_values_match_css_contract() {
        assert_eq!(NavigationTransition::None.data_value(), "none");
        assert_eq!(NavigationTransition::CrossFade.data_value(), "cross-fade");
        assert_eq!(NavigationTransition::Forward.data_value(), "forward");
        assert_eq!(NavigationTransition::Backward.data_value(), "backward");
        assert_eq!(
            NavigationTransition::PresentSheet.data_value(),
            "present-sheet"
        );
        assert_eq!(
            NavigationTransition::DismissSheet.data_value(),
            "dismiss-sheet"
        );
    }
    #[test]
    fn platform_data_values_match_css_contract() {
        assert_eq!(Platform::Ios.data_value(), "ios");
        assert_eq!(Platform::Material.data_value(), "material");
    }
    #[test]
    fn set_platform_overrides_auto_detection() {
        set_platform(Platform::Ios);
        assert_eq!(get_platform(), Platform::Ios);
        set_platform(Platform::Material);
        assert_eq!(get_platform(), Platform::Material);
    }
    #[test]
    fn animation_and_platform_are_written_in_rather_than_awaited() {
        let start = VIEW_TRANSITION_NAVIGATE
            .find("document.startViewTransition")
            .expect("the transition is started");
        assert!(
            !VIEW_TRANSITION_NAVIGATE[..start].contains("await dioxus.recv()"),
            "no round trip may precede the snapshot",
        );
        let filled = VIEW_TRANSITION_NAVIGATE
            .replace(
                ANIMATION_PLACEHOLDER,
                NavigationTransition::PresentSheet.data_value(),
            )
            .replace(PLATFORM_PLACEHOLDER, Platform::Ios.data_value())
            .replace(FROM_PLACEHOLDER, &js_string_literal("/queue"))
            .replace(TO_PLACEHOLDER, &js_string_literal("/watch/abc"));
        assert!(filled.contains(r#"const animation = "present-sheet";"#));
        assert!(filled.contains(r#"const platform = "ios";"#));
        assert!(filled.contains(r#"const from = "/queue";"#));
        assert!(filled.contains(r#"const to = "/watch/abc";"#));
        assert!(
            !filled.contains("__DX_ROUTE_TRANSITION"),
            "every placeholder is substituted",
        );
        assert!(
            filled
                .contains("document.documentElement.dataset.routeTransitionPlatform = platform;",),
        );
        assert!(!filled.contains("navigator.userAgent"));
    }
    /// The documented CSS contract: every animated transition value and every
    /// snapshot name is used, and the pre-0.4 spellings are gone.
    #[test]
    fn stylesheet_keys_on_the_documented_values_and_snapshot_names() {
        let stylesheet = include_str!("../assets/route_transitions.css");
        for transition in [
            NavigationTransition::CrossFade,
            NavigationTransition::Forward,
            NavigationTransition::Backward,
            NavigationTransition::PresentSheet,
            NavigationTransition::DismissSheet,
        ] {
            let selector = format!("[data-route-transition=\"{}\"]", transition.data_value());
            assert!(stylesheet.contains(&selector), "missing {selector}");
        }
        for name in ["root", "base", "overlay", "page", "segment", "persistent"] {
            assert!(
                stylesheet.contains(&format!("::view-transition-old({name})")),
                "missing snapshot {name}",
            );
        }
        for legacy in [
            "\"push-left\"",
            "\"push-right\"",
            "\"fade\"",
            "(cover)",
            "-md-",
        ] {
            assert!(!stylesheet.contains(legacy), "legacy {legacy} remains");
        }
    }
    #[test]
    fn cross_fade_holds_the_incoming_base_region_opaque_like_the_root() {
        let stylesheet = include_str!("../assets/route_transitions.css");
        let outgoing = stylesheet
            .split("html[data-route-transition=\"cross-fade\"]::view-transition-old(base) {")
            .nth(1)
            .and_then(|block| block.split('}').next())
            .expect("missing outgoing base fade rule");
        assert!(outgoing.contains("route-transition-fade-out"));
        let incoming = stylesheet
            .split("html[data-route-transition=\"cross-fade\"]::view-transition-new(base) {")
            .last()
            .and_then(|block| block.split('}').next())
            .expect("missing incoming base fade rule");
        assert!(incoming.contains("animation: none"));
        assert!(incoming.contains("opacity: 1"));
    }
    /// The helpers run inside spawned futures, where hooks must not be called.
    #[test]
    fn navigation_helpers_do_not_call_hooks() {
        let source = include_str!("lib.rs");
        for helper in [
            "pub async fn animated_navigate",
            "pub async fn try_animated_back",
        ] {
            let body = source
                .split(helper)
                .nth(1)
                .and_then(|rest| rest.split("\n}\n").next())
                .expect("helper is present");
            assert!(!body.contains("use_navigator"), "{helper} calls a hook");
            assert!(body.contains("navigator()"));
        }
    }
}
