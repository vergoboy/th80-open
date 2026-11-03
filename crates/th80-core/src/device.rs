use crate::capability::{offline_capabilities, CapabilityDescriptor, FirmwareConstraintEvidence};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Known product model/role. This is identity metadata, not a protocol capability claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    Th80Pro,
    RoyuanReceiver,
}

/// Verified USB product identity for the keyboard or receiver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceIdentity {
    pub kind: DeviceKind,
    pub vendor_id: u16,
    pub product_id: u16,
}

impl DeviceIdentity {
    pub const TH80_PRO: Self = Self {
        kind: DeviceKind::Th80Pro,
        vendor_id: 0x3151,
        product_id: 0x4010,
    };
    pub const ROYUAN_RECEIVER: Self = Self {
        kind: DeviceKind::RoyuanReceiver,
        vendor_id: 0x3151,
        product_id: 0x4011,
    };
    /// Resolves only the two verified product IDs; unknown identities return `None`.
    pub fn from_vid_pid(vendor_id: u16, product_id: u16) -> Option<Self> {
        match (vendor_id, product_id) {
            (0x3151, 0x4010) => Some(Self::TH80_PRO),
            (0x3151, 0x4011) => Some(Self::ROYUAN_RECEIVER),
            _ => None,
        }
    }
}

/// Product identity plus explicitly optional, non-fabricated hardware metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Product identity with explicitly optional, non-fabricated hardware metadata.
pub struct DeviceDescriptor {
    pub identity: DeviceIdentity,
    pub hardware_revision: Option<String>,
    pub firmware_version: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<CapabilityDescriptor>,
    /// Evidence-backed firmware limits; empty when no constraint has been established.
    #[serde(default)]
    pub firmware_constraints: Vec<FirmwareConstraintEvidence>,
    /// Facts that have not been verified for this particular device instance.
    #[serde(default)]
    pub unverified_properties: BTreeMap<String, String>,
}

impl DeviceDescriptor {
    /// Creates an offline TH80 Pro descriptor without inventing revision or firmware values.
    pub fn th80_pro() -> Self {
        Self {
            identity: DeviceIdentity::TH80_PRO,
            hardware_revision: None,
            firmware_version: None,
            capabilities: offline_capabilities(),
            firmware_constraints: Vec::new(),
            unverified_properties: BTreeMap::from([
                (
                    "hardware_revision".into(),
                    "not established by current evidence".into(),
                ),
                (
                    "firmware_version".into(),
                    "not read from a physical device".into(),
                ),
                (
                    "configuration_protocol".into(),
                    "device-side semantics remain unverified".into(),
                ),
            ]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn does_not_fabricate_revision_or_firmware_and_keeps_communication_disabled() {
        let device = DeviceDescriptor::th80_pro();
        assert_eq!(device.identity, DeviceIdentity::TH80_PRO);
        assert!(device.hardware_revision.is_none());
        assert!(device.firmware_version.is_none());
        assert!(device
            .capabilities
            .iter()
            .all(|c| !c.device_communication_approved));
        assert!(device.firmware_constraints.is_empty());
        assert!(device
            .capabilities
            .iter()
            .all(|c| c.evidence == crate::capability::EvidenceClass::Unknown
                && c.evidence_sources.is_empty()));
        let unknown = FirmwareConstraintEvidence {
            feature_id: "macros".into(),
            constraint_id: "maximum_steps".into(),
            value: None,
            evidence: crate::capability::EvidenceClass::Unknown,
            sources: Vec::new(),
            rationale: "No firmware limit established".into(),
        };
        assert!(unknown.value.is_none());
        assert_eq!(unknown.evidence, crate::capability::EvidenceClass::Unknown);
    }
}
