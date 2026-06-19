import httpx
from typing import Optional
from .base import BaseLLMProvider, LLMMessage, LLMResponse


GEMINI_MODELS = [
    "gemini-2.5-flash-001",
    "gemini-2.5-pro-001",
    "gemini-2.0-flash",
    "gemini-1.5-pro",
    "gemini-1.5-flash",
]


class GeminiProvider(BaseLLMProvider):
    def __init__(self, api_key: str, model: Optional[str] = None):
        super().__init__(name="gemini", api_key=api_key, model=model or "gemini-2.5-flash-001")

    async def chat(self, messages: list[LLMMessage], system_prompt: Optional[str] = None) -> LLMResponse:
        url = f"https://generativelanguage.googleapis.com/v1beta/models/{self.model}:generateContent?key={self.api_key}"
        contents = [{"role": m.role, "parts": [{"text": m.content}]} for m in messages]

        payload = {"contents": contents}
        if system_prompt:
            payload["systemInstruction"] = {"parts": [{"text": system_prompt}]}

        async with httpx.AsyncClient() as client:
            resp = await client.post(url, json=payload, timeout=120)
            resp.raise_for_status()
            data = resp.json()
            text = ""
            for candidate in data.get("candidates", []):
                for part in candidate.get("content", {}).get("parts", []):
                    text += part.get("text", "")
            return LLMResponse(content=text, model=self.model, provider=self.name)

    async def list_models(self) -> list[str]:
        url = f"https://generativelanguage.googleapis.com/v1beta/models?key={self.api_key}"
        async with httpx.AsyncClient() as client:
            try:
                resp = await client.get(url, timeout=10)
                resp.raise_for_status()
                data = resp.json()
                return [m["name"].replace("models/", "") for m in data.get("models", []) if "generateContent" in m.get("supportedGenerationMethods", [])]
            except Exception:
                return GEMINI_MODELS[:]

    async def is_available(self) -> bool:
        return bool(self.api_key)

    def get_default_model(self) -> str:
        return "gemini-2.5-flash-001"
