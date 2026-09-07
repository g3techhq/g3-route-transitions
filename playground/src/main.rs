use dioxus::prelude::*;
use g3_route_transitions::{
    Platform, RouteTransitionBase, RouteTransitionCover, RouteTransitionPage,
    RouteTransitionProvider, animated_navigate, route_transitions, set_platform,
};
use gloo_timers::future::TimeoutFuture;
use manganis::{AssetOptions, asset};

#[allow(dead_code)]
const PLAYGROUND_CSS: Asset = asset!(
    "/assets/playground.css",
    AssetOptions::css().with_static_head(true)
);

#[route_transitions]
#[derive(Clone, Debug, PartialEq, Routable)]
#[rustfmt::skip]
enum Route {
    #[layout(DemoShell)]
        #[transition(root)]
        #[route("/")]
        Home {},

        #[transition(root)]
        #[route("/discover")]
        Discover {},

        #[transition(pushed)]
        #[route("/detail")]
        Detail {},

        #[transition(cover)]
        #[route("/queue")]
        QueueSheet {},

        #[transition(base)]
        #[route("/gallery")]
        Gallery {},

        #[transition(morph)]
        #[route("/gallery/story")]
        Story {},
}

fn main() {
    set_platform(Platform::Ios);
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Link { rel: "preconnect", href: "https://fonts.gstatic.com", crossorigin: "anonymous" }
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=DM+Sans:wght@400;500;600;700&family=Manrope:wght@500;600;700&display=swap",
        }
        Router::<Route> {}
    }
}

#[component]
fn DemoShell() -> Element {
    let mut platform = use_signal(|| Platform::Ios);
    let mut playing = use_signal(|| false);
    let current_route: Route = use_route();
    set_platform(platform());

    let play_all = move |_| {
        if playing() {
            return;
        }
        playing.set(true);
        spawn(async move {
            animated_navigate(Route::Home {}).await;
            pause(420).await;
            animated_navigate(Route::Detail {}).await;
            pause(520).await;
            animated_navigate(Route::Home {}).await;
            pause(420).await;
            animated_navigate(Route::QueueSheet {}).await;
            pause(520).await;
            animated_navigate(Route::Home {}).await;
            pause(420).await;
            animated_navigate(Route::Discover {}).await;
            pause(420).await;
            animated_navigate(Route::Gallery {}).await;
            pause(420).await;
            animated_navigate(Route::Story {}).await;
            pause(520).await;
            animated_navigate(Route::Gallery {}).await;
            playing.set(false);
        });
    };

    rsx! {
        div {
            class: "studio",
            "data-platform": platform().data_value(),
            aside { class: "control-panel",
                div { class: "brand-lockup",
                    div { class: "brand-mark", "g3" }
                    div {
                        p { class: "eyebrow", "Dioxus motion toolkit" }
                        h1 { "Route transitions" }
                    }
                }
                p { class: "intro",
                    "Route-owned navigation that feels native on the web, iOS, and Android."
                }

                div { class: "control-group",
                    span { class: "control-label", "Motion language" }
                    div { class: "platform-switch", role: "group", aria_label: "Motion language",
                        button {
                            class: if platform() == Platform::Ios { "platform-option active" } else { "platform-option" },
                            aria_pressed: platform() == Platform::Ios,
                            onclick: move |_| platform.set(Platform::Ios),
                            "iOS"
                        }
                        button {
                            class: if platform() == Platform::Md { "platform-option active" } else { "platform-option" },
                            aria_pressed: platform() == Platform::Md,
                            onclick: move |_| platform.set(Platform::Md),
                            "Material"
                        }
                    }
                }

                div { class: "control-group transition-list",
                    span { class: "control-label", "Transitions in this tour" }
                    TransitionKey { swatch: "violet", title: "Push / pop", detail: "Hierarchy" }
                    TransitionKey { swatch: "coral", title: "Cover / uncover", detail: "Sheet" }
                    TransitionKey { swatch: "aqua", title: "Cross-fade", detail: "Peers" }
                    TransitionKey { swatch: "lime", title: "Morph", detail: "Card detail" }
                }

                button {
                    class: "play-all",
                    disabled: playing(),
                    aria_label: "Play every route transition",
                    onclick: play_all,
                    span { class: "play-icon", aria_hidden: "true", if playing() { "•••" } else { "▶" } }
                    if playing() { "Playing tour" } else { "Play all" }
                }
                p { class: "hint", "Tip: record at 1440 × 900 for a clean GitHub preview." }
            }

            main { class: "stage",
                div { class: "stage-glow stage-glow-one" }
                div { class: "stage-glow stage-glow-two" }
                div { class: "device-wrap",
                    div { class: "device",
                        div { class: "device-hardware",
                            span { class: "sensor" }
                            span { class: "speaker" }
                        }
                        div { class: "device-screen",
                            RouteTransitionProvider {
                                Outlet::<Route> {}
                            }
                        }
                    }
                    div { class: "now-showing",
                        span { class: "live-dot" }
                        span { "Now showing" }
                        strong { "{transition_name(&current_route)}" }
                    }
                }
            }
        }
    }
}

#[component]
fn TransitionKey(swatch: &'static str, title: &'static str, detail: &'static str) -> Element {
    rsx! {
        div { class: "transition-key",
            span { class: "key-swatch {swatch}" }
            span { class: "key-title", "{title}" }
            span { class: "key-detail", "{detail}" }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        RouteTransitionPage { class: "app-page".to_string(),
            RouteTransitionBase { class: "app-surface".to_string(),
                PhoneHeader { eyebrow: "Saturday, September 6", title: "Good evening" }
                section { class: "hero-card",
                    div { class: "hero-copy",
                        span { class: "pill", "FEATURED" }
                        h2 { "The Quiet Orbit" }
                        p { "A human story at the edge of everything." }
                        button {
                            class: "primary-action",
                            onclick: move |_| async move { animated_navigate(Route::Detail {}).await },
                            "View details"
                            span { "→" }
                        }
                    }
                    div { class: "planet" }
                    div { class: "orbit" }
                }
                section { class: "section-row",
                    div {
                        p { class: "section-kicker", "CURATED FOR YOU" }
                        h3 { "Tonight's picks" }
                    }
                    button {
                        class: "text-button",
                        onclick: move |_| async move { animated_navigate(Route::Gallery {}).await },
                        "Explore"
                    }
                }
                div { class: "poster-row",
                    MiniPoster { tone: "sunset", title: "Afterglow", meta: "Drama · 2026" }
                    MiniPoster { tone: "ocean", title: "Blue Static", meta: "Mystery · 2025" }
                }
                BottomNav { active: "home" }
                button {
                    class: "floating-queue",
                    aria_label: "Open queue sheet",
                    onclick: move |_| async move { animated_navigate(Route::QueueSheet {}).await },
                    span { "＋" }
                }
            }
        }
    }
}

#[component]
fn Discover() -> Element {
    rsx! {
        RouteTransitionPage { class: "app-page".to_string(),
            RouteTransitionBase { class: "app-surface discover-page".to_string(),
                PhoneHeader { eyebrow: "FIND SOMETHING NEW", title: "Discover" }
                label { class: "search-box",
                    span { "⌕" }
                    input { aria_label: "Search", placeholder: "Films, shows, people" }
                }
                div { class: "genre-row",
                    span { class: "genre active", "All" }
                    span { class: "genre", "Drama" }
                    span { class: "genre", "Sci-fi" }
                    span { class: "genre", "Comedy" }
                }
                div { class: "feature-grid",
                    DiscoverCard { tone: "violet", number: "01", title: "Parallel Lines" }
                    DiscoverCard { tone: "orange", number: "02", title: "Summer, Again" }
                    DiscoverCard { tone: "green", number: "03", title: "Wild Signal" }
                    DiscoverCard { tone: "blue", number: "04", title: "Northbound" }
                }
                BottomNav { active: "discover" }
            }
        }
    }
}

#[component]
fn Detail() -> Element {
    rsx! {
        RouteTransitionPage { class: "app-page".to_string(),
            RouteTransitionBase { class: "app-surface detail-page".to_string(),
                div { class: "detail-art",
                    div { class: "detail-planet" }
                    button {
                        class: "round-back",
                        aria_label: "Back to home",
                        onclick: move |_| async move { animated_navigate(Route::Home {}).await },
                        "‹"
                    }
                    span { class: "detail-score", "8.9" }
                }
                div { class: "detail-body",
                    span { class: "pill dark", "SCIENCE FICTION" }
                    h2 { "The Quiet Orbit" }
                    p { class: "detail-meta", "2026 · 2h 14m · PG-13" }
                    p { class: "detail-description",
                        "A cartographer follows a dying signal beyond the mapped edge of the solar system."
                    }
                    div { class: "cast-row",
                        Avatar { initials: "MA", tone: "rose" }
                        Avatar { initials: "JL", tone: "gold" }
                        Avatar { initials: "SK", tone: "blue" }
                        span { "+12 cast" }
                    }
                    button { class: "watch-button", "▶  Watch trailer" }
                }
            }
        }
    }
}

#[component]
fn QueueSheet() -> Element {
    rsx! {
        div { class: "sheet-scene",
            RouteTransitionBase { class: "sheet-base".to_string(),
                HomeBackdrop {}
            }
            RouteTransitionCover { class: "sheet-layer".to_string(),
                button {
                    class: "sheet-scrim",
                    aria_label: "Close queue sheet",
                    onclick: move |_| async move { animated_navigate(Route::Home {}).await },
                }
                section { class: "queue-sheet",
                    div { class: "sheet-handle" }
                    div { class: "sheet-title-row",
                        div {
                            p { class: "section-kicker", "QUICK ADD" }
                            h2 { "Save to your queue" }
                        }
                        button {
                            class: "sheet-close",
                            aria_label: "Close queue sheet",
                            onclick: move |_| async move { animated_navigate(Route::Home {}).await },
                            "×"
                        }
                    }
                    QueueItem { icon: "✦", title: "Weekend watchlist", count: "14 titles", selected: true }
                    QueueItem { icon: "♡", title: "Thoughtful sci-fi", count: "8 titles", selected: false }
                    QueueItem { icon: "☾", title: "Late night", count: "21 titles", selected: false }
                    button { class: "sheet-done", "Done" }
                }
            }
        }
    }
}

#[component]
fn Gallery() -> Element {
    rsx! {
        RouteTransitionPage { class: "app-page".to_string(),
            RouteTransitionBase { class: "app-surface gallery-page".to_string(),
                PhoneHeader { eyebrow: "VISUAL STORIES", title: "The collection" }
                button {
                    class: "story-card",
                    onclick: move |_| async move { animated_navigate(Route::Story {}).await },
                    div { class: "story-art",
                        div { class: "story-sun" }
                        div { class: "mountain mountain-back" }
                        div { class: "mountain mountain-front" }
                    }
                    div { class: "story-copy",
                        span { "FIELD NOTES · 06" }
                        h2 { "Where the light stays" }
                        p { "Open story" }
                    }
                }
                p { class: "gallery-caption",
                    "Tap the feature card to see the route-level container morph."
                }
                button {
                    class: "quiet-link",
                    onclick: move |_| async move { animated_navigate(Route::Home {}).await },
                    "Back to home"
                }
            }
        }
    }
}

#[component]
fn Story() -> Element {
    rsx! {
        div { class: "story-detail",
            div { class: "story-detail-art",
                div { class: "story-sun" }
                div { class: "mountain mountain-back" }
                div { class: "mountain mountain-front" }
                button {
                    class: "round-back light",
                    aria_label: "Back to gallery",
                    onclick: move |_| async move { animated_navigate(Route::Gallery {}).await },
                    "‹"
                }
                div { class: "story-heading",
                    span { "FIELD NOTES · 06" }
                    h2 { "Where the light stays" }
                }
            }
            article {
                p { class: "dropcap",
                    "Beyond the last marked trail, the valley holds the afternoon long after the ridgeline turns blue."
                }
                p { "We followed the river until the maps became suggestions." }
            }
        }
    }
}

#[component]
fn PhoneHeader(eyebrow: &'static str, title: &'static str) -> Element {
    rsx! {
        header { class: "phone-header",
            div {
                p { "{eyebrow}" }
                h2 { "{title}" }
            }
            div { class: "profile-avatar", "M" }
        }
    }
}

#[component]
fn MiniPoster(tone: &'static str, title: &'static str, meta: &'static str) -> Element {
    rsx! {
        article { class: "mini-poster",
            div { class: "poster-art {tone}", span { "{title}" } }
            h4 { "{title}" }
            p { "{meta}" }
        }
    }
}

#[component]
fn DiscoverCard(tone: &'static str, number: &'static str, title: &'static str) -> Element {
    rsx! {
        article { class: "discover-card {tone}",
            span { class: "card-number", "{number}" }
            h3 { "{title}" }
            p { "EDITOR'S PICK" }
        }
    }
}

#[component]
fn Avatar(initials: &'static str, tone: &'static str) -> Element {
    rsx! { span { class: "cast-avatar {tone}", "{initials}" } }
}

#[component]
fn QueueItem(
    icon: &'static str,
    title: &'static str,
    count: &'static str,
    selected: bool,
) -> Element {
    rsx! {
        button { class: "queue-item",
            span { class: "queue-icon", "{icon}" }
            span { class: "queue-copy", strong { "{title}" } small { "{count}" } }
            span { class: if selected { "queue-check selected" } else { "queue-check" }, if selected { "✓" } }
        }
    }
}

#[component]
fn BottomNav(active: &'static str) -> Element {
    rsx! {
        nav { class: "bottom-nav",
            button {
                class: if active == "home" { "nav-item active" } else { "nav-item" },
                onclick: move |_| async move { animated_navigate(Route::Home {}).await },
                span { "⌂" }
                small { "Home" }
            }
            button {
                class: if active == "discover" { "nav-item active" } else { "nav-item" },
                onclick: move |_| async move { animated_navigate(Route::Discover {}).await },
                span { "◇" }
                small { "Discover" }
            }
            button {
                class: "nav-item",
                onclick: move |_| async move { animated_navigate(Route::Gallery {}).await },
                span { "▧" }
                small { "Gallery" }
            }
        }
    }
}

#[component]
fn HomeBackdrop() -> Element {
    rsx! {
        div { class: "app-surface backdrop-copy",
            PhoneHeader { eyebrow: "Saturday, September 6", title: "Good evening" }
            section { class: "hero-card",
                div { class: "hero-copy",
                    span { class: "pill", "FEATURED" }
                    h2 { "The Quiet Orbit" }
                    p { "A human story at the edge of everything." }
                }
                div { class: "planet" }
                div { class: "orbit" }
            }
            BottomNav { active: "home" }
        }
    }
}

fn transition_name(route: &Route) -> &'static str {
    match route {
        Route::Home {} => "Ready",
        Route::Discover {} => "Cross-fade",
        Route::Detail {} => "Push",
        Route::QueueSheet {} => "Sheet",
        Route::Gallery {} => "Base route",
        Route::Story {} => "Morph",
    }
}

async fn pause(milliseconds: u32) {
    TimeoutFuture::new(milliseconds).await;
}
