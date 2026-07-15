fn main() {
    let observations = future_poll::run_experiment();

    println!(
        "async fn 函数体：首次 poll 前执行 {} 次，首次 poll 后执行 {} 次，结果 {:?}",
        observations.body_runs_before_first_poll,
        observations.body_runs_after_first_poll,
        observations.async_first_poll
    );
    println!(
        "受控 Future：前两次 poll 分别为 {:?} 和 {:?}",
        observations.controlled_first_poll, observations.controlled_second_poll
    );
    println!(
        "条件改变：旧 Waker 收到 {} 次 wake，最近 Waker 收到 {} 次 wake",
        observations.stale_waker_wakes, observations.latest_waker_wakes
    );
    println!(
        "重新 poll：结果 {:?}",
        observations.controlled_completion_poll
    );
}
