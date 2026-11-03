//! Hardware-independent, offline configuration domain and profile repository.
//!
//! # Safety boundary
//! This crate does not open device handles, access operating-system device APIs, enumerate
//! hardware, communicate with firmware, or activate a transport. Application validation only
//! checks the local profile model; it never establishes device or firmware compatibility.
//!
//! # Public API and compatibility
//! Domain models are available under [`config`], evidence types under [`capability`], and
//! validation/repository operations under [`validation`] and [`repository`]. The crate is
//! currently `0.1.x`: public APIs may evolve between minor releases before `1.0.0`, with
//! migration notes documented for intentional breaking changes. JSON `schema_version` is an
//! independent persisted-data contract and is not derived from the crate version. Unsupported
//! future schema versions are rejected rather than reinterpreted.
//!
//! See the `offline_profile` example for construction, validation, serialization and persistence.
#![deny(unsafe_code)]

pub mod capability;
pub mod config;
pub mod device;
pub mod error;
pub mod repository;
pub mod validation;

pub use capability::FirmwareConstraintEvidence;
pub use config::{Configuration, Profile, ProfileId};
pub use device::{DeviceDescriptor, DeviceIdentity, DeviceKind};
pub use error::{CoreError, Diagnostic, DiagnosticDomain, DiagnosticSeverity, ValidationIssue};
pub use repository::ProfileRepository;
pub use validation::{ApplicationPolicy, APPLICATION_POLICY};

/// Current persisted profile JSON schema version. Independent of crate version.
pub const PROFILE_SCHEMA_VERSION: u32 = 1;
