use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};

static CAPTCHA_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaChallenge {
    pub id: String,
    pub text: String,
    pub contact_jid: String,
    pub action_id: String,
    pub attempts: u32,
    pub max_attempts: u32,
    pub verified: bool,
}

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
        matches!(self, RiskLevel::High)
    }
    pub fn requires_image_captcha(&self) -> bool {
        matches!(self, RiskLevel::High)
    }
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, RiskLevel::Medium | RiskLevel::High)
    }

    pub fn generate_captcha(&self, contact_jid: &str, action_id: &str) -> CaptchaChallenge {
        use rand::Rng;
        let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
        let text: String = (0..6).map(|_| {
            let idx = rand::thread_rng().gen_range(0..chars.len());
            chars[idx]
        }).collect();
        let counter = CAPTCHA_COUNTER.fetch_add(1, Ordering::Relaxed);
        CaptchaChallenge {
            id: format!("captcha_{}", counter),
            text,
            contact_jid: contact_jid.to_string(),
            action_id: action_id.to_string(),
            attempts: 0,
            max_attempts: 3,
            verified: false,
        }
    }
}

/// Generate a simple captcha PNG image as base64
pub fn render_captcha_image(text: &str) -> Vec<u8> {
    let width = 300;
    let height = 100;
    let mut img = image::RgbImage::new(width, height);

    for pixel in img.pixels_mut() {
        *pixel = image::Rgb([240, 240, 240]);
    }

    use image::Rgb;
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0..8 {
        let x1 = rng.gen_range(0..width);
        let y1 = rng.gen_range(0..height);
        let x2 = rng.gen_range(0..width);
        let y2 = rng.gen_range(0..height);
        let color = Rgb([rng.gen_range(100..200), rng.gen_range(100..200), rng.gen_range(100..200)]);
        draw_line(&mut img, x1, y1, x2, y2, color);
    }

    for _ in 0..200 {
        let x = rng.gen_range(0..width);
        let y = rng.gen_range(0..height);
        if x < width && y < height {
            img.put_pixel(x, y, Rgb([rng.gen_range(0..255), rng.gen_range(0..255), rng.gen_range(0..255)]));
        }
    }

    let char_width = width / (text.len() as u32 + 1);
    for (i, ch) in text.chars().enumerate() {
        let x = char_width * (i as u32 + 1) - 5;
        let y = 30 + rng.gen_range(0..20);
        let color = Rgb([rng.gen_range(30..200), rng.gen_range(30..200), rng.gen_range(30..200)]);
        draw_char(&mut img, ch, x, y, color);
    }

    let mut buf = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut buf, image::ImageFormat::Png)
        .ok();
    buf.into_inner()
}

fn draw_line(img: &mut image::RgbImage, x1: u32, y1: u32, x2: u32, y2: u32, color: image::Rgb<u8>) {
    let dx = (x2 as i32 - x1 as i32).abs();
    let dy = -(y2 as i32 - y1 as i32).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x1 as i32;
    let mut y = y1 as i32;
    loop {
        if x >= 0 && (x as u32) < img.width() && y >= 0 && (y as u32) < img.height() {
            img.put_pixel(x as u32, y as u32, color);
        }
        if x == x2 as i32 && y == y2 as i32 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x += sx; }
        if e2 <= dx { err += dx; y += sy; }
    }
}

fn draw_char(img: &mut image::RgbImage, _ch: char, x: u32, y: u32, color: image::Rgb<u8>) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let radius = 4;
    for dy in 0..=radius*2 {
        for dx in 0..=radius*2 {
            let px = x + dx;
            let py = y + dy;
            let dist = ((dx as i32 - radius).pow(2) + (dy as i32 - radius).pow(2)) as f64;
            if dist <= (radius * radius) as f64 {
                if px < img.width() && py < img.height() {
                    let _ = rng.gen_range(0..20);
                    img.put_pixel(px, py, color);
                }
            }
        }
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
    pub captcha_challenges: Vec<CaptchaChallenge>,
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
            captcha_challenges: Vec::new(),
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

    pub fn create_captcha(&mut self, contact_jid: &str, action_id: &str, risk: RiskLevel) -> CaptchaChallenge {
        let challenge = risk.generate_captcha(contact_jid, action_id);
        self.captcha_challenges.push(challenge.clone());
        challenge
    }

    pub fn verify_captcha(&mut self, captcha_id: &str, answer: &str) -> bool {
        if let Some(challenge) = self.captcha_challenges.iter_mut().find(|c| c.id == captcha_id) {
            challenge.attempts += 1;
            if challenge.text.to_lowercase() == answer.trim().to_lowercase() {
                challenge.verified = true;
                return true;
            }
            if challenge.attempts >= challenge.max_attempts {
                self.captcha_challenges.retain(|c| c.id != captcha_id);
            }
        }
        false
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
        if proposal.risk_level.requires_image_captcha() { requires.push("recaptcha".to_string()); }
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
