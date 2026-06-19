import asyncio
import shlex
from typing import Optional


class ShellExecutor:
    def __init__(self, timeout: int = 30, max_output: int = 10000):
        self.timeout = timeout
        self.max_output = max_output
        self._allowed_commands: Optional[list[str]] = None
        self._blocked_commands: list[str] = ["sudo", "rm -rf /", "dd", "mkfs", ":(){ :|:& };:"]

    def set_allowed_commands(self, commands: list[str]) -> None:
        self._allowed_commands = commands

    def _is_blocked(self, command: str) -> bool:
        cmd_lower = command.lower().strip()
        for blocked in self._blocked_commands:
            if blocked.lower() in cmd_lower:
                return True
        if self._allowed_commands is not None:
            base = shlex.split(command)[0] if shlex.split(command) else ""
            return base not in self._allowed_commands
        return False

    async def execute(self, command: str) -> dict:
        if self._is_blocked(command):
            return {"success": False, "output": "", "error": "Command is blocked by security policy"}

        try:
            proc = await asyncio.create_subprocess_shell(
                command,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            try:
                stdout, stderr = await asyncio.wait_for(proc.communicate(), timeout=self.timeout)
            except asyncio.TimeoutError:
                proc.kill()
                await proc.wait()
                return {"success": False, "output": "", "error": f"Command timed out after {self.timeout}s"}

            output = stdout.decode(errors="replace")[:self.max_output]
            error = stderr.decode(errors="replace")[:self.max_output]
            return {
                "success": proc.returncode == 0,
                "output": output,
                "error": error,
                "return_code": proc.returncode,
            }
        except Exception as e:
            return {"success": False, "output": "", "error": str(e)}
