use std::f32;
use std::str::FromStr;
use anyhow::anyhow;
use rand::Rng;
use crate::message::stratum::StratumMessage;

pub fn convert_speed_from_msg(speed_msg: &StratumMessage) -> anyhow::Result<f32> {
    match speed_msg {
        StratumMessage::LocalSpeed(_, speed) => {
            match f32::from_str(speed.as_str()) {
                Ok(speed) => Ok(f32::from_str(format!("{:.2}", speed).as_str()).unwrap()),
                Err(e) => Err(anyhow!("{}, speed should be f32", e))
            }
        }
        _ => {
            Err(anyhow!("unsupported message"))
        }
    }
}

#[test]
fn test_convert_speed_from_msg() {
    use json_rpc_types::Id;
    let speed = rand::thread_rng().gen_range(1f32..10f32);
    let speed_s = speed.to_string();
    let split = speed_s.split('.').into_iter().map(|s| s.to_string()).collect::<Vec<String>>();
    let sp = {
        if split.len() != 2 {
            f32::from_str(split[0].as_str()).unwrap()
        } else {
            if split[1].len() <= 1 {
                f32::from_str(format!("{}.{}", &split[0], &split[1]).as_str()).unwrap()
            } else {
                let mut second = u8::from_str(&split[1][1..2]).unwrap();
                if second > 5 {
                    second += 1;
                }
                f32::from_str(format!("{}.{}{}", &split[0], &split[1][..1], second).as_str()).unwrap()
            }
        }
    };
    let msg = StratumMessage::LocalSpeed(Id::Num(1), speed.to_string());
    let i = convert_speed_from_msg(&msg).unwrap();
    println!("{}", i);
    assert_eq!(sp, i)
}