use anyhow::anyhow;
use serde::{Serialize, Deserialize};


/// ProverSpeed: p/s
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct ProverSpeed {
    speed_1m: u32,
    speed_5m: u32,
    speed_15m: u32,
    speed_30m: u32,
    speed_60m: u32,
}

impl ProverSpeed {
    pub fn new(speed_1m: u32, speed_5m: u32, speed_15m: u32, speed_30m: u32, speed_60m: u32) -> Self {
        Self {
            speed_1m,
            speed_5m,
            speed_15m,
            speed_30m,
            speed_60m,
        }
    }

    pub fn speed_1m(&self) -> u32 {
        self.speed_1m
    }

    pub fn speed_5m(&self) -> u32 {
        self.speed_5m
    }

    pub fn speed_15m(&self) -> u32 {
        self.speed_15m
    }

    pub fn speed_30m(&self) -> u32 {
        self.speed_30m
    }

    pub fn speed_60m(&self) -> u32 {
        self.speed_60m
    }
}

impl ToString for ProverSpeed {
    fn to_string(&self) -> String {
        // match serde_json::to_string(self) {
        //     Ok(speed) => Ok(speed),
        //     Err(e) => anyhow!("convert ProverSpeed to string failed with error: {}", e)
        // }
        serde_json::to_string(self).unwrap_or_default()
    }
}

impl From<String> for ProverSpeed {
    fn from(value: String) -> Self {
        serde_json::from_str(value.as_str()).unwrap_or_default()
    }
}

#[test]
fn test_prover_speed() {
    let sp = ProverSpeed::new(2, 2, 3, 4, 5);
    let sp_s = sp.to_string();
    println!("{}", &sp_s);
    let speed = ProverSpeed::from(sp_s);
    assert_eq!(sp, speed)
}