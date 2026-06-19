from abc import ABC, abstractmethod
from typing import Optional


class LLMMessage:
    def __init__(self, role: str, content: str):
        self.role = role
        self.content = content


class LLMResponse:
    def __init__(self, content: str, model: str, provider: str):
        self.content = content
        self.model = model
        self.provider = provider


class BaseLLMProvider(ABC):
    def __init__(self, name: str, api_key: Optional[str] = None, model: Optional[str] = None):
        self.name = name
        self.api_key = api_key
        self.model = model

    @abstractmethod
    async def chat(self, messages: list[LLMMessage], system_prompt: Optional[str] = None) -> LLMResponse:
        pass

    @abstractmethod
    async def list_models(self) -> list[str]:
        pass

    @abstractmethod
    async def is_available(self) -> bool:
        pass

    @abstractmethod
    def get_default_model(self) -> str:
        pass
