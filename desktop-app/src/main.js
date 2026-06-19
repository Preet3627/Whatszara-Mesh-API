// Whatszara Desktop App - Main

// Navigation
document.querySelectorAll(".nav-item").forEach((btn) => {
  btn.addEventListener("click", () => {
    document.querySelectorAll(".nav-item").forEach((b) => b.classList.remove("active"));
    btn.classList.add("active");
    const view = btn.dataset.view;
    document.querySelectorAll(".view").forEach((v) => v.classList.add("hidden"));
    const target = document.getElementById(`view-${view}`);
    if (target) target.classList.remove("hidden");
  });
});

// Save settings
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

// Load saved settings
window.addEventListener("DOMContentLoaded", () => {
  const saved = localStorage.getItem("whatszara-settings");
  if (saved) {
    const settings = JSON.parse(saved);
    if (document.getElementById("bridge-url")) document.getElementById("bridge-url").value = settings.bridgeUrl || "http://localhost:8080";
    if (document.getElementById("ollama-endpoint")) document.getElementById("ollama-endpoint").value = settings.ollamaEndpoint || "http://localhost:11434";
    if (document.getElementById("active-provider-select")) document.getElementById("active-provider-select").value = settings.activeProvider || "ollama";
  }

  // Simulate status check
  setTimeout(() => {
    const badge = document.getElementById("status-badge");
    if (badge) {
      badge.textContent = "connected";
      badge.classList.add("connected");
    }
  }, 2000);
});

// Refresh action log (placeholder)
document.getElementById("refresh-log")?.addEventListener("click", () => {
  const tbody = document.getElementById("action-log-body");
  if (tbody) {
    tbody.innerHTML = "<tr><td colspan='4'>Connecting to orchestrator...</td></tr>";
  }
});
