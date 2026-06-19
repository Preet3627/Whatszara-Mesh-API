import httpx
from typing import Optional
from .base import BaseLLMProvider, LLMMessage, LLMResponse


class OllamaProvider(BaseLLMProvider):
    def __init__(self, endpoint: str = "http://localhost:11434", model: Optional[str] = None):
        super().__init__(name="ollama", model=model or "llama3")
        self.endpoint = endpoint.rstrip("/")

    async def chat(self, messages: list[LLMMessage], system_prompt: Optional[str] = None) -> LLMResponse:
        payload = {
            "model": self.model,
            "messages": [{"role": m.role, "content": m.content} for m in messages],
            "stream": False,
        }
        if system_prompt:
            payload["messages"] = [{"role": "system", "content": system_prompt}] + payload["messages"]

        async with httpx.AsyncClient() as client:
            resp = await client.post(f"{self.endpoint}/api/chat", json=payload, timeout=120)
            resp.raise_for_status()
            data = resp.json()
            return LLMResponse(content=data["message"]["content"], model=self.model, provider=self.name)

    async def list_models(self) -> list[str]:
        async with httpx.AsyncClient() as client:
            try:
                resp = await client.get(f"{self.endpoint}/api/tags", timeout=10)
                resp.raise_for_status()
                data = resp.json()
                return [m["name"] for m in data.get("models", [])]
            except Exception:
                return [self.get_default_model()]

    async def is_available(self) -> bool:
        try:
            async with httpx.AsyncClient() as client:
                resp = await client.get(f"{self.endpoint}/api/tags", timeout=5)
                return resp.status_code == 200
        except Exception:
            return False

    def get_default_model(self) -> str:
        return "llama3"
