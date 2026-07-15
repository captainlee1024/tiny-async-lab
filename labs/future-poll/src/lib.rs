#![doc = include_str!("../README.md")]

use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::{Pin, pin};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};

/// 执行一次手动 poll 实验并返回各观察点。
pub fn run_experiment() -> Observations {
    // 第一条路径只回答“调用 async fn 是否会立刻执行函数体”。
    // `body_runs` 是可观察探针：构造 Future 时保持为 0，第一次 poll 才在函数体内加 1。
    let body_runs = Rc::new(Cell::new(0));
    let body_runs_before_first_poll = body_runs.get();
    let mut async_future = pin!(run_on_first_poll(Rc::clone(&body_runs)));

    // `poll` 总要接收一个携带 Waker 的 Context，即使这个 Future 第一次 poll 就会 Ready。
    // 这条路径不会使用 Waker；它只证明 Future 不会因为作为一个值存在就自行执行。
    let async_counter = Arc::new(WakeCounter::default());
    let async_waker = Waker::from(Arc::clone(&async_counter));
    let mut async_context = Context::from_waker(&async_waker);
    let async_first_poll = async_future.as_mut().poll(&mut async_context);
    let body_runs_after_first_poll = body_runs.get();

    // 第二条路径把“等待条件”和“收到调度通知”拆成两个独立动作。
    // 第一次 poll 返回 Pending，并把 `stale_waker` 登记为当前 task 的模拟通知入口。
    let (mut controlled_future, completion) = controlled_future();
    let stale_counter = Arc::new(WakeCounter::default());
    let stale_waker = Waker::from(Arc::clone(&stale_counter));
    let mut stale_context = Context::from_waker(&stale_waker);
    let controlled_first_poll = Pin::new(&mut controlled_future).poll(&mut stale_context);

    // 这里故意不等待 wake 就用另一个 Context 再 poll 一次，只为观察“最近 Waker”契约。
    // 真实 executor 不应在 Pending 后这样忙循环；它通常等到某个 Waker 被调用再调度 task。
    let latest_counter = Arc::new(WakeCounter::default());
    let latest_waker = Waker::from(Arc::clone(&latest_counter));
    let mut latest_context = Context::from_waker(&latest_waker);
    let controlled_second_poll = Pin::new(&mut controlled_future).poll(&mut latest_context);

    // `complete` 先把共享条件发布为 Ready，再调用最近登记的 Waker。
    // 本实验的 Waker 只累计通知次数，因此最后这次 poll 仍由测试驱动手动发起。
    // 这一步模拟真实 executor 收到 wake 后把 task 重新调度，并在轮到它时 poll 根 Future。
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
    // 这一行位于 async fn 函数体中，所以只会在生成的 Future 获得首次 poll 后执行。
    body_runs.set(body_runs.get() + 1);
    "完成"
}

fn controlled_future() -> (ControlledFuture, Completion) {
    // 两个句柄通过同一个分配共享状态，但承担相反角色：
    // `ControlledFuture` 是等待结果的消费方，`Completion` 是发布结果并发出通知的生产方。
    // `Rc<RefCell<_>>` 让实验可以在单线程中显式观察共享与可变状态。
    // 它不模拟真实跨线程 Future 所需的 `Arc`、锁、原子操作或 runtime driver。
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

// 这个 Future 自身只持有共享状态的一个所有权句柄。
// 真正决定 poll 结果的是 `SharedState::state`，不是 Waker 有没有被调用。
struct ControlledFuture {
    shared: Rc<RefCell<SharedState>>,
}

impl Future for ControlledFuture {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // `RefCell` 把借用检查移到运行时，使生产方和消费方可以修改同一份单线程状态。
        // 整个状态检查和 Waker 登记发生在一次独占借用中，因此本实验没有交错修改窗口。
        let mut shared = self.shared.borrow_mut();

        // `State::Ready` 拥有最终输出，返回 `Poll::Ready(output)` 时必须把它移出共享状态。
        // `mem::replace` 先留下 `Completed` 占位，再把旧状态按值交给 match。
        // Waiting 分支会恢复 Waiting；Ready 分支不恢复，所以 Completed 会成为稳定终态。
        match std::mem::replace(&mut shared.state, State::Completed) {
            State::Waiting => {
                // `mem::replace` 刚才暂时写入了 Completed，但条件实际仍未满足，所以先恢复 Waiting。
                shared.state = State::Waiting;

                // `Context::waker()` 的引用只在本次 poll 中有效，Future 必须克隆一个自有句柄才能留待以后通知。
                // 克隆得到的是指向同一唤醒目标的另一个句柄，并不是把这个 Future 的状态复制到 Waker 中。
                // 每次 Pending 都直接覆盖旧登记，使后续完成只通知最近一次 poll 提供的 Waker。
                // 真实实现可以用 `Waker::will_wake` 避免替换等价 Waker；本实验保留直接赋值以显式展示契约。
                shared.waker = Some(cx.waker().clone());

                // Pending 只表示这一次调用尚未交付输出。
                // Future 已安排条件改变时的通知后就应尽快返回，把执行权还给调用方。
                Poll::Pending
            }
            State::Ready(output) => {
                // `mem::replace` 已把共享状态留在 Completed，并把 output 移到了当前分支。
                // `complete` 通常已经用 `take` 移除了 Waker；这里再次清空，使完成态不保留无用通知句柄。
                shared.waker = None;

                // Ready 把这个 Future 的最终输出交给调用方。
                // 正常 executor 或父 Future 取得输出后会继续自身计算，并停止 poll 这个已完成 Future。
                Poll::Ready(output)
            }
            // 这里选择 panic，是 `ControlledFuture` 用来暴露错误重复 poll 的具体行为。
            // `Future` trait 只要求调用方在 Ready 后停止 poll，并不保证所有 Future 都会以同样方式 panic。
            State::Completed => panic!("ControlledFuture polled after completion"),
        }
    }
}

// `Completion` 模拟 Future 所等待资源的生产方或事件源。
// 它负责改变权威状态并通知等待 task，但不会亲自调用 `ControlledFuture::poll`。
struct Completion {
    shared: Rc<RefCell<SharedState>>,
}

impl Completion {
    fn complete(&self, output: &'static str) {
        // 大括号把共享状态借用限制在发布阶段。
        // 调用 Waker 前必须先结束借用，避免把内部可变借用跨越到 executor 定义的外部唤醒行为。
        let waker = {
            let mut shared = self.shared.borrow_mut();

            // 当前实验只允许生产方完成一次，合法的生产方状态转换只有 Waiting -> Ready。
            assert!(
                matches!(shared.state, State::Waiting),
                "ControlledFuture completed more than once"
            );

            // 第一步发布事实：从此以后 poll 可以观察到最终输出。
            // 这一步才改变 Future 的 readiness；Waker 本身不携带或修改这个状态。
            shared.state = State::Ready(output);

            // 第二步取出最近登记的通知句柄，并让共享状态不再持有它。
            // 如果 Future 从未返回过 Pending，这里可以得到 None；没有登记的等待 task 就无需通知。
            shared.waker.take()
        };

        if let Some(waker) = waker {
            // `wake(self)` 消耗这个自有 Waker，并执行其实现定义的唤醒行为。
            // 在真实 executor 中，这通常会把 Waker 所代表的 task 标记为可运行或放回 ready queue。
            // 它通知的是 task，而不是直接递归调用这个具体的子 Future。
            // task 以后被 poll 时，poll 链才会再次到达仍受关注的子 Future。
            waker.wake();
        }

        // 通用 wake 只表示“现在值得再 poll”，下一次 poll 仍可能 Pending。
        // 当前实验更强：Ready 没有合法路径退回 Waiting，所以 complete 后的下一次合法 poll 必定 Ready。
    }
}

// `SharedState` 同时保存权威资源状态和最近等待者的通知入口，但两者不能混为一个信号。
struct SharedState {
    // `state` 决定 poll 应返回 Pending、Ready，还是暴露完成后重复 poll。
    state: State,
    // `waker` 只回答“条件改变后通知哪个 task”，不表示结果是否已经准备好。
    waker: Option<Waker>,
}

// 对外可观察的稳定状态转换是：
// Waiting --poll--> Waiting，登记最近 Waker 并返回 Pending；
// Waiting --complete--> Ready，保存输出后通知最近 Waker；
// Ready --poll--> Completed，移出输出并返回 Ready。
enum State {
    Waiting,
    Ready(&'static str),
    // `mem::replace` 也会把 Completed 短暂用作取出旧状态的占位值。
    // Waiting 分支在释放借用前恢复状态，只有 Ready 分支会把它保留为稳定终态。
    Completed,
}

// 这个类型故意只观察 wake 次数，不实现 task、ready queue 或 executor。
// `Waker::from(Arc<W>)` 使用安全的 `Wake` 路径构造 Waker；Mutex 允许唤醒路径安全地修改计数。
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
        // 真实 executor 会在这里连接任务调度；实验只计数，重新 poll 由 `run_experiment` 显式完成。
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

        // `RepeatReady` 选择每次都返回相同值，证明重复 poll 不必然 panic。
        let mut repeat_ready = RepeatReady;
        assert_eq!(
            Pin::new(&mut repeat_ready).poll(&mut context),
            Poll::Ready("重复完成")
        );
        assert_eq!(
            Pin::new(&mut repeat_ready).poll(&mut context),
            Poll::Ready("重复完成")
        );

        // 标准库 `future::Ready` 会取走内部值，第二次 poll 时 panic。
        // 两种具体行为不同，正说明不能从单个实现外推 Future trait 的完成后行为。
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
