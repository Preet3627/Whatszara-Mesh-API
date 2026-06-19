// Whatszara Desktop App - Main

// ── Navigation ──
document.querySelectorAll(".nav-item").forEach((btn) => {
  btn.addEventListener("click", () => {
    document.querySelectorAll(".nav-item").forEach((b) => b.classList.remove("active"));
    btn.classList.add("active");
    const view = btn.dataset.view;
    document.querySelectorAll(".view").forEach((v) => v.classList.add("hidden"));
    const target = document.getElementById(`view-${view}`);
    if (target) target.classList.remove("hidden");
    if (view === "dashboard") refreshDashboard();
    if (view === "permissions") refreshPolicy();
    if (view === "actions") refreshActionLog();
    if (view === "providers") refreshModels();
  });
});

// ── Tauri invoke helper ──
async function invoke(cmd, args = {}) {
  if (window.__TAURI_INTERNALS__) {
    const { invoke } = window.__TAURI_INTERNALS__;
    return invoke(cmd, args);
  }
  console.warn("Tauri not available, returning mock");
  return JSON.stringify({ success: false, error: "Not running in Tauri" });
}

// ── Dashboard ──
async function refreshDashboard() {
  const badge = document.getElementById("status-badge");
  try {
    const raw = await invoke("get_status");
    const status = JSON.parse(raw);
    document.getElementById("llm-status").textContent = status.active_provider || "none";
    document.getElementById("actions-count").textContent = status.journal_entries || 0;
    if (badge) {
      badge.textContent = "connected";
      badge.classList.add("connected");
    }
  } catch {
    if (badge) {
      badge.textContent = "disconnected";
      badge.classList.remove("connected");
    }
  }
}

// ── Policy Management ──
async function refreshPolicy() {
  const raw = await invoke("get_policy");
  try {
    const policy = JSON.parse(raw);
    document.getElementById("allowlist-display").textContent =
      JSON.stringify(policy.allowlist || [], null, 2);
    document.getElementById("contact-modes-display").textContent =
      JSON.stringify(policy.contact_modes || {}, null, 2);
    document.getElementById("perm-shell").checked = policy.tool_permissions?.shell ?? false;
    document.getElementById("perm-file-access").checked = policy.tool_permissions?.file_access ?? true;
    document.getElementById("perm-media-control").checked = policy.tool_permissions?.media_control ?? true;
    document.getElementById("perm-app-launching").checked = policy.tool_permissions?.app_launching ?? true;
    document.getElementById("perm-whatsapp").checked = policy.tool_permissions?.whatsapp ?? true;
  } catch (e) {
    console.error("Failed to load policy", e);
  }
}

document.querySelectorAll("[data-perm]").forEach((cb) => {
  cb.addEventListener("change", async () => {
    const perm = cb.dataset.perm;
    const args = {};
    args[perm] = cb.checked;
    await invoke("update_permissions", args);
  });
});

document.getElementById("allowlist-add")?.addEventListener("click", async () => {
  const jid = document.getElementById("allowlist-jid").value.trim();
  if (!jid) return;
  await invoke("update_allowlist", { action: "add", jid });
  document.getElementById("allowlist-jid").value = "";
  refreshPolicy();
});

document.getElementById("allowlist-remove")?.addEventListener("click", async () => {
  const jid = document.getElementById("allowlist-jid").value.trim();
  if (!jid) return;
  await invoke("update_allowlist", { action: "remove", jid });
  document.getElementById("allowlist-jid").value = "";
  refreshPolicy();
});

document.getElementById("contact-mode-set")?.addEventListener("click", async () => {
  const jid = document.getElementById("contact-jid").value.trim();
  const mode = document.getElementById("contact-mode-select").value;
  if (!jid) return;
  await invoke("update_contact_mode", { jid, mode });
  document.getElementById("contact-jid").value = "";
  refreshPolicy();
});

// ── Providers ──
async function refreshModels() {
  try {
    const raw = await invoke("list_models");
    document.getElementById("models-list").textContent =
      JSON.stringify(JSON.parse(raw), null, 2);
  } catch {
    document.getElementById("models-list").textContent = "Failed to fetch models";
  }
}

document.getElementById("active-provider-select")?.addEventListener("change", async (e) => {
  await invoke("set_active_provider", { name: e.target.value });
});

// ── Action Log ──
async function refreshActionLog() {
  try {
    const raw = await invoke("get_status");
    const status = JSON.parse(raw);
    const tbody = document.getElementById("action-log-body");
    if (tbody) {
      tbody.innerHTML =
        `<tr><td>-</td><td>Journal: ${status.journal_entries || 0} entries</td><td>-</td><td>${status.reversible_actions || 0} reversible</td></tr>`;
    }
  } catch {
    // ignore
  }
}

document.getElementById("refresh-log")?.addEventListener("click", refreshActionLog);

// ── Settings ──
document.getElementById("save-settings")?.addEventListener("click", () => {
  const settings = {
    bridgeUrl: document.getElementById("bridge-url")?.value,
    apiKey: document.getElementById("api-key")?.value,
    ollamaEndpoint: document.getElementById("ollama-endpoint")?.value,
    activeProvider: document.getElementById("active-provider-select")?.value,
  };
  localStorage.setItem("whatszara-settings", JSON.stringify(settings));
  alert("Settings saved (local only for now)");
});

// ── Init ──
window.addEventListener("DOMContentLoaded", () => {
  const saved = localStorage.getItem("whatszara-settings");
  if (saved) {
    const settings = JSON.parse(saved);
    if (document.getElementById("bridge-url"))
      document.getElementById("bridge-url").value = settings.bridgeUrl || "http://localhost:8080";
    if (document.getElementById("ollama-endpoint"))
      document.getElementById("ollama-endpoint").value = settings.ollamaEndpoint || "http://localhost:11434";
    if (document.getElementById("active-provider-select"))
      document.getElementById("active-provider-select").value = settings.activeProvider || "ollama";
  }
  setTimeout(refreshDashboard, 1000);
});
