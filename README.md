# Download Opensource Components from Official site

## Prerequisites

通过 `rustup` 安装新版本的 `cargo`.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Usage

将 `config/config.toml` 中的 `username` 值修改为你的名字

```toml
[download]
username = "yangqiaoyang"
```

运行 `run.sh`

```bash
./run.sh
```
