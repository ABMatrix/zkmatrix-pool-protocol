use std::str::FromStr;
use anyhow::anyhow;
use json_rpc_types::Id;
use crate::message::stratum::StratumMessage;

pub fn convert_speed_from_msg(speed_msg: &StratumMessage) -> anyhow::Result<u32> {
    match speed_msg {
        StratumMessage::LocalSpeed(_, speed) => {
            match u32::from_str(speed.as_str()) {
                Ok(speed) => Ok(speed),
                Err(e) => Err(anyhow!("{}, speed should be u32", e))
            }
        }
        _ => {
            Err(anyhow!("unsupported message"))
        }
    }
}

#[test]
fn test_convert_speed_from_msg() {
    let speed: u32 = 10;
    let msg = StratumMessage::LocalSpeed(Id::Num(1), speed.to_string());
    let i = convert_speed_from_msg(&msg).unwrap();
    assert_eq!(speed, i)
}