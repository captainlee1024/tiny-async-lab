# Lab：手动 poll Future

在继续研究 `Future` 与 `Poll` 前，先运行并逐行阅读 [`future-poll` 观察版（`39c7969`）](https://github.com/captainlee1024/tiny-async-lab/tree/39c7969231ac1ae24d1bf64fb30419633bcb6875/labs/future-poll)。
这个实验用于观察 `async fn` 的惰性执行、`Pending`、最近登记的 Waker，以及条件改变后的重新 poll。

运行方式和实验边界见 [lab README](https://github.com/captainlee1024/tiny-async-lab/blob/39c7969231ac1ae24d1bf64fb30419633bcb6875/labs/future-poll/README.md)。
先形成自己的解释；完成讨论后，再用后续加入的解释版进行对照。
