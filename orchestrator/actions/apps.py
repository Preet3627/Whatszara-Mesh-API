import asyncio
import shutil
import platform
from typing import Optional

APP_ALIASES: dict[str, list[str]] = {
    "browser": ["google-chrome", "chrome", "firefox", "safari"],
    "terminal": ["terminal", "iterm", "alacritty", "kitty"],
    "code": ["code", "cursor", "vscode", "visual-studio-code"],
    "spotify": ["spotify"],
    "finder": ["finder", "nautilus", "dolphin"],
    "settings": ["system-settings", "gnome-control-center"],
}


class AppLauncher:
    def __init__(self):
        self.system = platform.system()

    async def open(self, app_name: str) -> dict:
        name = app_name.lower().strip()

        if name in APP_ALIASES:
            candidates = APP_ALIASES[name]
        else:
            candidates = [name]

        for candidate in candidates:
            if self.system == "Darwin":
                cmd = ["open", "-a", candidate]
            elif self.system == "Linux":
                cmd = [candidate]
            else:
                cmd = ["start", candidate]

            try:
                proc = await asyncio.create_subprocess_exec(
                    *cmd,
                    stdout=asyncio.subprocess.PIPE,
                    stderr=asyncio.subprocess.PIPE,
                )
                _, stderr = await proc.communicate()
                if proc.returncode == 0:
                    return {"success": True, "app": candidate}
            except FileNotFoundError:
                continue

        return {"success": False, "error": f"Could not find application: {app_name}"}

    async def list_installed(self) -> list[str]:
        if self.system == "Darwin":
            proc = await asyncio.create_subprocess_exec(
                "ls", "/Applications",
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            stdout, _ = await proc.communicate()
            return [name.replace(".app", "") for name in stdout.decode().split("\n") if name.endswith(".app")]
        return []
