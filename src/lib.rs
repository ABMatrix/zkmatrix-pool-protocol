use semver::Version;
use snarkvm::dpc::testnet2::Testnet2;

pub mod environment;
pub mod message;
pub mod speedometer;

pub static PROTOCOL_PREFIX: &'static str = "ABMatrix";
pub static MIN_SUPPORTED_PROTOCOL_VERSION: Version = Version::new(0, 1, 0);
pub static MAX_SUPPORTED_PROTOCOL_VERSION: Version = Version::new(0, 1, 0);
pub type PoolNetwork = Testnet2;