use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolCategory {
    Shell,
    FileAccess,
    MediaControl,
    AppLaunching,
    WhatsApp,
}

impl ToolCategory {
    pub fn from_action(action: &str) -> Self {
        match action {
            "execute_shell" | "run_command" => ToolCategory::Shell,
            "list_files" | "list_images" | "send_file" | "get_desktop_paths" => ToolCategory::FileAccess,
            "get_volume" | "set_volume" | "play_media" | "pause_media"
                | "next_track" | "prev_track" => ToolCategory::MediaControl,
            "open_app" => ToolCategory::AppLaunching,
            "send_message" | "search_contacts" | "list_chats" => ToolCategory::WhatsApp,
            _ => ToolCategory::Shell,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPermissions {
    pub shell_enabled: bool,
    pub file_access_enabled: bool,
    pub media_control_enabled: bool,
    pub app_launching_enabled: bool,
    pub whatsapp_enabled: bool,
}

impl Default for ToolPermissions {
    fn default() -> Self {
        Self {
            shell_enabled: true,
            file_access_enabled: true,
            media_control_enabled: true,
            app_launching_enabled: true,
            whatsapp_enabled: true,
        }
    }
}

impl ToolPermissions {
    pub fn is_category_enabled(&self, cat: ToolCategory) -> bool {
        match cat {
            ToolCategory::Shell => self.shell_enabled,
            ToolCategory::FileAccess => self.file_access_enabled,
            ToolCategory::MediaControl => self.media_control_enabled,
            ToolCategory::AppLaunching => self.app_launching_enabled,
            ToolCategory::WhatsApp => self.whatsapp_enabled,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContactMode {
    Assistant,
    Chat,
    Summarize,
    Blocked,
}

impl Default for ContactMode {
    fn default() -> Self {
        Self::Summarize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl RiskLevel {
    pub fn requires_captcha(&self) -> bool {
        false
    }
    pub fn requires_recaptcha(&self) -> bool {
        false
    }
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, RiskLevel::Medium | RiskLevel::High)
    }
}

fn action_risk(action: &str) -> RiskLevel {
    match action {
        "get_volume" | "list_files" | "list_images" | "get_desktop_paths"
        | "list_chats" | "search_contacts" => RiskLevel::Low,
        "open_app" | "play_media" | "pause_media" | "next_track" | "prev_track"
        | "set_volume" | "send_message" | "send_file" => RiskLevel::Medium,
        "execute_shell" | "run_command" | "delete_file" | "install_software" => RiskLevel::High,
        _ => RiskLevel::High,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionProposal {
    pub action: String,
    pub params: HashMap<String, String>,
    pub tool_category: ToolCategory,
    pub risk_level: RiskLevel,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub allowed: bool,
    pub reason: String,
    pub requires_verification: Vec<String>,
    pub risk_level: String,
}

pub const PREDEFINED_STYLES: &[&str] = &[
    "Human", "Fun", "Warm", "Teacher", "Principal", "Angry", "Calm", "AI", "Robot",
];

pub struct PolicyEngine {
    pub tool_permissions: ToolPermissions,
    pub allowlist: HashSet<String>,
    pub contact_modes: HashMap<String, ContactMode>,
    pub chat_style: String,
    pub poll_interval_ms: u64,
}

impl Default for PolicyEngine {
    fn default() -> Self {
        let mut allowlist = HashSet::new();
        allowlist.insert("self".to_string());
        Self {
            tool_permissions: ToolPermissions::default(),
            allowlist,
            contact_modes: HashMap::new(),
            chat_style: "Human".to_string(),
            poll_interval_ms: 0,
        }
    }
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_chat_style(&mut self, style: &str) {
        self.chat_style = style.to_string();
    }

    pub fn set_contact_mode(&mut self, jid: &str, mode: ContactMode) {
        self.contact_modes.insert(jid.to_string(), mode);
    }

    pub fn get_contact_mode(&self, jid: &str) -> ContactMode {
        self.contact_modes.get(jid).cloned().unwrap_or(ContactMode::Summarize)
    }

    pub fn add_to_allowlist(&mut self, jid: &str) {
        self.allowlist.insert(jid.to_string());
    }

    pub fn remove_from_allowlist(&mut self, jid: &str) {
        self.allowlist.remove(jid);
    }

    pub fn is_allowed(&self, jid: &str) -> bool {
        self.allowlist.contains(jid)
    }

    pub fn evaluate_proposal(&self, proposal: &ActionProposal, contact_jid: &str) -> PolicyDecision {
        let mode = self.get_contact_mode(contact_jid);

        match mode {
            ContactMode::Blocked => {
                return PolicyDecision {
                    allowed: false,
                    reason: "This contact is blocked".into(),
                    requires_verification: vec![],
                    risk_level: "blocked".into(),
                };
            }
            ContactMode::Chat => {
                return PolicyDecision {
                    allowed: false,
                    reason: "Contact is in chat-only mode".into(),
                    requires_verification: vec![],
                    risk_level: "chat".into(),
                };
            }
            ContactMode::Summarize => {
                return PolicyDecision {
                    allowed: false,
                    reason: "Messages from this contact are summarized, not executed".into(),
                    requires_verification: vec![],
                    risk_level: "summarize".into(),
                };
            }
            ContactMode::Assistant => {}
        }

        if !self.is_allowed(contact_jid) {
            return PolicyDecision {
                allowed: false,
                reason: "Contact not in allowlist".into(),
                requires_verification: vec![],
                risk_level: "blocked".into(),
            };
        }

        if !self.tool_permissions.is_category_enabled(proposal.tool_category) {
            let cat_name = format!("{:?}", proposal.tool_category);
            return PolicyDecision {
                allowed: false,
                reason: format!("{} tools are disabled by policy", cat_name),
                requires_verification: vec![],
                risk_level: "policy_blocked".into(),
            };
        }

        let mut requires = vec![];
        if proposal.risk_level.requires_captcha() { requires.push("image_captcha".to_string()); }
        if proposal.risk_level.requires_recaptcha() { requires.push("recaptcha".to_string()); }
        if proposal.risk_level.requires_confirmation() { requires.push("confirmation".to_string()); }

        PolicyDecision {
            allowed: requires.is_empty(),
            reason: if requires.is_empty() { "Auto-approved".into() } else { "Requires verification".into() },
            requires_verification: requires,
            risk_level: format!("{:?}", proposal.risk_level).to_lowercase(),
        }
    }

    pub fn propose(&self, action: &str, params: HashMap<String, String>, contact_jid: &str, reason: &str) -> (ActionProposal, PolicyDecision) {
        let cat = ToolCategory::from_action(action);
        let risk = action_risk(action);
        let proposal = ActionProposal {
            action: action.to_string(),
            params,
            tool_category: cat,
            risk_level: risk,
            reason: reason.to_string(),
        };
        let decision = self.evaluate_proposal(&proposal, contact_jid);
        (proposal, decision)
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "tool_permissions": {
                "shell": self.tool_permissions.shell_enabled,
                "file_access": self.tool_permissions.file_access_enabled,
                "media_control": self.tool_permissions.media_control_enabled,
                "app_launching": self.tool_permissions.app_launching_enabled,
                "whatsapp": self.tool_permissions.whatsapp_enabled,
            },
            "allowlist": self.allowlist.iter().cloned().collect::<Vec<_>>(),
            "contact_modes": self.contact_modes.iter().map(|(k, v)| {
                (k.clone(), format!("{:?}", v).to_lowercase())
            }).collect::<HashMap<_, _>>(),
            "chat_style": self.chat_style,
            "predefined_styles": PREDEFINED_STYLES.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            "poll_interval_ms": self.poll_interval_ms,
        })
    }
}
