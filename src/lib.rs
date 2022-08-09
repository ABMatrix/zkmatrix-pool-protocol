// extern crate core;
#[macro_use]
extern crate lazy_static;

use semver::Version;

pub mod message;

pub static PROTOCOL_PREFIX: &str = "ABMatrix";
pub static MIN_SUPPORTED_PROTOCOL_VERSION: Version = Version::new(0, 2, 0);
pub static MAX_SUPPORTED_PROTOCOL_VERSION: Version = Version::new(0, 2, 9);

lazy_static! {
    pub static ref CURRENT_PROTOCOL_VERSION: Version =
        Version::parse(env!("VERGEN_BUILD_SEMVER")).unwrap();
}

#[test]
fn get_current_version() {
    println!("{}", CURRENT_PROTOCOL_VERSION.to_string());
    println!("{}", CURRENT_PROTOCOL_VERSION.major);
    println!("{}", CURRENT_PROTOCOL_VERSION.minor);
    println!("{}", CURRENT_PROTOCOL_VERSION.patch);
    let v1 = Version::new(0, 0, 1);
    let v2 = Version::new(0, 2, 3);
    assert!(v1.lt(&CURRENT_PROTOCOL_VERSION));
    assert!(v2.gt(&CURRENT_PROTOCOL_VERSION))
}
