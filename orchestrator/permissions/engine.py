from typing import Optional
from .risk_profiles import RiskLevel, get_risk_level, get_profile


class PermissionRequest:
    def __init__(
        self,
        action: str,
        parameters: dict,
        contact_jid: str,
        risk_level: RiskLevel,
    ):
        self.action = action
        self.parameters = parameters
        self.contact_jid = contact_jid
        self.risk_level = risk_level
        self.captcha_passed: Optional[bool] = None
        self.recaptcha_score: Optional[float] = None
        self.confirmed: Optional[bool] = None

    @property
    def is_approved(self) -> bool:
        profile = get_profile(self.risk_level)
        if profile["auto_approve"]:
            return True
        if self.risk_level == RiskLevel.MEDIUM:
            return self.captcha_passed is True
        if self.risk_level == RiskLevel.HIGH:
            return (
                self.captcha_passed is True
                and self.recaptcha_score is not None
                and self.recaptcha_score >= 0.5
                and self.confirmed is True
            )
        return False

    @property
    def requires_action(self) -> list[str]:
        profile = get_profile(self.risk_level)
        required = []
        if profile["requires_captcha"]:
            required.append("image_captcha")
        if profile["requires_recaptcha"]:
            required.append("recaptcha")
        if profile["requires_confirmation"]:
            required.append("confirmation")
        return required


class PermissionEngine:
    def __init__(self):
        self._contact_overrides: dict[str, dict[str, RiskLevel]] = {}

    def set_contact_action_risk(self, contact_jid: str, action: str, level: RiskLevel) -> None:
        if contact_jid not in self._contact_overrides:
            self._contact_overrides[contact_jid] = {}
        self._contact_overrides[contact_jid][action] = level

    def evaluate(self, action: str, parameters: dict, contact_jid: str) -> PermissionRequest:
        if contact_jid in self._contact_overrides and action in self._contact_overrides[contact_jid]:
            risk_level = self._contact_overrides[contact_jid][action]
        else:
            risk_level = get_risk_level(action)
        return PermissionRequest(action=action, parameters=parameters, contact_jid=contact_jid, risk_level=risk_level)

    def approve(self, request: PermissionRequest, captcha_passed: bool = False, recaptcha_score: Optional[float] = None, confirmed: bool = False) -> bool:
        request.captcha_passed = captcha_passed
        request.recaptcha_score = recaptcha_score
        request.confirmed = confirmed
        return request.is_approved
