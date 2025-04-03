# Download Opensource Components from Official site

## TODO

- [ ] add functionality to handle 3-hop link parsing: official site -> download page -> download link

## Prerequisites

通过 `rustup` 安装新版本的 `cargo`.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

> [!NOTE]
> 遇到错误提示输入 y 回车即可。安装完成后需要重启终端。

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
