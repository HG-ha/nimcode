<p align="center">
  <img src="assets/nimcode-icon.png" alt="NimCode" width="128" />
</p>

<h1 align="center">NimCode</h1>
<p align="center">Powering Claw Code with free models from NVIDIA NIM — that's NimCode</p>

<p align="center">
  English | <a href="README.md">中文</a>
</p>

## Installation

### One-line Install (Recommended)

**Linux / macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/HG-ha/nimcode/main/install.sh | bash
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/HG-ha/nimcode/main/install.ps1 | iex
```

### Build from Source

```bash
git clone https://github.com/HG-ha/nimcode
cd nimcode/rust
cargo install --path crates/rusty-claude-cli
```

The `nimcode` binary will be installed to `~/.cargo/bin/`.

### Manual Download

Download the binary for your platform from [Releases](https://github.com/HG-ha/nimcode/releases).

### Upgrade

```bash
nimcode upgrade
```

## Quick Start

```bash
# Just run it — you'll be prompted for an API key on first launch
nimcode

# Or set the env var beforehand
export NVIDIA_NIM_API_KEY="nvapi-..."
nimcode
```

On first launch, NimCode will guide you to enter your NVIDIA NIM API Key (get one for free at [build.nvidia.com](https://build.nvidia.com/)). It's saved automatically — no need to enter it again.

## Features

- **NVIDIA NIM Backend** — Access all NIM models (DeepSeek, Kimi, GLM, Qwen, etc.)
- **Interactive REPL** — Tab completion, command history, slash commands
- **Bilingual (EN/ZH)** — Auto-detects system language, switch with `/lang`
- **Dynamic Model Switching** — Fuzzy search the NIM model catalog with `/model`
- **Dev Tools** — Built-in file I/O, search, Git, shell execution
- **Session Management** — Auto-save, resume with `/resume`
- **Streaming Output** — SSE streaming with reasoning content display
- **Self-upgrade** — `nimcode upgrade` to update to the latest version
- **MCP / Plugins** — Extensible architecture

## Common Commands

| Command | Description |
|---------|-------------|
| `/help` | Show all commands |
| `/model list` | List available models |
| `/model deepseek` | Fuzzy search and switch model |
| `/lang en` | Switch to English |
| `/status` | Show session status |
| `/upgrade` | Upgrade to latest version |
| `/diff` → `/commit` | Review changes and commit |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `NVIDIA_NIM_API_KEY` | NIM API key (auto-prompted on first run) |
| `NVIDIA_NIM_BASE_URL` | Custom NIM endpoint (default: `integrate.api.nvidia.com/v1`) |
| `NIMCODE_LANG` | UI language: `zh` / `en` / `auto` |
| `NIMCODE_MODEL` | Default model |

## Project Structure

```
nimcode/
├── assets/             # Icons and resources
├── install.sh          # Linux/macOS install script
├── install.ps1         # Windows install script
├── README.md
├── NIMCODE.md
└── rust/
    ├── Cargo.toml      # Workspace root
    ├── Cargo.lock
    └── crates/
        ├── api/            # NIM API client
        ├── commands/       # Slash commands
        ├── runtime/        # Session, config, permissions
        ├── tools/          # Built-in tools
        ├── plugins/        # Plugin system
        ├── rusty-claude-cli/ # CLI entry point (bin: nimcode)
        ├── compat-harness/
        └── telemetry/
```

## License

MIT
