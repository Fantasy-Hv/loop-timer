# eye-friend

Wayland 定时循环通知器，提醒你该休息眼睛了。

## 功能

- **倒计时提醒** — 可配置倒计时时长，到时弹出全屏遮罩通知
- **全屏遮罩** — 暗色半透明背景覆盖全部显示器，居中显示通知文本
- **点击确认** — 点击"我知道了"按钮或按 Escape/Enter 确认后重新计时
- **系统托盘** — 左键切换暂停/继续，右键菜单查看剩余时间和退出
- **配置热重载** — 修改配置文件自动生效，无需重启

## 依赖

- Rust 1.80+
- GTK4

```bash
# Arch
sudo pacman -S gtk4

# Debian/Ubuntu
sudo apt install libgtk-4-dev

# Fedora
sudo dnf install gtk4-devel
```

## 安装

```bash
git clone https://github.com/your/eye-friend.git
cd eye-friend
cargo build --release
sudo cp target/release/eye-friend /usr/local/bin/
```

## 使用

```bash
# 使用默认配置（~/.config/eye-friend/config.toml）
eye-friend

# 指定配置文件
eye-friend --config /path/to/config.toml
```

首次运行自动生成配置文件 `~/.config/eye-friend/config.toml`：

```toml
[general]
countdown_seconds = 1200   # 倒计时秒数（默认 20 分钟）

[notification]
text = "该休息一下眼睛了！\nTime to rest your eyes!"
```

修改配置后自动热重载（300ms 防抖）。

## 技术栈

Rust + GTK4 + ksni (StatusNotifierItem 系统托盘) + notify (文件监控)
