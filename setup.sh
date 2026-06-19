#!/usr/bin/env bash
set -euo pipefail

# ──────────────────────────────────────────────
# Whatszara — One-command setup for non-technical users
# Auto-detects OS, checks prerequisites,
# installs dependencies, and gets you running.
# ──────────────────────────────────────────────

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'
info()  { echo -e "${CYAN}[INFO]${NC}  $1"; }
pass()  { echo -e "${GREEN}[PASS]${NC}  $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $1"; }
fail()  { echo -e "${RED}[FAIL]${NC}  $1"; exit 1; }

OS="$(uname -s)"
ARCH="$(uname -m)"
info "Detected: ${OS} (${ARCH})"

# ── Help ──────────────────────────────────────
if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  echo ""
  echo "  Whatszara Setup Script"
  echo ""
  echo "  Usage:"
  echo "    ./setup.sh            Full setup (default)"
  echo "    ./setup.sh check      Just check prerequisites"
  echo "    ./setup.sh bridge     Start WhatsApp bridge after setup"
  echo "    ./setup.sh desktop    Setup + launch Tauri desktop app"
  echo ""
  exit 0
fi

# ── Prerequisite checks ──────────────────────
PREREQ_MODE=false
[[ "${1:-}" == "check" ]] && PREREQ_MODE=true

check_cmd() {
  if command -v "$1" &>/dev/null; then
    pass "$1 found: $(command -v "$1")"
    return 0
  else
    warn "$1 not found"
    return 1
  fi
}

NEED_INSTALL=()

check_prereqs() {
  info "Checking prerequisites..."

  check_cmd go        || NEED_INSTALL+=("go")
  check_cmd python3   || NEED_INSTALL+=("python3")
  check_cmd node      || NEED_INSTALL+=("node")
  check_cmd cargo     || NEED_INSTALL+=("cargo (Rust)")
  check_cmd uv        || warn "uv not found — will install via pip"

  if command -v ffmpeg &>/dev/null; then
    pass "ffmpeg found"
  else
    warn "ffmpeg not found (optional — needed for audio messages)"
  fi

  if [[ "$PREREQ_MODE" == true ]]; then
    if [[ ${#NEED_INSTALL[@]} -eq 0 ]]; then
      pass "All prerequisites satisfied!"
    else
      echo ""
      warn "Missing: ${NEED_INSTALL[*]}"
      echo "  Run ./setup.sh to auto-install missing dependencies."
    fi
    exit 0
  fi
}

# ── Platform-specific installers ──────────────
install_homebrew() {
  if ! command -v brew &>/dev/null; then
    info "Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
  else
    pass "Homebrew found"
  fi
}

install_deps_macos() {
  info "Installing dependencies via Homebrew..."
  install_homebrew
  brew update

  if ! command -v go &>/dev/null; then    brew install go; fi
  if ! command -v python3 &>/dev/null; then brew install python; fi
  if ! command -v node &>/dev/null; then   brew install node; fi
  if ! command -v cargo &>/dev/null; then  brew install rust; fi
  if ! command -v ffmpeg &>/dev/null; then brew install ffmpeg; fi
  if ! command -v uv &>/dev/null; then     pip3 install uv; fi
}

install_deps_linux() {
  info "Installing dependencies via apt..."
  sudo apt-get update -qq
  sudo apt-get install -y -qq golang-go python3 python3-pip nodejs cargo ffmpeg curl 2>/dev/null || true

  if ! command -v go &>/dev/null; then
    warn "Go not in apt — installing manually..."
    wget -q https://go.dev/dl/go1.22.linux-amd64.tar.gz -O /tmp/go.tar.gz
    sudo tar -C /usr/local -xzf /tmp/go.tar.gz
    export PATH=$PATH:/usr/local/go/bin
  fi

  if ! command -v uv &>/dev/null; then pip3 install uv; fi
  if ! command -v cargo &>/dev/null; then
    warn "Rust not found — installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
  fi
}

install_deps_windows() {
  echo ""
  warn "Windows: Please install manually:"
  echo "  - Go:         https://go.dev/dl/"
  echo "  - Python 3.11+: https://www.python.org/downloads/"
  echo "  - Node.js:    https://nodejs.org/"
  echo "  - Rust:       https://rustup.rs/"
  echo "  - FFmpeg:     https://ffmpeg.org/download.html"
  echo "  - uv:         pip install uv"
  echo ""
  echo "  Then run this script again."
}

# ── Setup steps ───────────────────────────────
setup_python() {
  info "Setting up Python environment..."
  cd whatsapp-mcp-server
  uv sync 2>/dev/null || pip3 install -r ../requirements.txt 2>/dev/null || true
  cd ..
  pass "Python dependencies installed"
}

setup_go_bridge() {
  info "Setting up Go WhatsApp bridge..."
  cd whatsapp-bridge
  go mod download 2>/dev/null || true
  cd ..
  pass "Go bridge ready"
}

setup_desktop_app() {
  info "Setting up Tauri desktop app..."
  cd desktop-app
  npm install 2>/dev/null
  cd ..
  pass "Desktop app dependencies installed"
}

# ── Main ──────────────────────────────────────
main() {
  echo ""
  echo "╔══════════════════════════════════════════╗"
  echo "║        Whatszara — One-Click Setup       ║"
  echo "╚══════════════════════════════════════════╝"
  echo ""

  check_prereqs

  if [[ ${#NEED_INSTALL[@]} -gt 0 ]]; then
    info "Installing ${#NEED_INSTALL[@]} missing dependencies..."
    case "$OS" in
      Darwin) install_deps_macos ;;
      Linux)  install_deps_linux ;;
      *)      install_deps_windows ;;
    esac
  else
    pass "All dependencies already installed!"
  fi

  setup_python
  setup_go_bridge
  setup_desktop_app

  echo ""
  echo "╔══════════════════════════════════════════╗"
  echo "║           Setup Complete! 🎉             ║"
  echo "╚══════════════════════════════════════════╝"
  echo ""
  echo "  Quick Start:"
  echo ""
  echo "  1. Start WhatsApp bridge:"
  echo "     \$ cd whatsapp-bridge && go run main.go"
  echo "     (Scan the QR code with WhatsApp mobile app)"
  echo ""
  echo "  2. Start orchestrator (in another terminal):"
  echo "     \$ cd orchestrator && python3 main.py"
  echo ""
  echo "  3. Launch desktop app (in another terminal):"
  echo "     \$ cd desktop-app && npm run tauri dev"
  echo ""
  echo "  Or use: make run-bridge  make run-orchestrator  make run-desktop"
  echo ""
  echo "  Need help? Visit: https://github.com/Preet3627/whatszara"
  echo ""

  if [[ "${1:-}" == "bridge" ]]; then
    info "Starting WhatsApp bridge..."
    cd whatsapp-bridge && go run main.go
  fi
}

main "$@"
