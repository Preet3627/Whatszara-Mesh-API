from enum import Enum


class RiskLevel(Enum):
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"


RISK_PROFILES: dict[RiskLevel, dict] = {
    RiskLevel.LOW: {
        "label": "Low Risk",
        "description": "Read-only or informational actions. Auto-approved and logged.",
        "requires_captcha": False,
        "requires_recaptcha": False,
        "requires_confirmation": False,
        "auto_approve": True,
        "color": "green",
    },
    RiskLevel.MEDIUM: {
        "label": "Medium Risk",
        "description": "Actions that affect system state but are non-destructive. Requires image-to-text verification.",
        "requires_captcha": True,
        "requires_recaptcha": False,
        "requires_confirmation": False,
        "auto_approve": False,
        "color": "yellow",
    },
    RiskLevel.HIGH: {
        "label": "High Risk",
        "description": "Destructive or security-sensitive actions. Requires reCAPTCHA + image-to-text + explicit confirmation.",
        "requires_captcha": True,
        "requires_recaptcha": True,
        "requires_confirmation": True,
        "auto_approve": False,
        "color": "red",
    },
}


ACTION_RISK_MAP: dict[str, RiskLevel] = {
    "get_volume": RiskLevel.LOW,
    "list_images": RiskLevel.LOW,
    "get_desktop_paths": RiskLevel.LOW,
    "get_system_info": RiskLevel.LOW,
    "open_app": RiskLevel.MEDIUM,
    "play_music": RiskLevel.MEDIUM,
    "pause_music": RiskLevel.MEDIUM,
    "next_track": RiskLevel.MEDIUM,
    "prev_track": RiskLevel.MEDIUM,
    "set_volume": RiskLevel.MEDIUM,
    "send_images": RiskLevel.MEDIUM,
    "send_file": RiskLevel.MEDIUM,
    "execute_shell": RiskLevel.HIGH,
    "install_software": RiskLevel.HIGH,
    "delete_file": RiskLevel.HIGH,
    "modify_system": RiskLevel.HIGH,
}


def get_risk_level(action: str) -> RiskLevel:
    return ACTION_RISK_MAP.get(action, RiskLevel.HIGH)


def get_profile(level: RiskLevel) -> dict:
    return RISK_PROFILES[level]
