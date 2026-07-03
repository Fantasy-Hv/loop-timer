# eye-friend

Wayland 定时循环通知器。Rust + GTK4 实现。

## 需求概要

- 状态 A（计时中）：倒计时（时长可配置），结束后进入状态 B
- 状态 B（计时结束）：弹出全屏遮罩窗口通知（文本可配置），等待用户点击确认
- 用户点击确认后回到状态 A 重新计时

## 技术栈

- Rust 2021 edition
- GTK4 (`gtk4` crate v0.11) + `gtk4-layer-shell` v0.8 —— overlay 窗口
- `ksni` v0.3 (blocking feature) —— 系统托盘
- `notify` v8 —— 配置文件热重载
- TOML 配置，由 `serde` + `toml` 解析

## 构建与运行

```bash
cargo build
cargo run
cargo run -- --config /path/to/config.toml
```

系统依赖（Arch）：
```
gtk4 gtk4-layer-shell
```

## 架构

```
src/
  main.rs       —— 入口、GTK 事件循环、timer、tray commands poll、config watcher
  config.rs     —— TOML 配置加载、默认值生成、热重载
  tray.rs       —— ksni::Tray 实现，菜单项和 tooltip 通过 mpsc 与主线程通信
  overlay.rs    —— 全屏遮罩窗口，wlr-layer-shell overlay 层，独占键盘
  style.css     —— overlay 窗口 CSS 样式
```

## 配置

首次运行自动生成 `~/.config/eye-friend/config.toml`：

```toml
[general]
countdown_seconds = 1200  # 倒计时秒数

[notification]
text = "该休息一下眼睛了！\nTime to rest your eyes!"
```

配置文件修改后自动热重载（300ms 防抖）。倒计时秒数修改后仅在 A 状态下重置。

## 依赖库 API 注意

- `gtk4-layer-shell` v0.8：trait 方法是 `init_layer_shell()`（不是 `init_for_window`），导入 `use gtk4_layer_shell::LayerShell;`
- `ksni` v0.3 blocking：使用 `ksni::blocking::TrayMethods::spawn()` 同步启动，`MenuItem::Standard(StandardItem { .. })` 结构体语法
- glib v0.22：无 `MainContext::channel`，用 `std::sync::mpsc` + `timeout_add_local` 轮询跨线程通信；`timeout_add_seconds` 需要 `Send` 闭包，用 `timeout_add_seconds_local` 避免
- GTK widgets 不是 `Send`，不能移入 `timeout_add_seconds` 的闭包；用 `Rc` 包装并从 `_local` 变体访问
