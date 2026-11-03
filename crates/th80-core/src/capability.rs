use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Sourced, evidence-graded capability statement. Offline editability is not device support.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityDescriptor {
    pub feature_id: String,
    pub support: SupportStatus,
    pub evidence: EvidenceClass,
    pub rationale: String,
    /// Source/report references supporting this evidence; empty for currently unknown claims.
    #[serde(default)]
    pub evidence_sources: Vec<String>,
    pub offline_editing: bool,
    /// Always false in Phase 10; retained as an explicit future authorization gate.
    pub device_communication_approved: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
/// Confidence grade of evidence, not an authorization to communicate.
pub enum EvidenceClass {
    Confirmed,
    StronglySupported,
    Hypothesis,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Status of a capability in the current application/evidence model.
pub enum SupportStatus {
    OfflineOnly,
    Partial,
    Unsupported,
    Unknown,
}

/// Evidence record for an actual firmware constraint. Unknown constraints have no value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FirmwareConstraintEvidence {
    pub feature_id: String,
    pub constraint_id: String,
    pub value: Option<Value>,
    pub evidence: EvidenceClass,
    #[serde(default)]
    pub sources: Vec<String>,
    pub rationale: String,
}

/// Returns generic offline-editable capabilities with unknown device evidence.
pub fn offline_capabilities() -> Vec<CapabilityDescriptor> {
    ["keymap", "macros", "lighting", "keyboard_options", "profiles"]
        .into_iter().map(|feature_id| CapabilityDescriptor {
            feature_id: feature_id.into(), support: SupportStatus::OfflineOnly,
            evidence: EvidenceClass::Unknown,
            rationale: "Representable and editable locally; firmware encoding and device behavior are not established.".into(),
            evidence_sources: Vec::new(),
            offline_editing: true, device_communication_approved: false,
        }).collect()
}
