//! Forward navigations that have started but not yet changed the route.
//!
//! An animated navigation changes the route only once the old page has been
//! captured, a frame or more after the call. A Back that started inside that
//! window would pop history from the page being left while the forward
//! navigation's push was still to come, and which of the two landed first
//! decided where the user ended up. Back waits for those route changes
//! instead, so it always leaves the page the forward navigation arrived at.
use std::{
    cell::RefCell,
    future::{Future, poll_fn},
    mem,
    task::{Poll, Waker},
};

#[derive(Default)]
struct Pending {
    /// Forward navigations that have not changed the route yet.
    count: usize,
    /// Tasks waiting for `count` to reach zero.
    waiting: Vec<Waker>,
}

thread_local! {
    // Dioxus polls every task of a VirtualDom on its own thread.
    static PENDING: RefCell<Pending> = RefCell::new(Pending::default());
}

/// One forward navigation's route change, still to come. Finished when the
/// route changes, or when the navigation is dropped without changing it.
pub(crate) struct PendingRouteChange {
    open: bool,
}

impl PendingRouteChange {
    pub(crate) fn start() -> Self {
        PENDING.with_borrow_mut(|pending| pending.count += 1);
        Self { open: true }
    }

    /// The route has changed (or never will); wake anything waiting.
    pub(crate) fn finish(&mut self) {
        if !mem::replace(&mut self.open, false) {
            return;
        }
        let ready = PENDING.with_borrow_mut(|pending| {
            pending.count = pending.count.saturating_sub(1);
            if pending.count == 0 {
                mem::take(&mut pending.waiting)
            } else {
                Vec::new()
            }
        });
        for waker in ready {
            waker.wake();
        }
    }
}

impl Drop for PendingRouteChange {
    fn drop(&mut self) {
        self.finish();
    }
}

/// Resolves once every forward navigation started so far has changed the
/// route, immediately when none is in flight.
pub(crate) fn route_changes_landed() -> impl Future<Output = ()> {
    poll_fn(|context| {
        PENDING.with_borrow_mut(|pending| {
            if pending.count == 0 {
                return Poll::Ready(());
            }
            if !pending
                .waiting
                .iter()
                .any(|waker| waker.will_wake(context.waker()))
            {
                pending.waiting.push(context.waker().clone());
            }
            Poll::Pending
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{pin::pin, task::Context};

    fn poll_once(future: std::pin::Pin<&mut impl Future<Output = ()>>) -> Poll<()> {
        future.poll(&mut Context::from_waker(Waker::noop()))
    }

    #[test]
    fn nothing_in_flight_is_ready_at_once() {
        let mut landed = pin!(route_changes_landed());
        assert_eq!(poll_once(landed.as_mut()), Poll::Ready(()));
    }

    #[test]
    fn back_waits_for_every_forward_route_change() {
        let mut first = PendingRouteChange::start();
        let second = PendingRouteChange::start();
        let mut landed = pin!(route_changes_landed());
        assert_eq!(poll_once(landed.as_mut()), Poll::Pending);
        first.finish();
        // Finishing twice counts once.
        first.finish();
        assert_eq!(poll_once(landed.as_mut()), Poll::Pending);
        // A navigation dropped before it changed the route releases it too.
        drop(second);
        assert_eq!(poll_once(landed.as_mut()), Poll::Ready(()));
    }
}
