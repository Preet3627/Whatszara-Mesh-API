# Whatszara — Development Roadmap

> **Legend:** ✅ Done | 🔄 In Progress | 📋 Planned | 🔮 Future

---

## Phase 1: Foundation (Week 1-2)

### 1.1 Repository & Licensing
- [x] Clone and include [whatsapp-mcp](https://github.com/lharries/whatsapp-mcp) as base
- [x] MIT License with proper attribution (`LICENSE`, `LICENSE-THIRD-PARTY`)
- [x] Project structure scaffold
- [x] `.gitignore` for Python, Rust, Node, OS artifacts

### 1.2 Multi-LLM Provider Layer
- [x] Abstract `BaseLLMProvider` interface
- [x] Ollama provider with live model list fetching
- [x] Claude (Anthropic) provider
- [x] Groq provider
- [x] Grok (xAI) provider
- [x] Gemini (Google) provider
- [x] Vercel AI SDK integration
- [x] Provider registry and hot-swapping

### 1.3 Tauri Desktop App Shell
- [x] Tauri v2 project scaffold
- [x] Frontend framework setup (Svelte/React)
- [x] System tray integration
- [x] Settings window with provider config
- [x] QR code display for WhatsApp auth

### 1.4 Permission Engine Scaffold
- [x] Risk profile definitions (High/Medium/Low)
- [x] Permission resolution logic
- [x] Per-contact configuration model

---

## Phase 2: Core Action Engine (Week 3-4) 🔄

### 2.1 System Action Tools
- [ ] Shell command execution (`execute_shell`)
- [ ] App launcher (`open_app`)
- [ ] Volume get/set (`get_volume`, `set_volume`)
- [ ] Media playback control (`play_music`, `pause_music`, `next_track`, `prev_track`)
- [ ] Desktop file listing (`list_images`, `get_desktop_paths`)
- [ ] Bulk image sending (`send_images`)

### 2.2 Undo/Reversible Action System
- [ ] Action journal (SQLite-based event store)
- [ ] Reverse action registry
- [ ] `undo(action_id)` tool for LLM/user
- [ ] Automatic expiry & cleanup

---

## Phase 3: Permission & Security Layer (Week 5-6) 📋

### 3.1 reCAPTCHA Integration
- [ ] reCAPTCHA v3 score-based risk assessment
- [ ] Image-to-text CAPTCHA generation
- [ ] Tiered verification flow (auto / image-only / full)
- [ ] Session-based authentication tokens

### 3.2 Security Hardening
- [ ] Shell command whitelist/blacklist
- [ ] Command timeout enforcement
- [ ] Output size limits and truncation
- [ ] Path traversal protection
- [ ] Rate limiting
- [ ] SQLite encryption at rest

---

## Phase 4: Desktop Integration (Week 7-8) 📋

### 4.1 "Send All Images" Feature
- [ ] Desktop/Pictures directory scanning
- [ ] Smart deduplication
- [ ] Progress reporting via WhatsApp
- [ ] Batch sending with size limits

### 4.2 Advanced Media Playback
- [ ] Spotify/Apple Music integration
- [ ] Now-playing feedback via WhatsApp
- [ ] Playlist search and queue management

### 4.3 WhatsApp Incoming Message Webhook
- [ ] Message listener in Go bridge
- [ ] Real-time push to orchestrator
- [ ] Multi-user session handling

---

## Phase 5: Polish & Hardening (Week 9-10) 📋

### 5.1 Dashboard (Tauri Native)
- [ ] Live chat/activity view
- [ ] Action history with undo buttons
- [ ] Permission rule editor UI
- [ ] Risk profile visualizer
- [ ] System resource monitor
- [ ] Provider/model configuration UI

### 5.2 Testing & Documentation
- [ ] Unit tests for permission engine
- [ ] Integration tests for LLM providers
- [ ] E2E test: WhatsApp → LLM → Shell → Response
- [ ] Comprehensive user guide
- [ ] Video demo

---

## Future / Stretch Goals 🔮

- [ ] Voice message transcription (LLM processes audio)
- [ ] Scheduled actions ("remind me at 5 PM")
- [ ] Multiple WhatsApp number support
- [ ] Plugin system for custom tools
- [ ] End-to-end encryption audit
- [ ] Mobile companion app
- [ ] Cloud sync for permissions/config

---

## Scaling: From whatsapp-mcp to Whatszara

| Aspect | whatsapp-mcp | Whatszara |
|--------|-------------|-----------|
| **Source files** | 3 (main.go, main.py, whatsapp.py) | 20+ across 6 modules |
| **Components** | Go bridge + Python MCP server | + Orchestrator + Tauri app + Permission engine |
| **LLM support** | Claude only | 6 providers (Ollama, Claude, Groq, Grok, Gemini, Vercel) |
| **MCP tools** | 12 (read/send WhatsApp) | 12 WhatsApp + 10+ system action tools |
| **Security** | None | 3-tier risk, reCAPTCHA, image-to-text, undo journal |
| **Interface** | JSON config file | Full desktop GUI with tray, settings, logs |
| **Message direction** | LLM → WhatsApp | WhatsApp ↔ LLM ↔ Desktop (bidirectional) |
| **Lines of code** | ~1,200 | ~5,000+ |
