import httpx
from typing import Optional
from .base import BaseLLMProvider, LLMMessage, LLMResponse


GROQ_MODELS = [
    "llama-3.3-70b-versatile",
    "llama-3.1-8b-instant",
    "mixtral-8x7b-32768",
    "gemma2-9b-it",
    "deepseek-r1-distill-llama-70b",
]


class GroqProvider(BaseLLMProvider):
    def __init__(self, api_key: str, model: Optional[str] = None):
        super().__init__(name="groq", api_key=api_key, model=model or "llama-3.3-70b-versatile")
        self.api_url = "https://api.groq.com/openai/v1/chat/completions"

    async def chat(self, messages: list[LLMMessage], system_prompt: Optional[str] = None) -> LLMResponse:
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
        headers = {"Authorization": f"Bearer {self.api_key}"}
        async with httpx.AsyncClient() as client:
            try:
                resp = await client.get("https://api.groq.com/openai/v1/models", headers=headers, timeout=10)
                resp.raise_for_status()
                data = resp.json()
                return [m["id"] for m in data.get("data", [])]
            except Exception:
                return GROQ_MODELS[:]

    async def is_available(self) -> bool:
        return bool(self.api_key)

    def get_default_model(self) -> str:
        return "llama-3.3-70b-versatile"
