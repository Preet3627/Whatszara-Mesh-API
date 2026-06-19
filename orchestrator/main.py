#!/usr/bin/env python3
"""
Whatszara Orchestrator
Entry point that connects WhatsApp MCP to the LLM provider layer and action engine.
"""

import os
import sys
import json
import asyncio
from typing import Optional

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from llm import ProviderRegistry, LLMMessage
from llm.ollama import OllamaProvider
from llm.claude import ClaudeProvider
from llm.groq import GroqProvider
from llm.grok import GrokProvider
from llm.gemini import GeminiProvider
from permissions.engine import PermissionEngine
from permissions.risk_profiles import RiskLevel
from actions.shell import ShellExecutor
from actions.apps import AppLauncher
from actions.media import MediaController
from actions.desktop import DesktopScanner
from actions.registry import ActionRegistry
from undo.journal import ActionJournal


class WhatszaraOrchestrator:
    def __init__(self):
        self.providers = ProviderRegistry.create_default()
        self.permissions = PermissionEngine()
        self.shell = ShellExecutor()
        self.apps = AppLauncher()
        self.media = MediaController()
        self.desktop = DesktopScanner()
        self.journal = ActionJournal()
        self.action_registry = ActionRegistry()
        self._setup_actions()
        self._load_env_providers()

    def _setup_actions(self):
        self.action_registry.register("execute_shell", self.shell.execute)
        self.action_registry.register("open_app", self.apps.open)
        self.action_registry.register("get_volume", self.media.get_volume)
        self.action_registry.register("set_volume", self.media.set_volume)
        self.action_registry.register("play_music", self.media.play_music)
        self.action_registry.register("pause_music", self.media.pause_music)
        self.action_registry.register("next_track", self.media.next_track)
        self.action_registry.register("prev_track", self.media.prev_track)
        self.action_registry.register("list_images", self.desktop.list_images)
        self.action_registry.register("get_desktop_paths", self.desktop.get_desktop_paths)

    def _load_env_providers(self):
        if os.getenv("ANTHROPIC_API_KEY"):
            self.providers.register(ClaudeProvider(api_key=os.getenv("ANTHROPIC_API_KEY")))
        if os.getenv("GROQ_API_KEY"):
            self.providers.register(GroqProvider(api_key=os.getenv("GROQ_API_KEY")))
        if os.getenv("XAI_API_KEY"):
            self.providers.register(GrokProvider(api_key=os.getenv("XAI_API_KEY")))
        if os.getenv("GEMINI_API_KEY"):
            self.providers.register(GeminiProvider(api_key=os.getenv("GEMINI_API_KEY")))

        active = os.getenv("ACTIVE_PROVIDER")
        if active:
            try:
                self.providers.set_active(active)
            except KeyError:
                pass

    async def process_message(self, message: str, contact_jid: str, sender: str) -> str:
        history = [
            LLMMessage(role="system", content=(
                "You are Whatszara, a desktop assistant controlled via WhatsApp. "
                "You can execute shell commands, open applications, control volume and media playback, "
                "and list files on the desktop. Always ask for confirmation before destructive actions. "
                "Available tools: execute_shell, open_app, get_volume, set_volume, play_music, "
                "pause_music, next_track, prev_track, list_images, get_desktop_paths. "
                "Respond concisely and report results clearly."
            )),
            LLMMessage(role="user", content=message),
        ]

        try:
            response = await self.providers.chat(history)
            return response.content
        except Exception as e:
            return f"Error processing message: {str(e)}"

    async def handle_action(self, action: str, params: dict, contact_jid: str) -> dict:
        perm_request = self.permissions.evaluate(action, params, contact_jid)
        profile_name = perm_request.risk_level.value

        approved = self.permissions.approve(
            perm_request,
            captcha_passed=perm_request.risk_level == RiskLevel.LOW,
            recaptcha_score=1.0 if perm_request.risk_level == RiskLevel.LOW else None,
            confirmed=perm_request.risk_level == RiskLevel.LOW,
        )

        if not approved:
            return {
                "success": False,
                "error": f"Action requires verification",
                "requires": perm_request.requires_action,
                "risk_level": profile_name,
            }

        result = await self.action_registry.execute(action, **params)

        reverse = self._build_reverse_action(action, params, result)
        self.journal.record(
            action_type=action,
            parameters=params,
            result=result,
            reverse_action=reverse,
            risk_level=profile_name,
            contact_jid=contact_jid,
        )

        return result

    def _build_reverse_action(self, action: str, params: dict, result: dict) -> Optional[dict]:
        if action == "set_volume" and "volume" in params:
            return {"action": "set_volume", "params": {"level": 50}}
        if action == "execute_shell":
            return None
        if action == "play_music":
            return {"action": "pause_music", "params": {}}
        return None

    async def undo_last(self, contact_jid: str) -> dict:
        reversible = self.journal.get_reversible()
        for entry in reversible:
            if entry.contact_jid == contact_jid and entry.reverse_action:
                reverse = entry.reverse_action
                result = await self.action_registry.execute(reverse["action"], **reverse["params"])
                self.journal.mark_reversed(entry.action_id)
                return {"success": True, "undone_action": entry.action_type, "reverse_result": result}
        return {"success": False, "error": "No reversible actions found"}

    async def list_providers_with_models(self) -> list[dict]:
        return await self.providers.list_models()

    def status(self) -> dict:
        return {
            "active_provider": self.providers._active_provider if self.providers.active else None,
            "available_providers": self.providers.list_providers(),
            "available_actions": self.action_registry.list_tools(),
            "journal_entries": len(self.journal._entries),
            "reversible_actions": len(self.journal.get_reversible()),
        }


async def main():
    orch = WhatszaraOrchestrator()
    print(json.dumps(orch.status(), indent=2))
    print(f"\nActive provider: {orch.providers._active_provider}")

    print("\nListing providers and models...")
    models = await orch.list_providers_with_models()
    for entry in models:
        print(f"  {entry['provider']}: {', '.join(entry['models'][:5])}")

    print("\nOrchestrator ready. Connect via WhatsApp MCP to start.")


if __name__ == "__main__":
    asyncio.run(main())
