from typing import Optional
from .base import BaseLLMProvider, LLMMessage, LLMResponse
from .ollama import OllamaProvider
from .claude import ClaudeProvider
from .groq import GroqProvider
from .grok import GrokProvider
from .gemini import GeminiProvider


class ProviderRegistry:
    def __init__(self):
        self._providers: dict[str, BaseLLMProvider] = {}
        self._active_provider: Optional[str] = None

    def register(self, provider: BaseLLMProvider, activate: bool = False) -> None:
        self._providers[provider.name] = provider
        if activate or self._active_provider is None:
            self._active_provider = provider.name

    def get(self, name: str) -> Optional[BaseLLMProvider]:
        return self._providers.get(name)

    @property
    def active(self) -> Optional[BaseLLMProvider]:
        if self._active_provider is None:
            return None
        return self._providers.get(self._active_provider)

    def set_active(self, name: str) -> None:
        if name in self._providers:
            self._active_provider = name
        else:
            raise KeyError(f"Provider '{name}' not registered")

    def list_providers(self) -> list[str]:
        return list(self._providers.keys())

    async def chat(self, messages: list[LLMMessage], system_prompt: Optional[str] = None) -> LLMResponse:
        provider = self.active
        if provider is None:
            raise RuntimeError("No active LLM provider configured")
        return await provider.chat(messages, system_prompt)

    async def list_models(self, provider_name: Optional[str] = None) -> list[dict]:
        results = []
        names = [provider_name] if provider_name else self._providers.keys()
        for name in names:
            provider = self._providers.get(name)
            if provider:
                try:
                    models = await provider.list_models()
                    results.append({"provider": name, "models": models})
                except Exception:
                    results.append({"provider": name, "models": [provider.get_default_model()]})
        return results

    @classmethod
    def create_default(cls) -> "ProviderRegistry":
        registry = cls()
        registry.register(OllamaProvider(), activate=True)
        return registry
