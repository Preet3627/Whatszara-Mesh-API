import json
import time
from typing import Any, Optional
from datetime import datetime, timezone


class ActionEntry:
    def __init__(
        self,
        action_id: str,
        action_type: str,
        parameters: dict,
        result: dict,
        reverse_action: Optional[dict] = None,
        risk_level: str = "low",
        contact_jid: str = "",
    ):
        self.action_id = action_id
        self.action_type = action_type
        self.parameters = parameters
        self.result = result
        self.reverse_action = reverse_action
        self.risk_level = risk_level
        self.contact_jid = contact_jid
        self.timestamp = datetime.now(timezone.utc).isoformat()
        self.reversed = False
        self.reversed_at: Optional[str] = None

    def to_dict(self) -> dict:
        return {
            "action_id": self.action_id,
            "action_type": self.action_type,
            "parameters": self.parameters,
            "result": self.result,
            "reverse_action": self.reverse_action,
            "risk_level": self.risk_level,
            "contact_jid": self.contact_jid,
            "timestamp": self.timestamp,
            "reversed": self.reversed,
            "reversed_at": self.reversed_at,
        }


class ActionJournal:
    def __init__(self, max_entries: int = 1000):
        self._entries: list[ActionEntry] = []
        self._max_entries = max_entries
        self._counter = 0

    def record(
        self,
        action_type: str,
        parameters: dict,
        result: dict,
        reverse_action: Optional[dict] = None,
        risk_level: str = "low",
        contact_jid: str = "",
    ) -> ActionEntry:
        self._counter += 1
        action_id = f"act_{int(time.time())}_{self._counter}"
        entry = ActionEntry(
            action_id=action_id,
            action_type=action_type,
            parameters=parameters,
            result=result,
            reverse_action=reverse_action,
            risk_level=risk_level,
            contact_jid=contact_jid,
        )
        self._entries.append(entry)
        if len(self._entries) > self._max_entries:
            self._entries = self._entries[-self._max_entries:]
        return entry

    def mark_reversed(self, action_id: str) -> Optional[ActionEntry]:
        for entry in self._entries:
            if entry.action_id == action_id:
                entry.reversed = True
                entry.reversed_at = datetime.now(timezone.utc).isoformat()
                return entry
        return None

    def get(self, action_id: str) -> Optional[ActionEntry]:
        for entry in reversed(self._entries):
            if entry.action_id == action_id:
                return entry
        return None

    def get_recent(self, limit: int = 10) -> list[ActionEntry]:
        return list(reversed(self._entries[-limit:]))

    def get_reversible(self) -> list[ActionEntry]:
        return [e for e in reversed(self._entries) if not e.reversed and e.reverse_action is not None]

    def clear_expired(self, max_age_hours: int = 24) -> int:
        now = time.time()
        before = len(self._entries)
        self._entries = [
            e for e in self._entries
            if (now - datetime.fromisoformat(e.timestamp).timestamp()) < max_age_hours * 3600
        ]
        return before - len(self._entries)
