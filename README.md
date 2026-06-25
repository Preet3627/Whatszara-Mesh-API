# Whatszara — Desktop Assistant via WhatsApp + LLM

[![GitHub](https://img.shields.io/github/license/Preet3627/whatszara?style=flat-square&color=25D366)](LICENSE)

Control your desktop from anywhere using WhatsApp messages. Talk to an LLM through WhatsApp, and it executes your commands — shell, apps, media, file access — with a secure permission system and risk-based approval.

Built on top of **[whatsapp-mcp](https://github.com/lharries/whatsapp-mcp)** by Luke Harries, scaled from a simple MCP server into a full desktop automation platform.

**No Python required.** Everything is compiled into a single Tauri desktop app (Rust) + Go bridge for WhatsApp. No interpreters, no virtualenvs, no pip.

## How It Works

```
WhatsApp Message ──▶ Orchestrator ──▶ LLM (Ollama/Claude/Groq/etc.)
                        │                        │
                        ▼                        ▼
                 Policy/Risk Check       Returns JSON with "chat"
                         │                 + optional "actions" array
                         │                   (tool, params, thinking,
                         │                    delay_ms)
                         │                        │
                         └────────┬───────────────┘
                                  ▼
                     ┌──────────────────────────────────┐
                     │  Multi-Action Processing          │
                     │  For each action in array:        │
                     │   1. Show "thinking" block        │
                     │   2. Wait delay_ms (if set)       │
                     │   3. Evaluate risk                │
                     │   4. Auto-execute or queue pending │
                     └──────────┬───────────────────────┘
                                │
                     ┌──────────▼──────────┐
                     │  Any pending?       │
                     │  Yes → Send batch   │
                     │  approval request   │
                     │  via WhatsApp       │
                     │  User replies ALLOW │
                     │  ALL or individually │
                     └──────────┬──────────┘
                                ▼
                     ┌──────────────────┐
                     │  Execute approved │
                     │  actions in order │
                     │  with thinking &  │
                     │  delays between   │
                     └──────────────────┘
```

## Features

### ✅ Completed
- [x] Python eliminated — everything in Rust + Go (zero Python dependency)
- [x] Multi-LLM provider abstraction (Ollama, Claude, Groq, Grok, Gemini) in Rust
- [x] Live model list fetching for all 5 providers via their REST APIs
- [x] Tauri desktop app with system tray and 6-tab dashboard
- [x] Policy engine with 3 risk profiles (High/Medium/Low)
- [x] Per-tool permissions (independently toggle shell, file, media, apps, WhatsApp)
- [x] Structured action types with propose → evaluate → execute flow
- [x] WhatsApp account allowlist + per-contact mode (Assistant/Chat/Summarize/Blocked)
- [x] GUI contacts table with search, allowlist toggle, mode dropdown
- [x] Built-in chat view with message history and live 3-second auto-polling
- [x] AI reply capability from chat view with Enter-to-send
- [x] Risk/approval system: AI-triggered tool calls with approve/reject in chat UI
- [x] Pending actions panel with Approve/Reject buttons and badge counter
- [x] Shell command executor with blocklist (enabled by default)
- [x] App launcher, volume control, media playback (macOS)
- [x] Desktop image scanner and file listing
- [x] Send files from desktop via WhatsApp (image/audio/video/document)
- [x] Send text messages via WhatsApp programmatically
- [x] Platform-aware AI prompts (macOS/Windows/Linux detected at startup)
- [x] WhatsApp-text approval flow — reply ALLOW/DENY to approve high-risk actions
- [x] Approval confirmation with action-specific messages (no false "success" for pending actions)
- [x] Profile picture fetching and caching via `/api/picture` bridge endpoint
- [x] App icon in sidebar replacing text logo
- [x] Manual start bridge button with error reporting
- [x] Session detection — shows logout option when a saved session exists but bridge is stopped
- [x] Post-logout auto-restart to show fresh QR code immediately
- [x] Reversible undo journal for all actions
- [x] Permanent WhatsApp auth via platform-native credential store (auto-save + restore)
- [x] Persistent policy config in credential store (allowlist, modes, permissions)
- [x] Configurable Ollama endpoint from GUI
- [x] API key management for cloud providers from GUI
- [x] API_KEY env var auth on Go bridge endpoints
- [x] Logout button to clear auth and keychain entries
- [x] WhatsApp MCP tools in Rust (SQLite reads + HTTP to Go bridge)
- [x] Setup.sh one-click bootstrap
- [x] Multi-platform CI + Release workflows (GitHub Actions)
- [x] MIT License with whatsapp-mcp attribution
- [x] Chat style modes — 9 predefined styles (Human, Fun, Warm, Teacher, Principal, Angry, Calm, AI, Robot) + custom user-defined styles via Settings
- [x] Multi-action AI responses — AI can chain multiple tool calls in a single response with thinking blocks and configurable time delays between them
- [x] Batch approval — "Approve All" / "Reject All" buttons in chat UI and "ALLOW ALL" / "DENY ALL" WhatsApp replies
- [x] WhatsApp ban warning banner — dismissible warning about using a secondary WhatsApp account

### 📋 Planned
- [ ] Scheduled/automated actions
- [ ] Multiple WhatsApp number support
- [ ] Voice message transcription
- [ ] Plugin system for custom tools

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Whatszara                                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌────────────────────────┐      ┌─────────────────────────────────┐    │
│  │   WhatsApp Layer        │      │    Tauri Desktop App (Rust)      │    │
│  │   (Go Bridge)           │─────▶│                                  │    │
│  │   - whatsmeow client    │      │  ┌──────────────────────────┐   │    │
│  │   - SQLite msg store    │      │  │  LLM Providers           │   │    │
│  │   - REST API :8080      │      │  │  - Ollama (live fetch)   │   │    │
│  │   - API_KEY auth        │      │  │  - Claude/Groq/Grok/Gem  │   │    │
│  └──────────┬──────────────┘      │  └──────────────────────────┘   │    │
│             │                     │                                  │    │
│             │                     │  ┌──────────────────────────┐   │    │
│   ┌─────────▼──────────┐         │  │  Policy Engine           │   │    │
│   │  SQLite (messages)  │◀────────│  │  - 3 risk profiles      │   │    │
│   │  + contacts table   │         │  │  - Per-tool permissions │   │    │
│   └─────────────────────┘         │  │  - Allowlist + modes    │   │    │
│                                   │  └──────────────────────────┘   │    │
│                                   │                                  │    │
│  ┌────────────────────────┐      │  ┌──────────────────────────┐   │    │
 │  │  Credential Store       │      │  │  Action Engine          │   │    │
│  │  - WA session (auto)   │      │  │  - Shell (enabled)      │   │    │
│  │  - Config (allowlist   │      │  │  - macOS: osascript     │   │    │
│  │    modes, perms)       │      │  │  - Volume / Media       │   │    │
│  └────────────────────────┘      │  │  - File scanner         │   │    │
│                                   │  │  - Undo journal        │   │    │
│                                   │  └──────────────────────────┘   │    │
│                                   │                                  │    │
│                                   │  ┌──────────────────────────┐   │    │
│                                   │  │  Risk/Approval System    │   │    │
│                                   │  │  - AI responds in JSON   │   │    │
│                                   │  │  - Pending actions queue │   │    │
│                                   │  │  - ALLOW/DENY via WhatsApp│   │    │
│                                   │  └──────────────────────────┘   │    │
│                                   │                                  │    │
│                                   │  ┌──────────────────────────┐   │    │
│                                   │  │  Frontend (HTML/JS)      │   │    │
│                                   │  │  - Dashboard + Wizard   │   │    │
│                                   │  │  - Chat view + polling  │   │    │
│                                   │  │  - Permissions table    │   │    │
│                                   │  │  - Provider config      │   │    │
│                                   │  │  - Settings + Keychain  │   │    │
│                                   │  └──────────────────────────┘   │    │
│                                   └─────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
```

## Quick Start

### Requirements

- **Go** for the WhatsApp bridge
- **Node.js 20+** and **Rust** for the Tauri desktop app
- **Ollama** or an API key for Claude, Groq, Grok, or Gemini
- **FFmpeg** optional, only needed for audio-message workflows

**Python is NOT required.**

### Install

```bash
chmod +x setup.sh && ./setup.sh
```

Or run the project setup target directly:

```bash
make setup
```

### Launch

```bash
make desktop
```

The desktop app opens with a modern setup wizard, live bridge status, light/dark/vibrant themes, keyboard shortcuts, and a built-in Guide tab for help.

## First-Run Setup

### 1. Connect WhatsApp

The app starts the bridge automatically from `whatsapp-bridge/`. When the QR code appears, open WhatsApp on your phone:

```text
Linked Devices -> Link a Device -> Scan QR
```

The dashboard shows bridge states in real time:

| Status | Meaning |
|--------|---------|
| `stopped` | The bridge is not running; "Start Bridge" button visible |
| `starting...` | The bridge process is booting |
| `scan QR` | WhatsApp needs device linking |
| `connected` | The bridge API is reachable and authenticated |
| `error` | The bridge failed; error detail shown in dashboard |
| Logout button | Visible when connected OR when stopped/errored with a saved session |

### 2. Choose an LLM Provider

Open **Providers**, choose the active provider, and refresh model lists. Ollama works locally; cloud providers require API keys.

```bash
export OLLAMA_ENDPOINT=http://localhost:11434
export ANTHROPIC_API_KEY=sk-ant-...
export GROQ_API_KEY=gsk-...
export XAI_API_KEY=xai-...
export GEMINI_API_KEY=AIza...
```

You can also paste provider keys into **Settings** and save local settings from the app.

### 3. Allowlist Contacts

Open **Permissions**, review contacts, and allowlist only trusted WhatsApp JIDs. Contact modes control behavior:

| Mode | Behavior |
|------|----------|
| **Assistant** | Can request desktop actions through the LLM |
| **Chat** | Text-only responses, no action execution |
| **Summarize** | Produces short summaries |
| **Blocked** | Rejected by policy |

### 4. Send a Message

Send a WhatsApp message to the connected account. Whatszara reads the message, asks the active model what to do, checks policy, and logs the result.

## Desktop UI Guide

### Themes

Use the sidebar theme switcher:

| Theme | Best for |
|-------|----------|
| **Dark** | Default focused workspace |
| **Light** | Bright rooms and screenshots |
| **Vibrant** | High-contrast colorful dashboard |

Theme choice is saved in local storage and restored on launch.

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl + 1` | Dashboard |
| `Cmd/Ctrl + 2` | Chat |
| `Cmd/Ctrl + 3` | Providers |
| `Cmd/Ctrl + 4` | Permissions |
| `Cmd/Ctrl + 5` | Action Log |
| `Cmd/Ctrl + 6` | Settings |
| `Cmd/Ctrl + 7` | Guide |
| `Cmd/Ctrl + K` | Focus search or reply |
| `Cmd/Ctrl + J` | Open Chat |
| `Cmd/Ctrl + G` or `?` | Open Guide |
| `Esc` | Clear and blur the focused input |

## Troubleshooting

| Problem | Fix |
|---------|-----|
| Bridge shows `stopped` | Confirm Go is installed with `go version`, then restart the app |
| Bridge shows `error` | Run `cd whatsapp-bridge && go run main.go` to see raw bridge logs |
| `go: not found` | Install Go from [go.dev](https://go.dev/dl/) and confirm it is in `PATH` |
| QR code does not appear | Remove `whatsapp-bridge/store/`, restart, and link again |
| QR scan fails repeatedly | Confirm your phone has internet and WhatsApp Linked Devices is available |
| Port `8080` is busy | Stop the other process or change the bridge port in `whatsapp-bridge/main.go` |
| Ollama models are missing | Run `ollama serve`, then `ollama pull llama3.1`, then refresh Providers |
| Cloud provider fails | Check the matching API key in Settings or shell environment |
| Messages do not execute | Check allowlist, contact mode, active provider, and tool permission toggles |
| Actions stay pending | Open Chat, select the contact, then approve or reject pending actions |

### Manual Bridge Mode

Use this when debugging bridge logs separately:

```bash
# Terminal 1
make bridge

# Terminal 2
make desktop
```

## Policy & Permission System

Whatszara uses a **propose → evaluate → execute** flow with risk-based approval.

### Approval via WhatsApp

When the AI requests Medium or High risk actions, an approval request is sent directly to your WhatsApp. For single actions:

```
⚠️ *Action Requires Approval*

Action: `execute_shell`
Risk: 🔴 *High*
Command: `ls -la ~/Desktop`

Reply *ALLOW* to execute or *DENY* to reject.
```

For multiple actions in a single response (AI chains them with thinking blocks):

```
⚠️ *Multiple Actions Require Approval*

1. 🔴 execute_shell High `ls -la ~/Desktop`
   _First, let me see what's on the desktop..._
2. 🔴 execute_shell High `df -h`
   _Now checking disk usage..._

Reply *ALLOW ALL* to execute all, *ALLOW <id>* for one, or *DENY ALL* to reject all.
```

Reply **ALLOW**, **ALLOW ALL**, **DENY**, or **DENY ALL** — no chat UI needed. The action(s) execute in order with thinking blocks and configured delays. You can also reply **ALLOW pa_1** to approve a specific action when multiple are pending. Send **"pending"** or **"list"** to see all pending actions.

### Risk Profiles

| Risk Level | Examples | Approval Required |
|-----------|----------|------------------|
| **Low** | Read volume, list files | None (auto-execute) |
| **Medium** | Open apps, play music, set volume | Reply ALLOW/DENY via WhatsApp |
| **High** | Shell commands, send files | Reply ALLOW/DENY via WhatsApp |

### Per-Tool Permissions

| Category | Default | Actions |
|----------|---------|---------|
| Shell | Enabled | `execute_shell`, `run_command` |
| File Access | Enabled | `list_files`, `list_images`, `get_desktop_paths` |
| Media Control | Enabled | `get_volume`, `set_volume`, `play`, `pause`, `next_track`, `prev_track` |
| App Launching | Enabled | `open_app` |
| WhatsApp | Enabled | `send_message`, `send_file`, `search_contacts` |

### Contact Modes

Every allowed contact has a mode:
- **Assistant** — Full AI control (tool calls + approve/reject)
- **Chat** — Text only, no desktop actions
- **Summarize** — 2-3 sentence summary (default)
- **Blocked** — Ignored at policy level

## Chat Styles

Whatszara lets you control the AI's tone and personality through chat styles. Each style is a short instruction injected into the system prompt. Choose from 9 predefined styles or write your own custom instructions.

| Style | Tone |
|-------|------|
| **Human** | Natural, conversational, human-like (default) |
| **Fun** | Playful, energetic, casual with humor |
| **Warm** | Friendly, caring, supportive like a close friend |
| **Teacher** | Instructive, educational, clear explanations |
| **Principal** | Firm, authoritative, no-nonsense |
| **Angry** | Frustrated, annoyed, short clipped sentences |
| **Calm** | Measured, soothing, patient and gentle |
| **AI** | Neutral, efficient, factual, minimal personality |
| **Robot** | Overly formal, literal, stilted like a robot |
| **Custom** | Any style you describe in free text |

Set the style in **Settings > Chat Style** — it persists via credential store and applies to both Chat and Assistant modes.

## Multi-Action AI Responses

The AI can execute **multiple actions in a single response** by returning an `actions` array instead of a single `tool` field. Each action can include:

- **`thinking`** — Internal reasoning text shown before executing the step
- **`delay_ms`** — Milliseconds to wait before executing (e.g., 2000 for 2 seconds)

This enables complex workflows like "list my desktop files, wait 3 seconds, then check disk space" — all from one AI response. Actions are processed sequentially: thinking is displayed, delay is applied, the action executes, then the next step begins.

## Batch Approval

When multiple actions require approval, the UI and WhatsApp approval flow support batch operations:

- **Chat UI** — "Approve All" / "Reject All" buttons appear in the Pending Actions panel when 2+ actions are pending
- **WhatsApp** — Reply **ALLOW ALL** or **DENY ALL** to handle all pending actions at once
- **Individual** — Still supported: reply **ALLOW pa_1** to approve a specific action

## Persistent Storage

| What | Where | How |
|------|-------|-----|
| WhatsApp session | Credential Store (`whatszara-wa-session`) | Auto-saved on first connect, auto-restored on launch |
| Policy config | Credential Store (`whatszara-config`) | Auto-saved on every change, manual load from Settings |
| App settings | Browser localStorage | Endpoint URLs, API keys |

## Chat View & AI Replies

The built-in chat view features:
- **Left panel**: Searchable contact list sorted by allowlisted status (allowlisted contacts first)
- **Right panel**: Message history with timestamps, auto-scroll to newest
- **3-second auto-polling** for live updates
- **Reply area**: Type a message, AI processes and sends response via WhatsApp. Only visible for allowlisted contacts
- **Pending actions panel**: Shows AI-triggered tool calls with Approve/Reject buttons. High-risk actions require approval before execution

## Credential Storage

Whatszara uses the **[keyring](https://github.com/hwchen/keyring-rs)** crate for cross-platform credential storage — no platform-specific code needed.

| Platform | Backend |
|----------|---------|
| macOS | iCloud Keychain (via Security framework) |
| Windows | Credential Manager (via wincred) |
| Linux | Secret Service / keyutils |

Two entries are stored with service name and username `whatszara`:

- **`whatszara-wa-session`** — WhatsApp session DB (base64-encoded). Auto-saved on first connect, restored on startup — no QR re-scan needed
- **`whatszara-config`** — Policy config (allowlist, contact modes, tool permissions). Auto-saved on every change, auto-restored on startup

**Logout**: Kills the bridge, deletes both credential entries, removes session file. Click "Logout & Disconnect" on the Dashboard.

## License

MIT Licensed. © 2026 Preet3627 (Latestinssan). The WhatsApp bridge incorporates code from [whatsapp-mcp](https://github.com/lharries/whatsapp-mcp) by Luke Harries.

## Documentation

Full docs site: **[github.com/Preet3627/whatszara-docs](https://github.com/Preet3627/whatszara-docs)**
