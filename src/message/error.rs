use anyhow::anyhow;
use std::str::FromStr;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PoolError {
    /// StaleProof usually occurs when the height is switched and the old proof is submitted
    StaleProof,
    /// InvalidProof(reason_message)
    InvalidProof(Option<String>),
    /// ServerNotReady usually occurs when the server is started
    ServerNotReady,
    InternalServerError,
}

impl ToString for PoolError {
    fn to_string(&self) -> String {
        match self {
            PoolError::StaleProof => "StaleProof".to_string(),
            PoolError::InvalidProof(reason) => {
                if reason.is_none() {
                    "InvalidProof".to_string()
                } else {
                    format!("InvalidProof{}", reason.clone().unwrap())
                }
            }
            PoolError::ServerNotReady => "ServerNotReady".to_string(),
            PoolError::InternalServerError => "InternalServerError".to_string(),
        }
    }
}

impl FromStr for PoolError {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with(&Self::StaleProof.name()) {
            Ok(Self::StaleProof)
        } else if s.starts_with(&Self::InvalidProof(None).name()) {
            if let Some(msg) = s.strip_prefix(&Self::InvalidProof(None).name()) {
                if msg.is_empty() {
                    Ok(Self::InvalidProof(None))
                } else {
                    Ok(Self::InvalidProof(Some(msg.to_string())))
                }
            } else {
                Ok(Self::InvalidProof(None))
            }
        } else if s.starts_with(&Self::InternalServerError.name()) {
            Ok(Self::InternalServerError)
        } else if s.starts_with(&Self::ServerNotReady.name()) {
            Ok(Self::ServerNotReady)
        } else {
            Err(anyhow!(format!("Unsupported message: {}", s)))
        }
    }
}

impl PoolError {
    pub fn id(&self) -> i64 {
        match self {
            PoolError::StaleProof => 1,
            PoolError::InvalidProof(..) => 2,
            PoolError::ServerNotReady => 3,
            PoolError::InternalServerError => 100,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            PoolError::StaleProof => "StaleProof",
            PoolError::InvalidProof(..) => "InvalidProof",
            PoolError::ServerNotReady => "ServerNotReady",
            PoolError::InternalServerError => "InternalServerError",
        }
    }
}

#[test]
fn test_pool_errors() {
    let e1 = PoolError::InternalServerError;
    let m1 = e1.to_string();
    let e1_r = PoolError::from_str(&m1).unwrap();
    assert_eq!(e1, e1_r);
    assert_eq!(&m1, e1.name());

    let e2 = PoolError::InvalidProof(None);
    let m2 = e2.to_string();
    let e2_r = PoolError::from_str(&m2).unwrap();
    assert_eq!(e2, e2_r);
    assert_eq!(&m2, e2.name());

    let e3 = PoolError::InvalidProof(Some("test error".to_string()));
    let m3 = e3.to_string();
    let e3_r = PoolError::from_str(&m3).unwrap();
    assert_eq!(e3, e3_r);
    assert_eq!(&m3, &format!("{}test error", e3.name()));

    let e4 = PoolError::StaleProof;
    let m4 = e4.to_string();
    let e4_r = PoolError::from_str(&m4).unwrap();
    assert_eq!(e4, e4_r);
    assert_eq!(&m4, e4.name());

    let e5 = PoolError::ServerNotReady;
    let m5 = e5.to_string();
    let e5_r = PoolError::from_str(&m5).unwrap();
    assert_eq!(e5, e5_r);
    assert_eq!(&m5, e5.name());

    let res = PoolError::from_str("test");
    assert!(res.is_err())
}
