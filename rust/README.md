# NimCode — Rust 实现（NVIDIA NIM 专用）

基于 NVIDIA NIM 的高性能 Rust 命令行 AI 编程助手。所有模型请求通过
`https://integrate.api.nvidia.com/v1/chat/completions` 的 OpenAI 兼容接口发送。

## 快速开始

```bash
cd rust/
cargo build --workspace

# 设置 API Key
export NVIDIA_NIM_API_KEY="nvapi-..."

# 交互式 REPL
cargo run -p nimcode-cli

# 指定模型
cargo run -p nimcode-cli -- --model nvidia_nim/moonshotai/kimi-k2.5

# 单次提问
cargo run -p nimcode-cli -- prompt "explain this codebase"
```

## 配置

```bash
# 必需：NIM API Key
export NVIDIA_NIM_API_KEY="nvapi-..."

# 可选：自定义端点
export NVIDIA_NIM_BASE_URL="https://integrate.api.nvidia.com/v1"

# 可选：界面语言
export NIMCODE_LANG="zh"
```

## 模型别名

| 别名 | 实际模型 |
|------|----------|
| `opus`, `sonnet` | `deepseek-ai/deepseek-v4-pro` |
| `haiku` | `deepseek-ai/deepseek-v4-flash` |
| `deepseek`, `deepseek-v4` | `deepseek-ai/deepseek-v4-pro` |
| `kimi`, `kimi-k2` | `moonshotai/kimi-k2.5` |
| `glm`, `glm5` | `z-ai/glm-5.1` |
| `qwen`, `qwen-coder` | `qwen/qwen3-coder-480b-a35b-instruct` |
| `minimax` | `minimaxai/minimax-m2.5` |

## Workspace 布局

```
rust/
├── Cargo.toml
├── Cargo.lock
└── crates/
    ├── api/              # NIM OpenAI 兼容客户端、SSE 流式处理
    ├── commands/         # 斜杠命令定义与解析
    ├── compat-harness/   # TS manifest 提取
    ├── plugins/          # 插件管理
    ├── runtime/          # 会话、配置、权限、MCP、系统提示
    ├── rusty-claude-cli/ # 主 CLI（二进制名: nimcode）
    ├── telemetry/        # 遥测
    └── tools/            # 内置工具集
```

## 许可证

MIT
