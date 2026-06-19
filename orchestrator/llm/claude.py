import httpx
from typing import Optional
from .base import BaseLLMProvider, LLMMessage, LLMResponse


CLAUDE_MODELS = [
    "claude-sonnet-4-20250514",
    "claude-sonnet-4-20250514-sw60",
    "claude-4-opus-20250514",
    "claude-3-5-haiku-20241022",
    "claude-3-opus-20240229",
    "claude-3-sonnet-20240229",
    "claude-3-haiku-20240307",
]


class ClaudeProvider(BaseLLMProvider):
    def __init__(self, api_key: str, model: Optional[str] = None):
        super().__init__(name="claude", api_key=api_key, model=model or "claude-sonnet-4-20250514")
        self.api_url = "https://api.anthropic.com/v1/messages"

    async def chat(self, messages: list[LLMMessage], system_prompt: Optional[str] = None) -> LLMResponse:
        headers = {
            "x-api-key": self.api_key,
            "anthropic-version": "2023-06-01",
            "content-type": "application/json",
        }
        payload = {
            "model": self.model,
            "max_tokens": 4096,
            "messages": [{"role": m.role, "content": m.content} for m in messages],
        }
        if system_prompt:
            payload["system"] = system_prompt

        async with httpx.AsyncClient() as client:
            resp = await client.post(self.api_url, json=payload, headers=headers, timeout=120)
            resp.raise_for_status()
            data = resp.json()
            content = "".join(block["text"] for block in data["content"] if block["type"] == "text")
            return LLMResponse(content=content, model=self.model, provider=self.name)

    async def list_models(self) -> list[str]:
        return CLAUDE_MODELS[:]

    async def is_available(self) -> bool:
        return bool(self.api_key) and self.api_key != ""

    def get_default_model(self) -> str:
        return "claude-sonnet-4-20250514"
