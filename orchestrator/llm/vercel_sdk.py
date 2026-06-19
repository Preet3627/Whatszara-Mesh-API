from typing import Optional
from .base import BaseLLMProvider, LLMMessage, LLMResponse


class VercelAIProvider(BaseLLMProvider):
    def __init__(self, api_key: str, model: Optional[str] = None):
        super().__init__(name="vercel-ai-sdk", api_key=api_key, model=model or "gpt-4o")
        self.api_url = "https://api.vercel.ai/v1/chat/completions"

    async def chat(self, messages: list[LLMMessage], system_prompt: Optional[str] = None) -> LLMResponse:
        import httpx
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "content-type": "application/json",
        }
        payload = {
            "model": self.model,
            "messages": [{"role": m.role, "content": m.content} for m in messages],
        }
        if system_prompt:
            payload["messages"] = [{"role": "system", "content": system_prompt}] + payload["messages"]

        async with httpx.AsyncClient() as client:
            resp = await client.post(self.api_url, json=payload, headers=headers, timeout=120)
            resp.raise_for_status()
            data = resp.json()
            return LLMResponse(
                content=data["choices"][0]["message"]["content"],
                model=self.model,
                provider=self.name,
            )

    async def list_models(self) -> list[str]:
        return ["gpt-4o", "gpt-4o-mini", "gpt-4-turbo", "claude-sonnet-4-20250514", "claude-3-5-haiku"]

    async def is_available(self) -> bool:
        return bool(self.api_key)

    def get_default_model(self) -> str:
        return "gpt-4o"
