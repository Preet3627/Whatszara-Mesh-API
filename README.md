# Whatszara — Desktop Assistant via WhatsApp + LLM

Control your desktop from anywhere using WhatsApp messages. Talk to an LLM through WhatsApp, and it executes your commands — shell, apps, media, file access — with a secure permission system.

Built on top of **[whatsapp-mcp](https://github.com/lharries/whatsapp-mcp)** by Luke Harries, scaled from a simple MCP server into a full desktop automation platform.

## How It Works

```
WhatsApp Message ──▶ Orchestrator ──▶ LLM (Ollama/Claude/Groq/etc.)
                        │                        │
                        ▼                        ▼
                 Permission Check          Decides Action
                        │                        │
                        └────────┬───────────────┘
                                 ▼
                        System Action (shell, apps, media...)
                                 │
                                 ▼
                        Result sent back via WhatsApp
```

## The Scaling Story: from whatsapp-mcp to Whatszara

[whatsapp-mcp](https://github.com/lharries/whatsapp-mcp) is a focused MCP server with **12 tools** that lets Claude read and send WhatsApp messages. It's:
- **2 components**: Go bridge + Python MCP server
- **3 source files**: `main.go`, `main.py`, `whatsapp.py`
- **Single-direction**: LLM talks TO WhatsApp

**Whatszara inverts this.** WhatsApp messages trigger the LLM to control the desktop. Here's what we added:

| Dimension | whatsapp-mcp | Whatszara |
|-----------|-------------|-----------|
| **Message flow** | LLM → WhatsApp | WhatsApp → LLM → Desktop |
| **LLM support** | Claude only | Ollama, Claude, Groq, Grok, Gemini, Vercel AI SDK |
| **Actions** | None (read/send only) | Shell, open apps, volume, media, file scan, send images |
| **Permissions** | None | reCAPTCHA + image-to-text, 3 risk tiers, undo system |
| **Interface** | CLI config only | Tauri desktop app with GUI |
| **Scope** | WhatsApp tool | Full desktop assistant |

## Features

### ✅ Completed (Phase 1)
- [x] Project scaffold and structure
- [x] Derived from whatsapp-mcp (MIT licensed, attribution maintained)
- [x] Multi-LLM provider abstraction (Ollama, Claude, Groq, Grok, Gemini, Vercel AI SDK)
- [x] Live model list fetching for Ollama
- [x] Basic orchestrator with message routing
- [x] Tauri desktop app shell
- [x] Permission engine scaffold with risk profiles (High/Medium/Low)

### 🔄 In Progress
- [ ] System action tools (shell, open apps, volume control, media playback)
- [ ] Desktop file scanning and bulk image sending
- [ ] reCAPTCHA + image-to-text verification integration
- [ ] Undo/reversible action journal
- [ ] WhatsApp incoming message webhook

### 📋 Planned
- [ ] Permission configuration GUI
- [ ] Action history viewer
- [ ] Scheduled/automated actions
- [ ] Multiple WhatsApp number support
- [ ] Voice message transcription
- [ ] Plugin system for custom tools

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                        Whatszara                                 │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌─────────────────────┐      ┌──────────────────────────────┐   │
│  │   WhatsApp Layer     │      │    Orchestrator               │   │
│  │   (whatsapp-mcp)     │─────▶│                               │   │
│  │                      │      │  ┌─────────────────────────┐  │   │
│  │  ┌───────────────┐   │      │  │  LLM Provider Layer     │  │   │
│  │  │ Go Bridge      │   │      │  │  - Ollama (live list)   │  │   │
│  │  │ (whatsmeow)    │   │      │  │  - Claude               │  │   │
│  │  └───────┬───────┘   │      │  │  - Groq                 │  │   │
│  │          │           │      │  │  - Grok                 │  │   │
│  │  ┌───────▼───────┐   │      │  │  - Gemini               │  │   │
│  │  │ SQLite Store   │   │      │  │  - Vercel AI SDK        │  │   │
│  │  └───────┬───────┘   │      │  └─────────────────────────┘  │   │
│  │          │           │      │                               │   │
│  │  ┌───────▼───────┐   │      │  ┌─────────────────────────┐  │   │
│  │  │ Python MCP     │   │      │  │  Permission Engine      │  │   │
│  │  │ Server         │   │      │  │  - Risk profiles        │  │   │
│  │  └───────┬───────┘   │      │  │  - reCAPTCHA v3          │  │   │
│  │          │           │      │  │  - Image-to-text CAPTCHA │  │   │
│  └──────────┼───────────┘      │  └─────────────────────────┘  │   │
│             │                  │                               │   │
│             │                  │  ┌─────────────────────────┐  │   │
│             │                  │  │  Action Engine           │  │   │
│             │                  │  │  - Shell commands        │  │   │
│             │                  │  │  - App launcher          │  │   │
│             │                  │  │  - Volume/media control  │  │   │
│             │                  │  │  - Desktop file access   │  │   │
│             │                  │  │  - Undo journal          │  │   │
│             │                  │  └─────────────────────────┘  │   │
│             │                  └──────────────────────────────┘   │
│             │                                                     │
│             │                  ┌──────────────────────────────┐   │
│             └──────────────────│   Tauri Desktop App (GUI)    │   │
│                                │  - QR auth display           │   │
│                                │  - Settings configuration    │   │
│                                │  - Permission editor         │   │
│                                │  - Activity log              │   │
│                                └──────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
```

## Quick Start

### Prerequisites
- Go
- Python 3.11+
- Node.js 20+ (for Tauri desktop app)
- Rust (for Tauri)
- FFmpeg (optional, for audio)

### Setup

```bash
# 1. Start the WhatsApp bridge
cd whatsapp-bridge && go run main.go
# Scan QR code with WhatsApp mobile app

# 2. Start the orchestrator (in another terminal)
cd orchestrator && python main.py

# 3. Start the desktop app (in another terminal)
cd desktop-app && npm install && npm run tauri dev
```

### Configuring LLM Providers

Edit `orchestrator/config.yaml` or use the desktop GUI:

```yaml
providers:
  ollama:
    enabled: true
    endpoint: http://localhost:11434
    model: llama3
  claude:
    enabled: true
    api_key: ${ANTHROPIC_API_KEY}
    model: claude-sonnet-4-20250514
  groq:
    enabled: false
    api_key: ${GROQ_API_KEY}
  gemini:
    enabled: false
    api_key: ${GEMINI_API_KEY}
```

## Permission System

| Risk Level | Example Actions | Verification Required |
|-----------|----------------|---------------------|
| **Low** | Read volume, list files, get time | None (logged only) |
| **Medium** | Open apps, play music, send files | Image-to-text CAPTCHA |
| **High** | Shell commands, delete, install software | reCAPTCHA + image-to-text + confirm |

## License

This project is **MIT Licensed** (see [LICENSE](LICENSE)).

The WhatsApp bridge and MCP server components incorporate code from
[whatsapp-mcp](https://github.com/lharries/whatsapp-mcp) by Luke Harries,
also MIT licensed. See [LICENSE-THIRD-PARTY](LICENSE-THIRD-PARTY) for attribution.
