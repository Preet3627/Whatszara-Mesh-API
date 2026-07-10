# Whatszara — Desktop Assistant via WhatsApp + Mesh API

[![GitHub](https://img.shields.io/github/license/Preet3627/whatszara?style=flat-square&color=25D366)](LICENSE)

Control your desktop from anywhere using WhatsApp messages. Talk to an LLM through WhatsApp, and it executes your commands — shell, apps, media, file access — with a secure permission system and risk-based approval.

**Powered by [Mesh API](https://developers.meshapi.ai)** — a unified AI inference gateway that provides access to 300+ models (OpenAI, Anthropic, Groq, and more) through a single OpenAI-compatible endpoint. With BYOK (Bring Your Own Key) support, you can use your own upstream provider keys.

Built on top of **[whatsapp-mcp](https://github.com/lharries/whatsapp-mcp)** by Luke Harries, scaled from a simple MCP server into a full desktop automation platform.

**No Python required.** Everything is compiled into a single Tauri desktop app (Rust) + Go bridge for WhatsApp. No interpreters, no virtualenvs, no pip.

*Special thanks to the Mesh API team and Dhruv Rathee's team for making multi-model AI accessible to everyone.*

---

## How We Transformed Whatszara for Mesh API

Originally, Whatszara shipped with five separate LLM providers — Ollama (local), Claude, Groq, Grok, and Gemini — each with their own client implementation, authentication, model listing, and API quirks. Maintaining five parallel integrations meant duplicated code, inconsistent behavior, and a fragmented user experience.

### The Problem

- **Five API styles** — Ollama used `/api/chat`, Claude used Anthropic's Messages API, Grok and Groq followed OpenAI format differently, Gemini had its own generateContent format
- **Five auth methods** — API keys, bearer tokens, query params, custom headers, no auth for local
- **Five model lists** — each provider returned models in a different JSON structure
- **Key management** — users had to juggle multiple API keys, endpoints, and model names
- **No failover** — if one provider went down, you had to manually switch

### The Transformation

We replaced all five providers with a single **Mesh API** integration. Mesh API is an AI router that sits between Whatszara and the upstream AI providers. Instead of talking to five APIs, Whatszara now talks to one.

```
Before:  Whatszara ──▶ Ollama / Claude / Groq / Grok / Gemini
After:   Whatszara ──▶ Mesh API ──▶ OpenAI / Anthropic / Groq / etc.
                         (router)       (upstream providers)
```

This single change:

- **Eliminated 80% of provider code** — one LLMProvider implementation instead of five
- **Unified authentication** — a single RSK key for everything
- **Unified model listing** — one API call to list all 300+ available models
- **Enabled BYOK** — pass your own upstream keys for dedicated rate limits
- **Enabled model switching** — change models without changing providers
- **Simplified the UI** — from a dropdown of 5 providers to a single Mesh API config

The Mesh API provider implements the same LLMProvider trait that the old providers used, making the swap seamless. The orchestrator, policy engine, action engine, and frontend all work exactly as before — they just talk to Mesh API instead of five different backends.

### BYOK (Bring Your Own Key)

Mesh API lets you pass your own upstream provider keys. When configured, Mesh routes your requests through your own accounts:

| Key | Header | Provider |
|-----|--------|----------|
| MESH_OPENAI_KEY | x-mesh-openai-key | OpenAI (GPT-4o, GPT-4, etc.) |
| MESH_ANTHROPIC_KEY | x-mesh-anthropic-key | Anthropic (Claude) |
| MESH_GROQ_KEY | x-mesh-groq-key | Groq (Llama, Mixtral, etc.) |

This gives you dedicated rate limits and potentially lower costs compared to Mesh's shared pool.

### Quick Start with Mesh API

1. Create an account at [app.meshapi.ai](https://app.meshapi.ai)
2. Generate an RSK key from the **API Keys** section
3. Add credits via the **Billing** section
4. Set MESH_API_KEY=rsk_... in your environment
5. Launch Whatszara

---

## How It Works

WhatsApp Message → Orchestrator → Mesh API (300+ models via single API) → Policy/Risk Check → Action Execution

The AI returns structured JSON responses with optional action arrays. Each action goes through risk assessment — low risk auto-executes, high risk requires image captcha + user approval.

---

## Image-Based reCAPTCHA for High-Risk Actions

When an allowlisted assistant-mode user requests a **High Risk** action (like executing a shell command or deleting files), Whatszara generates a captcha image locally and sends it directly to **that specific contact** via WhatsApp.

### The Flow

1. **User sends a high-risk request** via WhatsApp (e.g., "run this shell command")
2. **Policy engine evaluates** the risk level and flags it as High Risk
3. **Captcha generated** — a PNG image with random alphanumeric characters is rendered locally (no external services, no API calls)
4. **Image sent to the requesting contact** — the captcha image is delivered as a WhatsApp image message to the exact same contact who requested the action
5. **User replies with captcha text** — they read the characters from the image and reply
6. **Verification** — the system checks their reply against the stored captcha
7. **Action executes** — only after correct captcha verification, 3 attempts max

This ensures the person requesting the action is physically present and reading WhatsApp. Everything runs locally — no third-party captcha services, no data leaving your machine.

### Why Local Captcha Generation

Unlike Google reCAPTCHA or hCaptcha, Whatszara's captcha is fully offline, has no tracking, no API costs, is customizable, and is session-scoped to a specific action and contact.

---

## AI Contact Management

Allowlisted assistant-mode users can ask the AI to manage their WhatsApp contacts naturally:

- **"Show my contacts"** — the AI calls the contact listing tool and returns all contacts
- **"Find John"** — the AI searches contacts by name or phone number
- **"Send message to John saying I'll be there in 10 minutes"** — the AI looks up John's contact info and sends the message

This works through the existing allowlist system with Assistant mode contacts.

---

## Features

### Completed
- Mesh API integration — single AI router for 300+ models
- BYOK (Bring Your Own Key) — upstream OpenAI/Anthropic/Groq keys
- Image-based reCAPTCHA for high-risk actions — locally generated, sent to requesting contact via WhatsApp
- AI contact management — list contacts, find by name, send messages
- Policy engine with 3 risk profiles (High/Medium/Low) and per-tool permissions
- WhatsApp account allowlist + per-contact mode (Assistant/Chat/Summarize/Blocked)
- Built-in chat view with live 3-second auto-polling
- Risk/approval system with approve/reject in chat UI and WhatsApp
- Shell command executor, app launcher, media control, file scanner
- Multi-action AI responses with thinking blocks and time delays
- Batch approval — Approve All / Reject All
- Chat style modes — 9 predefined + custom
- Persistent config in platform credential store

### Planned
- Scheduled/automated actions, multiple WhatsApp numbers, voice transcription, plugin system

---

## Setup

### Requirements
- Go, Node.js 20+, Rust
- Mesh API key at [app.meshapi.ai](https://app.meshapi.ai)

### Install & Launch
```bash
chmod +x setup.sh && ./setup.sh
make desktop
```

### Environment Variables
```bash
MESH_API_KEY=rsk_...
MESH_API_ENDPOINT=https://api.meshapi.ai/v1
MESH_OPENAI_KEY=sk-proj-...   # BYOK optional
MESH_ANTHROPIC_KEY=sk-ant-... # BYOK optional
MESH_GROQ_KEY=gsk_...         # BYOK optional
```

---

## License

MIT Licensed. 2026 Preet3627 (Latestinssan). Built on [whatsapp-mcp](https://github.com/lharries/whatsapp-mcp) by Luke Harries.

*Special thanks to the Mesh API team and Dhruv Rathee's team for making multi-model AI accessible to everyone.*
