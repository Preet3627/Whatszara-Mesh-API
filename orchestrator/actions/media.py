import asyncio
import platform
from typing import Optional


class MediaController:
    def __init__(self):
        self.system = platform.system()
        self._current_volume: Optional[int] = None

    async def get_volume(self) -> dict:
        if self.system == "Darwin":
            proc = await asyncio.create_subprocess_exec(
                "osascript", "-e", "output volume of (get volume settings)",
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            stdout, _ = await proc.communicate()
            volume = int(stdout.decode().strip())
            return {"success": True, "volume": volume}
        return {"success": False, "error": "Unsupported platform"}

    async def set_volume(self, level: int) -> dict:
        level = max(0, min(100, level))
        if self.system == "Darwin":
            proc = await asyncio.create_subprocess_exec(
                "osascript", "-e", f"set volume output volume {level}",
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            _, stderr = await proc.communicate()
            if proc.returncode == 0:
                self._current_volume = level
                return {"success": True, "volume": level}
            return {"success": False, "error": stderr.decode()}
        return {"success": False, "error": "Unsupported platform"}

    async def play_music(self, query: Optional[str] = None) -> dict:
        if self.system == "Darwin":
            script = 'tell application "Music" to play'
            if query:
                script = f'tell application "Music" to play track "{query}"'
            proc = await asyncio.create_subprocess_exec(
                "osascript", "-e", script,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            _, stderr = await proc.communicate()
            if proc.returncode == 0:
                return {"success": True, "action": "play"}
            return {"success": False, "error": stderr.decode()}
        return {"success": False, "error": "Unsupported platform"}

    async def pause_music(self) -> dict:
        if self.system == "Darwin":
            proc = await asyncio.create_subprocess_exec(
                "osascript", "-e", 'tell application "Music" to pause',
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            _, stderr = await proc.communicate()
            if proc.returncode == 0:
                return {"success": True, "action": "pause"}
            return {"success": False, "error": stderr.decode()}
        return {"success": False, "error": "Unsupported platform"}

    async def next_track(self) -> dict:
        if self.system == "Darwin":
            proc = await asyncio.create_subprocess_exec(
                "osascript", "-e", 'tell application "Music" to next track',
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            _, stderr = await proc.communicate()
            if proc.returncode == 0:
                return {"success": True, "action": "next"}
            return {"success": False, "error": stderr.decode()}
        return {"success": False, "error": "Unsupported platform"}

    async def prev_track(self) -> dict:
        if self.system == "Darwin":
            proc = await asyncio.create_subprocess_exec(
                "osascript", "-e", 'tell application "Music" to previous track',
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            _, stderr = await proc.communicate()
            if proc.returncode == 0:
                return {"success": True, "action": "previous"}
            return {"success": False, "error": stderr.decode()}
        return {"success": False, "error": "Unsupported platform"}
