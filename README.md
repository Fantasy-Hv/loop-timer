# rest-timer

Wayland 定时休息提醒器，提醒你该休息了。

## 功能

- **倒计时提醒** — 可配置倒计时时长，到时弹出全屏遮罩通知
- **强制休息** — 计时结束后显示休息倒计时，确认按钮置灰禁用，休息结束后方可确认
- **全屏遮罩** — 暗色半透明背景覆盖全部显示器，居中显示通知文本和休息倒计时
- **点击确认** — 仅按钮点击确认，不支持键盘快捷键
- **系统托盘** — 左键切换暂停/继续，右键菜单查看剩余时间、重新计时和退出
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
git clone https://github.com/Fantasy-Hv/rest-timer.git
cd rest-timer
cargo build --release
sudo cp target/release/rest-timer /usr/local/bin/
```

## 使用

```bash
# 使用默认配置（~/.config/rest-timer/config.toml）
rest-timer

# 指定配置文件
rest-timer --config /path/to/config.toml
```

首次运行自动生成配置文件 `~/.config/rest-timer/config.toml`：

```toml
[general]
countdown_seconds = 1200   # 倒计时秒数（默认 20 分钟）
rest_seconds = 30         # 强制休息秒数（确认前必须等待）

[notification]
text = "该休息一下眼睛了！"
confirm_text = "我知道了 / Got it"
```

## 技术栈

Rust + GTK4 + ksni + notify + serde/toml
