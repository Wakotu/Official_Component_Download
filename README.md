# Download Opensource Components from Official site

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

## Output

通过 LLM 查询到的下载页面地址会被保存到 `Official/available_url_list.json` 文件中。
从页面中找不到下载链接的下载页面地址会被保存到 `Official/abnormal_url_list.` 文件中。
