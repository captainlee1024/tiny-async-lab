# 工具与上游源码基线

本文档记录项目在 2026-07-13 验证过的开发工具和源码研究基线，使不同机器能够使用同一组明确版本继续研究、验证和开发。

表格是版本审计入口；已接入工具的实际执行版本以“固定位置”列中的配置为准，上游源码表本身记录当前研究基线。
升级 PR 必须同时更新配置、本表和受影响的生成文件、文档、实验或代码。

## 复现边界

- 精确固定会影响 Rust 语义、上游实现或文档检查结果的工具与源码版本。
- GitHub Actions 的参考环境固定为 `ubuntu-24.04`，本地环境不要求使用相同的 Linux 发行版。
- rustup、Git、GNU Make、Bash 4+、ripgrep 以及 Chrome/Chromium 是兼容性前提，不固定补丁版本；它们不作为源码语义依据。
- 标准库源码来自固定工具链的 `rust-src`，不克隆完整的 `rust-lang/rust` 仓库。
- 未进入当前研究范围的 `bytes`、`loom`、`tracing` 和 `socket2` 等仓库不提前固定。

## 开发与验证工具

| 工具 | 固定版本 | 固定位置 |
| --- | --- | --- |
| `rustc` 与 `rust-src` | `1.91.1`，Rust commit `ed61e7d7e242494fb7057f2657300d9e77bb4fcb` | [`rust-toolchain.toml`](../rust-toolchain.toml)；[`Cargo.toml`](../Cargo.toml) 的 `rust-version` 必须一致 |
| Cargo | `1.91.1`，Cargo commit `ea2d97820c16195b0ca3fadb4319fe512c199a43` | 随固定 Rust 工具链安装 |
| Node.js / npm | `24.18.0` / `11.16.0` | [`.node-version`](../.node-version) 与 [`package.json`](../package.json) |
| mdBook | `0.5.2` | [`Makefile`](../Makefile) 与 [文档 workflow](../.github/workflows/docs.yml) |
| `mdbook-linkcheck2` | `0.11.0` | [`Makefile`](../Makefile) 与 [文档 workflow](../.github/workflows/docs.yml) |
| `mdbook-mermaid` | `0.17.0` | [`Makefile`](../Makefile) 与 [文档 workflow](../.github/workflows/docs.yml) |
| Markdownlint CLI | `0.23.0` | [`package.json`](../package.json)、[`package-lock.json`](../package-lock.json) 与 [Markdownlint 配置](../.markdownlint-cli2.jsonc) |
| Mermaid.js / Mermaid CLI | `11.6.0` | [`package.json`](../package.json) 与 [`package-lock.json`](../package-lock.json)；版本与 `mdbook-mermaid` 生成的浏览器 bundle 一致 |
| Typos | `1.48.0` | [`Makefile`](../Makefile)、[文档 workflow](../.github/workflows/docs.yml) 与 [Typos 配置](../typos.toml) |
| Lychee | `0.24.2` | [`Makefile`](../Makefile) 与 [文档 workflow](../.github/workflows/docs.yml) |

运行 `make tools` 会将 Cargo 辅助工具安装到 `.tools/`，并根据 lockfile 将 Node.js 工具安装到 `node_modules/`；两个目录均由 Git 忽略。
`make ci` 只使用这些仓库本地工具，不依赖全局安装或固定的 `/tmp` 路径。

GitHub Actions 使用完整 commit 固定第三方 action，注释中的 release tag 只帮助阅读：

| Action | Release | Commit |
| --- | --- | --- |
| `actions/checkout` | `v6.0.3` | `df4cb1c069e1874edd31b4311f1884172cec0e10` |
| `actions/setup-node` | `v6.4.0` | `48b55a011bda9f5d6aeb4c2d9c7362e8dae4041e` |
| `actions/cache` | `v5.0.5` | `27d5ce7f107fe9357f9df03efb73ab90386fccae` |
| `crate-ci/typos` | `v1.48.0` | `bee27e3a4fd1ea2111cf90ab89cd076c870fce14` |
| `lycheeverse/lychee-action` | `v2.9.0` | `e7477775783ea5526144ba13e8db5eec57747ce8` |

## 上游源码

| 上游 | Release / tag | Commit | 本地来源 |
| --- | --- | --- | --- |
| [Rust](https://github.com/rust-lang/rust) 标准库 | `1.91.1` | `ed61e7d7e242494fb7057f2657300d9e77bb4fcb` | `$(rustc --print sysroot)/lib/rustlib/src/rust/library` |
| [Tokio](https://github.com/tokio-rs/tokio) monorepo | `tokio-1.52.3` | `d87569164fb61145e79e7ffe0b25783569cc8f93` | [`checkout-upstream.sh`](../scripts/checkout-upstream.sh) 通过 `make upstream` 创建 `upstream/checkouts/tokio/` |
| [Mio](https://github.com/tokio-rs/mio) | `v1.2.0` | `ce39a6be2cc739165daaeb10cce609b9b77242ac` | [`checkout-upstream.sh`](../scripts/checkout-upstream.sh) 通过 `make upstream` 创建 `upstream/checkouts/mio/` |

Tokio 的 tag 固定整个 monorepo，其中包含 `tokio`、`tokio-macros`、`tokio-stream`、`tokio-test` 和 `tokio-util`，不为这些 crate 重复创建 checkout。

Tokio `1.52.3` 的 [`tokio/Cargo.toml`](https://github.com/tokio-rs/tokio/blob/d87569164fb61145e79e7ffe0b25783569cc8f93/tokio/Cargo.toml#L99) 声明 Mio `1.2.0`，因此 Mio 源码基线与该 Tokio release 对齐。
这个声明是 Cargo semver version requirement，不是本项目 `Cargo.lock` 的解析结果；项目实际依赖 Tokio 时还要单独记录 lockfile 解析版本。

`make upstream` 根据本表对应的固定配置创建完整历史 checkout，核对官方 `origin`、tag 的 peeled commit 和当前 `HEAD`。
这些目录由 Git 忽略且默认只读；重复执行不会访问网络，存在未提交变更时也不会自动清理或覆盖。

## 核验与升级

使用 `rustc --version --verbose` 和 `cargo --version --verbose` 核对工具链 commit，使用上游官方 remote 的 `git ls-remote --tags` 核对 tag 指向的 commit；不以浮动分支 HEAD 作为研究依据。

版本升级遵循以下规则：

1. 一个 PR 只升级一组具有直接依赖关系的基线，并说明升级原因和行为差异。
2. 同一 PR 更新执行配置、本表和必要的兼容性修改；较大的适配先拆成向前兼容的准备 PR。
3. `mdbook-mermaid` 升级时重新生成 `docs/mermaid-init.js` 与 `docs/mermaid.min.js`，并让 Mermaid CLI 与 bundle 版本保持一致。
4. Tokio 升级时重新确认其 Mio 依赖；只有研究范围实际扩展时才加入其他上游仓库。
5. 完成与风险相称的文档、实验和代码验证后再合并，Git 历史负责保存旧基线。
