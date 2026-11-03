//! In-page UI that takes native Back before the router does.
use dioxus::prelude::*;
use std::{cell::Cell, rc::Rc};

/// In-page layers holding native Back right now. Global on purpose: the
/// native-back hook runs in the app's root layout and must see layers opened
/// anywhere below it.
static BACK_LAYERS: GlobalSignal<usize> = Signal::global(|| 0);

/// Whether any [`use_native_back_layer`] is open. Reading it subscribes the
/// caller.
#[cfg_attr(
    not(all(feature = "native-back", any(target_os = "android", target_os = "ios"))),
    allow(dead_code, reason = "Read only by the native-back hook and tests.")
)]
pub(crate) fn back_layer_open() -> bool {
    BACK_LAYERS() > 0
}

/// Keep native Back intercepted while `open` is true, even at a root route
/// with no history to pop, so an in-page layer such as a sheet can close
/// on Back instead of the app exiting.
///
/// The layer must still claim each press itself: listen for the
/// `g3nativeback` window event, call `preventDefault()` synchronously, and
/// close. A press nobody claims falls through to the platform as usual.
///
/// UI libraries call this for their own surfaces (g3-ui does for its
/// sheets). An app with one fixed layer can pass it to
/// [`use_native_back_navigation_with_layer`](crate::use_native_back_navigation_with_layer)
/// instead. Without the `native-back` integration it does nothing visible.
pub fn use_native_back_layer(open: ReadSignal<bool>) {
    let held = use_hook(|| Rc::new(Cell::new(false)));
    {
        let held = held.clone();
        use_effect(move || {
            let want = open();
            if held.get() != want {
                held.set(want);
                let mut count = BACK_LAYERS.write();
                *count = if want {
                    *count + 1
                } else {
                    count.saturating_sub(1)
                };
            }
        });
    }
    // A layer whose page unmounts while it is open must not keep Back held.
    use_drop(move || {
        if held.get() {
            let mut count = BACK_LAYERS.write();
            *count = count.saturating_sub(1);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    static OPEN: GlobalSignal<bool> = Signal::global(|| false);
    static PRESENT: GlobalSignal<bool> = Signal::global(|| true);

    #[component]
    fn Layer() -> Element {
        use_native_back_layer(OPEN.signal().into());
        rsx! {}
    }

    fn app() -> Element {
        rsx! {
            if PRESENT() {
                Layer {}
            }
        }
    }

    fn count(dom: &VirtualDom) -> usize {
        dom.in_scope(ScopeId::ROOT, || *BACK_LAYERS.peek())
    }

    fn set(dom: &mut VirtualDom, signal: &'static GlobalSignal<bool>, value: bool) {
        dom.in_scope(ScopeId::ROOT, || *signal.write() = value);
        dom.render_immediate_to_vec();
    }

    #[test]
    fn a_layer_holds_back_while_open_and_releases_it_on_close_or_unmount() {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dom.render_immediate_to_vec();
        assert_eq!(count(&dom), 0);
        set(&mut dom, &OPEN, true);
        assert_eq!(count(&dom), 1);
        assert!(dom.in_scope(ScopeId::ROOT, back_layer_open));
        set(&mut dom, &OPEN, false);
        assert_eq!(count(&dom), 0);
        set(&mut dom, &OPEN, true);
        assert_eq!(count(&dom), 1);
        // Unmounted while open, as when its page navigates away.
        set(&mut dom, &PRESENT, false);
        assert_eq!(count(&dom), 0);
    }
}
