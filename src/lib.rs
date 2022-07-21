use semver::Version;

pub mod message;

pub static PROTOCOL_PREFIX: &'static str = "ABMatrix";
pub static MIN_SUPPORTED_PROTOCOL_VERSION: Version = Version::new(0, 1, 0);
pub static MAX_SUPPORTED_PROTOCOL_VERSION: Version = Version::new(0, 1, 0);