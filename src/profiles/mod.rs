//! Obfuscator plugin detection and transformation registry.

#[allow(missing_docs)]
pub mod generic;
#[allow(missing_docs)]
pub mod ironbrew;
#[allow(missing_docs)]
pub mod luraph;
#[allow(missing_docs)]
pub mod moonsec;
#[allow(missing_docs)]
pub mod registry;

pub use generic::GenericProfile;
pub use ironbrew::IronbrewProfile;
pub use luraph::LuraphProfile;
pub use moonsec::MoonSecProfile;
pub use registry::{DetectionContext, ObfuscatorProfile, ProfileRegistry};
