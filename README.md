<p align="center">
  <img src="assets/nimcode-icon.png" alt="NimCode" width="128" />
</p>

<h1 align="center">NimCode</h1>
<p align="center">基于 NVIDIA NIM 的命令行 AI 编程助手</p>

## 安装

### 一键安装（推荐）

**Linux / macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/HG-ha/nimcode/main/install.sh | bash
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/HG-ha/nimcode/main/install.ps1 | iex
```

### 从源码编译

```bash
git clone https://github.com/HG-ha/nimcode
cd nimcode/rust
cargo install --path crates/rusty-claude-cli
```

编译后 `nimcode` 会安装到 `~/.cargo/bin/`。

### 手动下载

从 [Releases](https://github.com/HG-ha/nimcode/releases) 下载对应平台的二进制文件。

## 快速开始

```bash
# 直接运行，首次会提示输入 API Key
nimcode

# 或提前设置环境变量
export NVIDIA_NIM_API_KEY="nvapi-..."
nimcode
```

首次启动时，NimCode 会引导你输入 NVIDIA NIM API Key（从 [build.nvidia.com](https://build.nvidia.com/) 免费获取），输入后自动保存，以后不用再输。

## 功能

- **NVIDIA NIM 后端** — 支持 NIM 平台全部模型（DeepSeek、Kimi、GLM、Qwen 等）
- **交互式 REPL** — Tab 补全、命令历史、斜杠命令
- **中英双语** — 根据系统语言自动切换，`/lang` 手动切换
- **动态模型切换** — `/model` 实时搜索 NIM 模型目录
- **开发工具** — 内置文件读写、搜索、Git、Shell 执行等
- **会话管理** — 自动保存，`/resume` 恢复历史对话
- **流式输出** — SSE 流式响应，支持推理内容展示
- **MCP / 插件** — 可扩展架构

## 常用命令

| 命令 | 说明 |
|------|------|
| `/help` | 查看所有命令 |
| `/model list` | 查看可用模型 |
| `/model deepseek` | 模糊搜索并切换模型 |
| `/lang zh` | 切换中文 |
| `/status` | 查看会话状态 |
| `/diff` → `/commit` | 查看改动并提交 |

## 环境变量

| 变量 | 说明 |
|------|------|
| `NVIDIA_NIM_API_KEY` | NIM API 密钥（首次运行时会自动引导设置） |
| `NVIDIA_NIM_BASE_URL` | 自定义 NIM 端点（默认 `integrate.api.nvidia.com/v1`） |
| `NIMCODE_LANG` | 界面语言：`zh` / `en` / `auto` |
| `NIMCODE_MODEL` | 默认模型 |

## 项目结构

```
nimcode/
├── assets/             # 图标等资源
├── install.sh          # Linux/macOS 安装脚本
├── install.ps1         # Windows 安装脚本
├── README.md
├── NIMCODE.md
└── rust/
    ├── Cargo.toml      # Workspace 根
    ├── Cargo.lock
    └── crates/
        ├── api/            # NIM API 客户端
        ├── commands/       # 斜杠命令
        ├── runtime/        # 会话、配置、权限
        ├── tools/          # 内置工具
        ├── plugins/        # 插件系统
        ├── rusty-claude-cli/ # CLI 主程序 (bin: nimcode)
        ├── compat-harness/
        └── telemetry/
```

## 许可证

MIT
