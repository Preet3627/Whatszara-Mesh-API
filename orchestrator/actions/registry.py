from typing import Any, Callable, Optional


class ActionRegistry:
    def __init__(self):
        self._tools: dict[str, Callable] = {}

    def register(self, name: str, func: Callable, description: str = "") -> None:
        self._tools[name] = func

    def get(self, name: str) -> Optional[Callable]:
        return self._tools.get(name)

    def list_tools(self) -> list[dict]:
        return [{"name": name, "func": func.__name__} for name, func in self._tools.items()]

    async def execute(self, name: str, **params) -> Any:
        func = self._tools.get(name)
        if func is None:
            return {"success": False, "error": f"Unknown action: {name}"}
        try:
            if asyncio.iscoroutinefunction(func):
                return await func(**params)
            return func(**params)
        except Exception as e:
            return {"success": False, "error": str(e)}


import asyncio
