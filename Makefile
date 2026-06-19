.PHONY: setup check setup-macos setup-linux bridge orchestrator desktop build clean help

# ── Whatszara Makefile ────────────────────────
# One-command interface for common tasks.

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

setup: ## Full one-click setup (recommended for non-technical users)
	chmod +x setup.sh && ./setup.sh

check: ## Check prerequisites without installing
	chmod +x setup.sh && ./setup.sh check

setup-macos: ## Install deps on macOS via Homebrew
	chmod +x setup.sh && ./setup.sh

setup-linux: ## Install deps on Linux via apt
	chmod +x setup.sh && ./setup.sh

setup-windows: ## Print Windows install instructions
	@echo "Windows: install Go, Python, Node, Rust, FFmpeg manually."
	@echo "Then run: pip install uv && cd desktop-app && npm install"

bridge: ## Start WhatsApp bridge (scan QR code)
	cd whatsapp-bridge && go run main.go

orchestrator: ## Start the LLM orchestrator
	cd orchestrator && python3 main.py

desktop: ## Launch Tauri desktop app (dev mode)
	cd desktop-app && npm run tauri dev

build-desktop: ## Build desktop app for current platform
	cd desktop-app && npm run tauri build

build-macos: ## Build macOS app (must run on macOS)
	cd desktop-app && npm run tauri build -- --target universal-apple-darwin

build-linux: ## Build Linux app (must run on Linux)
	cd desktop-app && npm run tauri build

build-windows: ## Build Windows app (must run on Windows)
	cd desktop-app && npm run tauri build

run: bridge orchestrator desktop ## Start all three components
	@echo "Open three terminals and run: make bridge, make orchestrator, make desktop"

clean: ## Clean build artifacts
	rm -rf desktop-app/dist desktop-app/src-tauri/target
	rm -rf **/__pycache__ **/.venv
	find . -name "*.pyc" -delete
	find . -name "*.db" -delete
	@echo "Cleaned."
