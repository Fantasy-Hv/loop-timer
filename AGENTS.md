# rest-timer

Wayland 定时循环通知器。Rust + GTK4 实现。

## 需求概要

- 状态 A（计时中）：倒计时（时长可配置），结束后进入状态 B
- 状态 B（计时结束）：弹出全屏遮罩窗口，显示休息倒计时（时长可配置），确认按钮置灰不可点击。休息倒计时结束后按钮恢复可点击。
- 用户点击确认后回到状态 A 重新计时

## 技术栈

- Rust 2021 edition
- GTK4 (`gtk4` crate v0.11) —— overlay 窗口
- `ksni` v0.3 (blocking feature) —— 系统托盘
- `notify` v8 —— 配置文件热重载
- TOML 配置，由 `serde` + `toml` 解析

## 构建与运行

```bash
cargo build
cargo run
cargo run -- --config /path/to/config.toml   # -c 也可用
```

系统依赖（Arch）：`gtk4`（无需其他系统包）

## 架构

```
src/
  main.rs       —— 入口、GTK 事件循环、timer、tray commands poll、config watcher
  config.rs     —— TOML 配置加载、默认值生成、热重载
  tray.rs       —— ksni::Tray 实现，左键暂停/继续，右键菜单（暂停/继续、重新计时、退出），tooltip 实时时间
  overlay.rs    —— 全屏遮罩窗口，每次通知动态创建、确认后销毁。含休息倒计时：确认按钮初始禁用，倒计时归零后启用，仅按钮点击可确认
  style.css     —— overlay 窗口 CSS 样式
```

## 配置

首次运行自动生成 `~/.config/rest-timer/config.toml`（可通过 `--config` / `-c` 覆盖）：

```toml
[general]
countdown_seconds = 1200  # 倒计时秒数
rest_seconds = 30         # 强制休息秒数（确认前必须等待）

[notification]
text = "该休息一下眼睛了！"
confirm_text = "我知道了 / Got it"
```

配置文件修改后自动热重载（300ms 防抖）。倒计时秒数修改后始终按最新配置重新计时。
托盘右键菜单：暂停/继续、重新计时、退出；tooltip 实时显示剩余时间。

## 依赖库 API 注意

- glib v0.22：无 `MainContext::channel`，用 `std::sync::mpsc` + `timeout_add_local` 轮询跨线程通信；`timeout_add_seconds` 需要 `Send` 闭包，用 `timeout_add_seconds_local` 避免
- GTK widgets 不是 `Send`，不能移入 `timeout_add_seconds` 的闭包；用 `Rc` 包装并从 `_local` 变体访问
- `app.hold()` 返回 `HoldGuard`，需 `std::mem::forget` 保持应用不退出
- `app.run_with_args(&["rest-timer"])` 防止 GTK 解析 `--config` 等自定义参数（参数在 GTK 初始化前由 `std::env::args()` 手动解析）
- `OverlayWindow` 不是 `Send`，用 `Rc` 包装；timer 回调使用 `timeout_add_seconds_local` / `timeout_add_local` 避免 Send 绑定
- CSS 资源 (`style.css`) 通过 `include_str!` 编译进二进制，修改 CSS 需重新编译
- 系统托盘失败时自动降级运行（`activate_no_tray`），程序不会退出
- 托盘实时更新通过 `ksni::blocking::Handle::update(|_| {})` 在每秒 timer 回调中触发；该调用阻塞主线程（通过 tokio/async-io `block_on`），但处理极快（<1ms）
- 无自动化测试
