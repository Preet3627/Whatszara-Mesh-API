use super::llm::{self, LLMMessage, ProviderRegistry};
use super::policy::{ContactMode, PolicyEngine};
use super::actions::{ShellExecutor, AppLauncher, MediaController, DesktopScanner, ActionResult};
use super::undo::{ActionJournal, ReverseAction};
use super::whatsapp;
use serde_json;
use shellexpand;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActionStep {
    pub tool: String,
    pub params: HashMap<String, String>,
    #[serde(default)]
    pub thinking: Option<String>,
    #[serde(default)]
    pub delay_ms: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PendingAction {
    pub id: String,
    pub action: String,
    pub params: HashMap<String, String>,
    pub risk_level: String,
    pub requires: Vec<String>,
    pub contact_jid: String,
    pub reason: String,
    #[serde(default)]
    pub thinking: Option<String>,
    #[serde(default)]
    pub delay_ms: Option<u64>,
}

fn extract_params(val: &serde_json::Value) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Some(obj) = val.as_object() {
        for (k, v) in obj {
            match v {
                serde_json::Value::String(s) => { map.insert(k.clone(), s.clone()); }
                serde_json::Value::Number(n) => { map.insert(k.clone(), n.to_string()); }
                serde_json::Value::Bool(b) => { map.insert(k.clone(), b.to_string()); }
                other => { map.insert(k.clone(), other.to_string()); }
            }
        }
    }
    map
}

fn strip_code_fences(text: &str) -> String {
    let result = text.to_string();
    for prefix in &["```json\n", "```json\r\n", "```\n", "```\r\n"] {
        if let Some(start) = result.find(prefix) {
            if let Some(end) = result[start + prefix.len()..].find("```") {
                let extracted = result[start + prefix.len()..start + prefix.len() + end].trim().to_string();
                return extracted;
            }
        }
    }
    result
}

fn find_json_objects(text: &str) -> Vec<String> {
    let mut results = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            let mut depth: i32 = 0;
            let mut in_string = false;
            let mut escaped = false;
            let start = i;
            while i < bytes.len() {
                if escaped {
                    escaped = false;
                } else if bytes[i] == b'\\' && in_string {
                    escaped = true;
                } else if bytes[i] == b'"' {
                    in_string = !in_string;
                } else if !in_string {
                    match bytes[i] {
                        b'{' => depth += 1,
                        b'}' => {
                            depth -= 1;
                            if depth == 0 {
                                results.push(text[start..=i].to_string());
                                break;
                            }
                            if depth < 0 { break; }
                        }
                        _ => {}
                    }
                }
                i += 1;
            }
        }
        i += 1;
    }
    results
}

fn try_parse_value(val: &serde_json::Value) -> Option<String> {
    let obj = val.as_object()?;
    let chat = obj.get("chat").and_then(|c| c.as_str()).unwrap_or("").to_string();
    if !chat.is_empty() { Some(chat) } else { None }
}

fn try_parse_actions(val: &serde_json::Value) -> Vec<ActionStep> {
    let obj = match val.as_object() {
        Some(o) => o,
        None => return vec![],
    };

    if let Some(actions_arr) = obj.get("actions").and_then(|a| a.as_array()) {
        let mut steps = Vec::new();
        for item in actions_arr {
            if let Some(item_obj) = item.as_object() {
                let tool = item_obj.get("tool").and_then(|t| t.as_str()).unwrap_or("").to_string();
                if tool.is_empty() { continue; }
                let params = item_obj.get("params").map(|p| extract_params(p)).unwrap_or_default();
                let thinking = item_obj.get("thinking").and_then(|t| t.as_str()).map(|s| s.to_string());
                let delay_ms = item_obj.get("delay_ms").and_then(|d| d.as_u64());
                steps.push(ActionStep { tool, params, thinking, delay_ms });
            }
        }
        if !steps.is_empty() { return steps; }
    }

    if let Some(tool_name) = obj.get("tool").and_then(|t| t.as_str()) {
        if !tool_name.is_empty() {
            let params = obj.get("params").map(|p| extract_params(p)).unwrap_or_default();
            let thinking = obj.get("thinking").and_then(|t| t.as_str()).map(|s| s.to_string());
            let delay_ms = obj.get("delay_ms").and_then(|d| d.as_u64());
            return vec![ActionStep { tool: tool_name.to_string(), params, thinking, delay_ms }];
        }
    }

    vec![]
}

fn parse_ai_response(content: &str) -> (String, Vec<ActionStep>) {
    let no_fences = strip_code_fences(content.trim());
    let mut chat_out: String;

    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&no_fences) {
        let actions = try_parse_actions(&val);
        chat_out = try_parse_value(&val).unwrap_or_else(|| {
            if actions.is_empty() { no_fences.clone() } else { String::new() }
        });
        if !actions.is_empty() || (actions.is_empty() && chat_out.is_empty()) {
            return (chat_out, actions);
        }
    }

    for json_str in find_json_objects(&no_fences) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
            let actions = try_parse_actions(&val);
            chat_out = try_parse_value(&val).unwrap_or_default();
            if !actions.is_empty() || (actions.is_empty() && !chat_out.is_empty()) {
                return (chat_out, actions);
            }
        }
    }

    (no_fences, vec![])
}

pub struct WhatszaraOrchestrator {
    pub providers: ProviderRegistry,
    pub policy: PolicyEngine,
    pub shell: ShellExecutor,
    pub actions_journal: ActionJournal,
    pending_actions: Vec<PendingAction>,
    action_counter: u64,
    pub auto_read_enabled: bool,
    pub last_rowid: i64,
    pub poll_interval_ms: u64,
    platform: String,
}

impl WhatszaraOrchestrator {
    pub fn new() -> Self {
        let platform = if cfg!(target_os = "macos") {
            "macOS".to_string()
        } else if cfg!(target_os = "windows") {
            "Windows".to_string()
        } else if cfg!(target_os = "linux") {
            "Linux".to_string()
        } else {
            "Unknown".to_string()
        };
        Self {
            providers: ProviderRegistry::new(),
            policy: PolicyEngine::new(),
            shell: ShellExecutor::default(),
            actions_journal: ActionJournal::new(1000),
            pending_actions: Vec::new(),
            action_counter: 0,
            auto_read_enabled: false,
            last_rowid: 0,
            poll_interval_ms: 0,
            platform,
        }
    }

    pub fn pending_actions(&self) -> &[PendingAction] {
        &self.pending_actions
    }

    pub fn remove_pending_action(&mut self, id: &str) -> Option<PendingAction> {
        if let Some(idx) = self.pending_actions.iter().position(|a| a.id == id) {
            Some(self.pending_actions.remove(idx))
        } else {
            None
        }
    }

    pub fn register_default_providers(&mut self) {
        let api_key = std::env::var("MESH_API_KEY").unwrap_or_else(|_| {
            std::env::var("MESHAI_API_KEY").unwrap_or_default()
        });
        self.providers.register(Box::new(llm::MeshApiProvider {
            api_key,
            model: "openai/gpt-4o".into(),
            endpoint: std::env::var("MESH_API_ENDPOINT").unwrap_or_else(|_| "https://api.meshapi.ai/v1".into()),
            openai_key: std::env::var("MESH_OPENAI_KEY").unwrap_or_default(),
            anthropic_key: std::env::var("MESH_ANTHROPIC_KEY").unwrap_or_default(),
            groq_key: std::env::var("MESH_GROQ_KEY").unwrap_or_default(),
        }));
    }

    fn build_history(&self, message: &str, contact_jid: &str, limit: usize) -> Vec<LLMMessage> {
        let mut history = Vec::new();
        if let Ok(messages) = whatsapp::list_messages(contact_jid, limit) {
            for msg in &messages {
                let role = if msg.is_from_me { "assistant" } else { "user" };
                history.push(LLMMessage {
                    role: role.into(),
                    content: msg.content.clone(),
                });
            }
        }
        history.push(LLMMessage { role: "user".into(), content: message.into() });
        history
    }

    pub async fn process_message(&mut self, message: &str, contact_jid: &str) -> Result<serde_json::Value, String> {
        let mode = self.policy.get_contact_mode(contact_jid);

        match mode {
            ContactMode::Blocked => {
                return Ok(serde_json::json!({
                    "success": false,
                    "error": "This contact is blocked",
                    "contact_mode": "blocked",
                }));
            }
            ContactMode::Summarize => {
                let history = vec![
                    LLMMessage { role: "user".into(), content: message.into() },
                ];
                let system = "Summarize the following WhatsApp message concisely in 2-3 sentences. \
                    Do not execute any actions. Only return a summary of the message.";

                return match self.providers.chat(&history, Some(system)).await {
                    Ok(resp) => Ok(serde_json::json!({
                        "success": true,
                        "content": resp.content,
                        "model": resp.model,
                        "provider": resp.provider,
                        "contact_mode": "summarize",
                    })),
                    Err(e) => Ok(serde_json::json!({
                        "success": false,
                        "error": e,
                        "contact_mode": "summarize",
                    })),
                };
            }
            ContactMode::Chat => {
                let style_instr = self.chat_style_instruction();
                let warning = self.whatsapp_ban_warning();
                let history = self.build_history(message, contact_jid, 10);
                let system = format!("You are Whatszara, a WhatsApp-connected AI assistant. \
                    You can only chat and answer questions. You cannot execute any desktop actions. \
                    Respond helpfully and concisely.\n\n\
                    Current contact JID: {}\n\n\
                    Chat Style: {}\n{}",
                    contact_jid, style_instr, warning
                );

                return match self.providers.chat(&history, Some(&system)).await {
                    Ok(resp) => Ok(serde_json::json!({
                        "success": true,
                        "content": resp.content,
                        "model": resp.model,
                        "provider": resp.provider,
                        "contact_mode": "chat",
                    })),
                    Err(e) => Ok(serde_json::json!({
                        "success": false,
                        "error": e,
                        "contact_mode": "chat",
                    })),
                };
            }
            ContactMode::Assistant => {
                let platform = &self.platform;
                let style_instr = self.chat_style_instruction();
                let warning = self.whatsapp_ban_warning();
                let system = format!("                    You are Whatszara, a desktop assistant running on {}. \
                    You are currently chatting with JID: {}. \
                    This is the person who sent you the latest message. \
                    You are powered by Mesh API (https://developers.meshapi.ai) - \
                    an AI router that provides access to 300+ models.\n\n\
                    Available tools:\n\
                    - search_contacts: params {{query}} - Search contacts by name or phone\n\
                    - list_chats: params {{limit}} - List recent conversations\n\
                    - send_message: params {{recipient, message}} - Send WhatsApp message\n\
                    - execute_shell: params {{command}}\n\
                    - open_app: params {{name}}\n\
                    - get_volume, set_volume: params {{level}}\n\
                    - play_music: params {{query}}, pause_music, next_track, prev_track\n\
                    - list_images: params {{path}}, get_desktop_paths\n\
                    - send_file: params {{recipient, path, message}}\n\n\
                    On {} use `open` to launch apps, `osascript` for AppleScript. \
                    Shell commands run via `sh -c`. \
                    Never use code blocks or markdown. \n\n\
                    CONTACT MANAGEMENT (you are THE assistant - use these freely):\n\
                    - When user says \"list my contacts\" or \"show contacts\", use search_contacts with empty query or list_chats\n\
                    - When user says \"find <name>\", use search_contacts with that name\n\
                    - When user says \"send message to <name> saying <text>\", FIRST search_contacts to find their JID, then use send_message\n\
                    - You can also send messages to phone numbers directly\n\n\
                    IMPORTANT for send_message:\n\
                    - If user says \"send\" or \"send a message\" without specifying a recipient, \
                    send to the current contact: {}. \
                    - If user specifies a different name like \"send to John\", FIRST use search_contacts \
                    to find their JID, then send.\n\n\
                    {} \
                    {} \n\n\
                    YOUR OUTPUT MUST BE VALID JSON. This is the most important rule. \
                    Permissions are handled automatically — you don't need to ask. \
                    Just tell the user what you're doing and include the action JSON.\n\n\
                    Response formats:\n\n\
                    SINGLE ACTION:\n\
                    {{\"chat\": \"Opening Firefox...\", \"tool\": \"open_app\", \"params\": {{\"name\": \"Firefox\"}}}}\n\n\
                    MULTIPLE ACTIONS:\n\
                    {{\"chat\": \"Let me check your desktop\", \"actions\": [\
                    {{\"tool\": \"execute_shell\", \"params\": {{\"command\": \"ls -la ~/Desktop\"}}, \"thinking\": \"Checking desktop contents...\"}},\
                    {{\"tool\": \"execute_shell\", \"params\": {{\"command\": \"df -h\"}}, \"thinking\": \"Now checking disk space...\", \"delay_ms\": 2000}}\
                    ]}}\n\n\
                    CHAT ONLY:\n\
                    {{\"chat\": \"Hello! How can I help?\"}}\n\n\
                    CONTACT MANAGEMENT EXAMPLES:\n\
                    {{\"chat\": \"Here are your contacts\", \"tool\": \"search_contacts\", \"params\": {{\"query\": \"\"}}}}\n\
                    {{\"chat\": \"Finding John...\", \"tool\": \"search_contacts\", \"params\": {{\"query\": \"John\"}}}}\n\
                    {{\"chat\": \"Sending message to John...\", \"tool\": \"send_message\", \"params\": {{\"recipient\": \"1234567890@s.whatsapp.net\", \"message\": \"Hello from Whatszara!\"}}}}\n\n\
                    CRITICAL RULES:\n\
                    1. When the user asks you to do something, YOU MUST INCLUDE THE JSON TOOL CALL. \
                    Never just say \"okay\" or ask without the JSON. If you don't output JSON, nothing happens.\n\
                    2. Permissions are handled by the system automatically — just output the action JSON.\n\
                    3. NEVER respond with text-only when an action is needed. Always include the JSON.\n\
                    4. Low-risk actions run immediately. Others wait for approval — but you MUST propose them with JSON.\n\
                    5. If multiple steps needed, use the \"actions\" array with thinking/delay_ms.",
                    platform, contact_jid, platform, contact_jid, style_instr, warning
                );

                let history = self.build_history(message, contact_jid, 10);

                return match self.providers.chat(&history, Some(&system)).await {
                    Ok(resp) => {
                        let (chat_content, action_steps) = parse_ai_response(&resp.content);
                        let mut result = serde_json::json!({
                            "success": true,
                            "content": chat_content,
                            "model": resp.model,
                            "provider": resp.provider,
                            "contact_mode": "assistant",
                            "has_pending_actions": false,
                            "auto_executed": false,
                        });

                        if !action_steps.is_empty() {
                            let mut pending_list: Vec<PendingAction> = Vec::new();
                            let mut auto_results: Vec<serde_json::Value> = Vec::new();
                            let mut requires_approval = false;
                            let mut anything_blocked = false;

                            for step in &action_steps {
                                let (_proposal, decision) = self.policy.propose(
                                    &step.tool, step.params.clone(), contact_jid,
                                    "AI-triggered action from Assistant mode",
                                );
                                if decision.allowed {
                                    let thinking = step.thinking.clone();
                                    if let Some(t) = &thinking {
                                        auto_results.push(serde_json::json!({
                                            "type": "thinking", "text": t
                                        }));
                                    }
                                    if let Some(ms) = step.delay_ms {
                                        if ms > 0 {
                                            tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
                                        }
                                    }
                                    let action_result = self.execute_action(&step.tool, &step.params).await;
                                    let reverse = self.build_reverse(&step.tool, &step.params, &action_result);
                                    self.actions_journal.record(
                                        &step.tool, step.params.clone(),
                                        serde_json::to_value(&action_result).unwrap_or_default(),
                                        reverse, &decision.risk_level, contact_jid,
                                    );
                                    auto_results.push(serde_json::to_value(&action_result).unwrap_or_default());
                                } else if !decision.requires_verification.is_empty() {
                                    requires_approval = true;
                                    self.action_counter += 1;
                                    pending_list.push(PendingAction {
                                        id: format!("pa_{}", self.action_counter),
                                        action: step.tool.clone(),
                                        params: step.params.clone(),
                                        risk_level: decision.risk_level.clone(),
                                        requires: decision.requires_verification.clone(),
                                        contact_jid: contact_jid.to_string(),
                                        reason: decision.reason.clone(),
                                        thinking: step.thinking.clone(),
                                        delay_ms: step.delay_ms,
                                    });
                                } else {
                                    anything_blocked = true;
                                    auto_results.push(serde_json::json!({
                                        "type": "blocked",
                                        "action": step.tool,
                                        "reason": decision.reason,
                                    }));
                                }
                            }

                            if requires_approval {
                                for p in &pending_list {
                                    self.pending_actions.push(p.clone());
                                }
                                result["has_pending_actions"] = serde_json::json!(true);
                                result["pending_action"] = serde_json::to_value(&pending_list[0]).unwrap_or_default();
                                result["pending_actions"] = serde_json::to_value(&pending_list).unwrap_or_default();
                            }

                            if !auto_results.is_empty() {
                                result["auto_executed"] = serde_json::json!(true);
                                result["action_results"] = serde_json::to_value(&auto_results).unwrap_or_default();
                            }
                            result["anything_blocked"] = serde_json::json!(anything_blocked);
                        }
                        Ok(result)
                    }
                    Err(e) => Ok(serde_json::json!({ "success": false, "error": e })),
                };
            }
        }
    }

    pub async fn handle_action(&mut self, action: &str, params: HashMap<String, String>, contact: &str) -> serde_json::Value {
        let reason = format!("Action requested by {}", contact);
        let (_proposal, decision) = self.policy.propose(action, params.clone(), contact, &reason);

        if !decision.allowed {
            return serde_json::json!({
                "success": false,
                "error": decision.reason,
                "requires_verification": decision.requires_verification,
                "risk_level": decision.risk_level,
                "action": action,
            });
        }

        let result = self.execute_action(action, &params).await;

        let reverse = self.build_reverse(action, &params, &result);
        self.actions_journal.record(
            action, params, serde_json::to_value(&result).unwrap_or_default(),
            reverse, &decision.risk_level, contact,
        );

        serde_json::to_value(&result).unwrap_or_default()
    }

    async fn execute_action(&self, action: &str, params: &HashMap<String, String>) -> ActionResult {
        match action {
            "execute_shell" => {
                let cmd = params.get("command").map(|s| s.as_str()).unwrap_or("");
                self.shell.execute(cmd).await
            }
            "open_app" => {
                let name = params.get("name").map(|s| s.as_str()).unwrap_or("");
                AppLauncher::open(name).await
            }
            "get_volume" => MediaController::get_volume().await,
            "set_volume" => {
                let level: u8 = params.get("level").and_then(|v| v.parse().ok()).unwrap_or(50);
                MediaController::set_volume(level).await
            }
            "play_music" => {
                let query = params.get("query").map(|s| s.as_str());
                MediaController::play(query).await
            }
            "pause_music" => MediaController::pause().await,
            "next_track" => MediaController::next_track().await,
            "prev_track" => MediaController::prev_track().await,
            "list_images" => {
                let path = params.get("path").map(|s| s.as_str());
                DesktopScanner::list_images(path).await
            }
            "get_desktop_paths" => DesktopScanner::get_desktop_paths().await,
            "list_chats" => {
                let limit = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
                match whatsapp::list_chats(limit) {
                    Ok(chats) => {
                        let output = serde_json::to_string(&chats).unwrap_or_default();
                        ActionResult { success: true, output, error: String::new(), action: action.to_string(), params: params.clone() }
                    }
                    Err(e) => ActionResult::fail(action, &e),
                }
            }
            "search_contacts" => {
                let query = params.get("query").map(|s| s.as_str()).unwrap_or("");
                if query.is_empty() {
                    match whatsapp::list_all_contacts() {
                        Ok(contacts) => {
                            let output = serde_json::to_string(&contacts).unwrap_or_default();
                            ActionResult { success: true, output, error: String::new(), action: action.to_string(), params: params.clone() }
                        }
                        Err(e) => ActionResult::fail(action, &e),
                    }
                } else {
                    match whatsapp::search_contacts(query) {
                        Ok(contacts) => {
                            let output = serde_json::to_string(&contacts).unwrap_or_default();
                            ActionResult { success: true, output, error: String::new(), action: action.to_string(), params: params.clone() }
                        }
                        Err(e) => ActionResult::fail(action, &e),
                    }
                }
            }
            "send_message" | "send_file" => {
                Self::bridge_send(params.get("recipient").map(|s| s.as_str()).unwrap_or(""),
                    params.get("message").map(|s| s.as_str()).unwrap_or(""),
                    params.get("path").or_else(|| params.get("file_path")).map(|s| s.as_str()).unwrap_or(""),
                    action).await
            }
            _ => ActionResult { success: false, output: String::new(), error: format!("Unknown action: {}", action), action: action.to_string(), params: params.clone() },
        }
    }

    async fn bridge_send(recipient: &str, message: &str, media_path: &str, action: &str) -> ActionResult {
        if recipient.is_empty() {
            return ActionResult::fail(action, "recipient is required");
        }
        if message.is_empty() && media_path.is_empty() {
            return ActionResult::fail(action, "message or file path is required");
        }
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build();
        let client = match client {
            Ok(c) => c,
            Err(e) => return ActionResult::fail(action, &format!("Failed to create HTTP client: {}", e)),
        };
        let mut payload = serde_json::json!({ "recipient": recipient, "message": message });
        if !media_path.is_empty() {
            let expanded = shellexpand::tilde(media_path).to_string();
            payload["media_path"] = serde_json::json!(expanded);
        }
        let mut req = client.post("http://localhost:8080/api/send").json(&payload);
        if let Ok(key) = std::env::var("API_KEY") {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        match req.send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    ActionResult {
                        success: true,
                        output: format!("Sent to {}", recipient),
                        error: String::new(),
                        action: action.to_string(),
                        params: std::collections::HashMap::new(),
                    }
                } else {
                    ActionResult::fail(action, &format!("Bridge rejected: {}", resp.status()))
                }
            }
            Err(e) => ActionResult::fail(action, &format!("Bridge call failed: {}", e)),
        }
    }

    fn build_reverse(&self, action: &str, _params: &HashMap<String, String>, _result: &ActionResult) -> Option<ReverseAction> {
        match action {
            "set_volume" => Some(ReverseAction {
                action: "set_volume".into(),
                params: [("level".into(), "50".into())].into(),
            }),
            "play_music" => Some(ReverseAction {
                action: "pause_music".into(),
                params: HashMap::new(),
            }),
            _ => None,
        }
    }

    pub async fn undo_last(&mut self, contact: &str) -> serde_json::Value {
        let ids: Vec<String> = self.actions_journal.reversible().iter()
            .filter(|e| e.contact_jid == contact)
            .map(|e| e.action_id.clone())
            .take(1)
            .collect();

        if ids.is_empty() {
            return serde_json::json!({ "success": false, "error": "No reversible actions found" });
        }

        let action_id = &ids[0];
        let entry = self.actions_journal.get(action_id).cloned();
        if let Some(entry) = entry {
            if let Some(ref reverse) = entry.reverse_action.clone() {
                let mut p = HashMap::new();
                for (k, v) in &reverse.params {
                    p.insert(k.clone(), v.clone());
                }
                let result = self.execute_action(&reverse.action, &p).await;
                self.actions_journal.mark_reversed(action_id);
                return serde_json::json!({
                    "success": true,
                    "undone_action": entry.action_type,
                    "reverse_result": result,
                });
            }
        }
        serde_json::json!({ "success": false, "error": "No reversible actions found" })
    }

    pub async fn approve_pending_action(&mut self, id: &str) -> serde_json::Value {
        let pending = match self.remove_pending_action(id) {
            Some(p) => p,
            None => return serde_json::json!({"success": false, "error": "Pending action not found"}),
        };
        let (_proposal, decision) = self.policy.propose(
            &pending.action, pending.params.clone(), &pending.contact_jid, &pending.reason,
        );
        if !decision.allowed && decision.requires_verification.is_empty() {
            return serde_json::json!({
                "success": false, "error": decision.reason, "action": pending.action,
            });
        }
        let action_result = self.execute_action(&pending.action, &pending.params).await;
        let reverse = self.build_reverse(&pending.action, &pending.params, &action_result);
        self.actions_journal.record(
            &pending.action, pending.params,
            serde_json::to_value(&action_result).unwrap_or_default(),
            reverse, &decision.risk_level, &pending.contact_jid,
        );
        serde_json::json!({"success": true, "action_result": action_result})
    }

    pub fn reject_pending_action(&mut self, id: &str) -> serde_json::Value {
        match self.remove_pending_action(id) {
            Some(_) => serde_json::json!({"success": true, "rejected": true}),
            None => serde_json::json!({"success": false, "error": "Pending action not found"}),
        }
    }

    pub async fn approve_all_pending_for_contact(&mut self, contact_jid: &str) -> Vec<serde_json::Value> {
        let ids: Vec<String> = self.pending_actions.iter()
            .filter(|pa| pa.contact_jid == contact_jid)
            .map(|pa| pa.id.clone())
            .collect();
        let mut results = Vec::new();
        for id in ids {
            let result = self.approve_pending_action(&id).await;
            results.push(result);
        }
        results
    }

    pub fn reject_all_pending_for_contact(&mut self, contact_jid: &str) -> usize {
        let ids: Vec<String> = self.pending_actions.iter()
            .filter(|pa| pa.contact_jid == contact_jid)
            .map(|pa| pa.id.clone())
            .collect();
        let count = ids.len();
        for id in ids {
            self.reject_pending_action(&id);
        }
        count
    }

    pub fn check_pending_action_reply(&self, message: &str, contact_jid: &str) -> Option<(String, bool, PendingAction)> {
        let lower = message.trim().to_lowercase();

        let pending: Vec<&PendingAction> = self.pending_actions.iter()
            .filter(|pa| pa.contact_jid == contact_jid)
            .collect();

        if pending.is_empty() {
            return None;
        }

        let is_approval = if lower == "allow" || lower == "approve" || lower == "yes"
            || lower.starts_with("allow ") || lower.starts_with("approve ")
        {
            true
        } else if lower == "deny" || lower == "reject" || lower == "no"
            || lower.starts_with("deny ") || lower.starts_with("reject ") || lower.starts_with("no ")
        {
            false
        } else {
            return None;
        };

        let words: Vec<&str> = lower.split_whitespace().collect();
        let id_from_msg = if words.len() >= 2 { words[1] } else { "" };

        if id_from_msg == "all" {
            let first = pending.first().map(|pa| (*pa).clone());
            return first.map(|pa| ("__all__".to_string(), is_approval, pa));
        }

        let matched = if !id_from_msg.is_empty() {
            pending.iter().find(|pa| pa.id.contains(id_from_msg))
        } else {
            pending.first()
        };

        matched.map(|pa| (pa.id.clone(), is_approval, (*pa).clone()))
    }

    fn risk_icon(risk: &str) -> &'static str {
        match risk {
            "low" => "🟢",
            "medium" | "mid" => "🟡",
            _ => "🔴",
        }
    }

    fn risk_label(risk: &str) -> &'static str {
        match risk {
            "low" => "Low",
            "medium" | "mid" => "Mid",
            _ => "High",
        }
    }

    pub fn format_approval_request(pending: &PendingAction) -> String {
        let command = pending.params.get("command")
            .map(|c| format!("Command: `{}`\n", c))
            .unwrap_or_default();
        let name = pending.params.get("name")
            .map(|n| format!("App: `{}`\n", n))
            .unwrap_or_default();
        let details = if !command.is_empty() { command } else { name };
        let icon = Self::risk_icon(&pending.risk_level);
        let label = Self::risk_label(&pending.risk_level);
        let thinking = pending.thinking.as_ref()
            .map(|t| format!("_Thinking:_ {}\n", t))
            .unwrap_or_default();

        format!(
            "⚠️ *Action Requires Approval*\n\nAction: `{}`\nRisk: {} *{}*\n{}{}{}\n\nReply *ALLOW* to execute or *DENY* to reject.",
            pending.action, icon, label, details, thinking,
            if !pending.reason.is_empty() && pending.reason != "AI-triggered action from Assistant mode" {
                format!("Reason: {}\n", pending.reason)
            } else { String::new() }
        )
    }

    pub fn format_batch_approval_request(pending_list: &[PendingAction]) -> String {
        if pending_list.is_empty() {
            return "No pending actions.".to_string();
        }
        if pending_list.len() == 1 {
            return Self::format_approval_request(&pending_list[0]);
        }
        let mut msg = "⚠️ *Multiple Actions Require Approval*\n\n".to_string();
        for (i, pa) in pending_list.iter().enumerate() {
            let icon = Self::risk_icon(&pa.risk_level);
            let label = Self::risk_label(&pa.risk_level);
            let params_str = pa.params.get("command")
                .map(|c| format!(" `{}`", c))
                .or_else(|| pa.params.get("name").map(|n| format!(" `{}`", n)))
                .unwrap_or_default();
            let thinking = pa.thinking.as_ref()
                .map(|t| format!("\n   _{}_", t))
                .unwrap_or_default();
            msg.push_str(&format!(
                "{}. {} {} {}{}{}\n",
                i + 1, icon, pa.action, label, params_str, thinking
            ));
        }
        msg.push_str("\nReply *ALLOW ALL* to execute all, *ALLOW <id>* for one, or *DENY ALL* to reject all.");
        msg
    }

    pub fn format_approval_confirmation(pending: &PendingAction, action_result: &ActionResult) -> String {
        let icon = Self::risk_icon(&pending.risk_level);
        let label = Self::risk_label(&pending.risk_level);
        let summary = if !action_result.output.is_empty() {
            format!("```\n{}\n```", action_result.output.trim().chars().take(500).collect::<String>())
        } else if !action_result.error.is_empty() {
            format!("Error: {}", action_result.error.trim().chars().take(200).collect::<String>())
        } else {
            match pending.action.as_str() {
                "execute_shell" => "Command completed (no output)".to_string(),
                "open_app" => "Application launched".to_string(),
                "set_volume" => "Volume changed".to_string(),
                "play_music" => "Music playback started".to_string(),
                "pause_music" => "Music paused".to_string(),
                "next_track" => "Skipped to next track".to_string(),
                "prev_track" => "Went to previous track".to_string(),
                "send_message" => "Message sent".to_string(),
                "send_file" => "File sent".to_string(),
                _ => "Done".to_string(),
            }
        };
        format!(
            "✅ *Action Approved*\n\nAction: `{}`\nRisk: {} {}\n\n{}",
            pending.action, icon, label, summary,
        )
    }

    pub fn format_rejection_confirmation(pending: &PendingAction) -> String {
        let icon = Self::risk_icon(&pending.risk_level);
        let label = Self::risk_label(&pending.risk_level);
        format!(
            "❌ *Action Denied*\n\nAction: `{}`\nRisk: {} {}\n\nThe action was not executed.",
            pending.action, icon, label
        )
    }

    pub fn check_pending_action_query(&self, message: &str, contact_jid: &str) -> Option<String> {
        let lower = message.trim().to_lowercase();

        let has_pending = self.pending_actions.iter()
            .any(|pa| pa.contact_jid == contact_jid);

        let is_query = lower == "show" || lower == "list" || lower == "json"
            || lower == "commands" || lower == "pending"
            || lower == "show pending" || lower == "show json"
            || lower == "show commands" || lower == "show actions"
            || lower == "list pending" || lower == "list commands"
            || lower == "list actions" || lower == "pending actions"
            || lower == "pending commands" || lower == "what's pending"
            || lower == "what is pending" || lower == "what pending"
            || lower.starts_with("show ") || lower.starts_with("list ")
            || lower.contains("show pending") || lower.contains("show json")
            || lower.contains("show command") || lower.contains("show action")
            || lower.contains("list pending") || lower.contains("list command")
            || lower.contains("list action") || lower.contains("pending action")
            || lower.contains("pending command") || lower.contains("what pending")
            || lower.contains("what's pending") || lower.contains("what is pending");

        if !is_query {
            return None;
        }

        if !has_pending {
            return Some("No pending actions. Send a command and if the AI requests a high-risk action, it will appear here for approval.".to_string());
        }

        let pending: Vec<&PendingAction> = self.pending_actions.iter()
            .filter(|pa| pa.contact_jid == contact_jid)
            .collect();

        Some(Self::format_pending_actions_list(&pending))
    }

    pub fn format_pending_actions_list(pending: &[&PendingAction]) -> String {
        let mut response = String::from("📋 *Pending Actions*\n\n");
        for (i, pa) in pending.iter().enumerate() {
            let icon = Self::risk_icon(&pa.risk_level);
            let label = Self::risk_label(&pa.risk_level);
            let params_str = serde_json::to_string(&pa.params).unwrap_or_default();
            response.push_str(&format!(
                "{}. *{}*\n   {} Risk: {}\n   `{}`\n   ID: `{}`\n\n",
                i + 1, pa.action, icon, label, params_str, pa.id
            ));
        }
        response.push_str("Reply *ALLOW ALL* to execute all, *ALLOW <id>* for one, or *DENY ALL* to reject all.");
        response
    }

    pub fn start_auto_read(&mut self) {
        self.auto_read_enabled = true;
    }

    pub fn stop_auto_read(&mut self) {
        self.auto_read_enabled = false;
    }

    fn chat_style_instruction(&self) -> String {
        let style = &self.policy.chat_style;
        let base = match style.as_str() {
            "Human" => "Write in a natural, human-like conversational tone. Vary sentence length, use contractions, and sound like a real person chatting on WhatsApp.",
            "Fun" => "Write in a fun, playful, and energetic tone. Use casual language, occasional humor, and keep things light.",
            "Warm" => "Write in a warm, friendly, and caring tone. Be empathetic and supportive, like a close friend.",
            "Teacher" => "Write in an instructive, educational tone. Explain things clearly and provide helpful guidance, like a teacher.",
            "Principal" => "Write in a firm, authoritative tone. Be direct and no-nonsense, like a school principal addressing a student.",
            "Angry" => "Write in a frustrated, annoyed tone. Express irritation and impatience with short, clipped sentences.",
            "Calm" => "Write in a calm, measured, and soothing tone. Be patient, gentle, and reassuring.",
            "AI" => "Write in a neutral, efficient, direct tone. Be precise and factual like a typical AI assistant. Minimize personality.",
            "Robot" => "Write in a robotic, overly literal tone. Be excessively formal and stilted, like a malfunctioning robot.",
            custom => {
                if custom.is_empty() {
                    "Write in a natural, human-like conversational tone."
                } else {
                    return format!("Write in the following style: {}", custom);
                }
            }
        };
        base.to_string()
    }

    fn whatsapp_ban_warning(&self) -> &'static str {
        "\n\n⚠️ WARNING: Do NOT use your primary WhatsApp account with this app. \
        WhatsApp bans accounts that connect through unofficial clients like this bridge. \
        Use a secondary or temporary phone number. You have been warned.\n"
    }

    pub fn status(&self) -> serde_json::Value {
        let names = self.providers.list_names();
        let active = names.get(self.providers.active).cloned();
        serde_json::json!({
            "active_provider": active,
            "available_providers": names,
            "journal_entries": self.actions_journal.len(),
            "reversible_actions": self.actions_journal.reversible().len(),
            "policy": self.policy.to_json(),
            "auto_read_enabled": self.auto_read_enabled,
            "last_rowid": self.last_rowid,
            "poll_interval_ms": self.poll_interval_ms,
        })
    }
}
