#![doc = include_str!("../README.md")]

use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::{Pin, pin};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};

/// 执行一次手动 poll 实验并返回各观察点。
pub fn run_experiment() -> Observations {
    let body_runs = Rc::new(Cell::new(0));
    let body_runs_before_first_poll = body_runs.get();
    let mut async_future = pin!(run_on_first_poll(Rc::clone(&body_runs)));
    let async_counter = Arc::new(WakeCounter::default());
    let async_waker = Waker::from(Arc::clone(&async_counter));
    let mut async_context = Context::from_waker(&async_waker);
    let async_first_poll = async_future.as_mut().poll(&mut async_context);
    let body_runs_after_first_poll = body_runs.get();

    let (mut controlled_future, completion) = controlled_future();
    let stale_counter = Arc::new(WakeCounter::default());
    let stale_waker = Waker::from(Arc::clone(&stale_counter));
    let mut stale_context = Context::from_waker(&stale_waker);
    let controlled_first_poll = Pin::new(&mut controlled_future).poll(&mut stale_context);

    let latest_counter = Arc::new(WakeCounter::default());
    let latest_waker = Waker::from(Arc::clone(&latest_counter));
    let mut latest_context = Context::from_waker(&latest_waker);
    let controlled_second_poll = Pin::new(&mut controlled_future).poll(&mut latest_context);

    completion.complete("完成");
    let stale_waker_wakes = stale_counter.wake_count();
    let latest_waker_wakes = latest_counter.wake_count();
    let controlled_completion_poll = Pin::new(&mut controlled_future).poll(&mut latest_context);

    Observations {
        body_runs_before_first_poll,
        body_runs_after_first_poll,
        async_first_poll,
        controlled_first_poll,
        controlled_second_poll,
        stale_waker_wakes,
        latest_waker_wakes,
        controlled_completion_poll,
    }
}

/// 手动 poll 实验在各边界得到的观察结果。
#[derive(Debug, Eq, PartialEq)]
pub struct Observations {
    /// 调用 `async fn` 后、首次 poll 前的函数体执行次数。
    pub body_runs_before_first_poll: usize,
    /// 首次 poll 后的函数体执行次数。
    pub body_runs_after_first_poll: usize,
    /// `async fn` 返回的 Future 在首次 poll 时给出的结果。
    pub async_first_poll: Poll<&'static str>,
    /// 受控 Future 使用第一个 Waker 时给出的结果。
    pub controlled_first_poll: Poll<&'static str>,
    /// 受控 Future 使用第二个 Waker 时给出的结果。
    pub controlled_second_poll: Poll<&'static str>,
    /// 条件改变后，第一个旧 Waker 收到的 wake 次数。
    pub stale_waker_wakes: usize,
    /// 条件改变后，最近登记的 Waker 收到的 wake 次数。
    pub latest_waker_wakes: usize,
    /// 条件改变并收到通知后重新 poll 的结果。
    pub controlled_completion_poll: Poll<&'static str>,
}

async fn run_on_first_poll(body_runs: Rc<Cell<usize>>) -> &'static str {
    body_runs.set(body_runs.get() + 1);
    "完成"
}

fn controlled_future() -> (ControlledFuture, Completion) {
    let shared = Rc::new(RefCell::new(SharedState {
        state: State::Waiting,
        waker: None,
    }));

    (
        ControlledFuture {
            shared: Rc::clone(&shared),
        },
        Completion { shared },
    )
}

struct ControlledFuture {
    shared: Rc<RefCell<SharedState>>,
}

impl Future for ControlledFuture {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared = self.shared.borrow_mut();

        match std::mem::replace(&mut shared.state, State::Completed) {
            State::Waiting => {
                shared.state = State::Waiting;
                shared.waker = Some(cx.waker().clone());
                Poll::Pending
            }
            State::Ready(output) => {
                shared.waker = None;
                Poll::Ready(output)
            }
            State::Completed => panic!("ControlledFuture polled after completion"),
        }
    }
}

struct Completion {
    shared: Rc<RefCell<SharedState>>,
}

impl Completion {
    fn complete(&self, output: &'static str) {
        let waker = {
            let mut shared = self.shared.borrow_mut();

            assert!(
                matches!(shared.state, State::Waiting),
                "ControlledFuture completed more than once"
            );
            shared.state = State::Ready(output);
            shared.waker.take()
        };

        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

struct SharedState {
    state: State,
    waker: Option<Waker>,
}

enum State {
    Waiting,
    Ready(&'static str),
    Completed,
}

#[derive(Default)]
struct WakeCounter {
    wakes: Mutex<usize>,
}

impl WakeCounter {
    fn wake_count(&self) -> usize {
        *self.wakes.lock().expect("wake counter lock poisoned")
    }
}

impl Wake for WakeCounter {
    fn wake(self: Arc<Self>) {
        *self.wakes.lock().expect("wake counter lock poisoned") += 1;
    }
}

#[cfg(test)]
mod tests {
    use std::future;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use super::*;

    #[test]
    fn observes_lazy_execution_and_latest_waker() {
        let observations = run_experiment();

        assert_eq!(observations.body_runs_before_first_poll, 0);
        assert_eq!(observations.body_runs_after_first_poll, 1);
        assert_eq!(observations.async_first_poll, Poll::Ready("完成"));
        assert_eq!(observations.controlled_first_poll, Poll::Pending);
        assert_eq!(observations.controlled_second_poll, Poll::Pending);
        assert_eq!(observations.stale_waker_wakes, 0);
        assert_eq!(observations.latest_waker_wakes, 1);
        assert_eq!(observations.controlled_completion_poll, Poll::Ready("完成"));
    }

    #[test]
    fn completion_behavior_is_specific_to_each_future() {
        let counter = Arc::new(WakeCounter::default());
        let waker = Waker::from(counter);
        let mut context = Context::from_waker(&waker);

        let mut repeat_ready = RepeatReady;
        assert_eq!(
            Pin::new(&mut repeat_ready).poll(&mut context),
            Poll::Ready("重复完成")
        );
        assert_eq!(
            Pin::new(&mut repeat_ready).poll(&mut context),
            Poll::Ready("重复完成")
        );

        let mut ready_once = future::ready("一次完成");
        assert_eq!(
            Pin::new(&mut ready_once).poll(&mut context),
            Poll::Ready("一次完成")
        );
        let second_poll = catch_unwind(AssertUnwindSafe(|| {
            Pin::new(&mut ready_once).poll(&mut context)
        }));

        assert!(second_poll.is_err());
    }

    struct RepeatReady;

    impl Future for RepeatReady {
        type Output = &'static str;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            Poll::Ready("重复完成")
        }
    }
}
