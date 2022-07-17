use std::str::FromStr;
use anyhow::anyhow;

#[derive(Clone, Debug, PartialEq)]
pub enum PoolError {
    StaleProof,
    InvalidProof(Option<String>),
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
                }else {
                    Ok(Self::InvalidProof(Some(msg.to_string())))
                }
            } else {
                Ok(Self::InvalidProof(None))
            }
        } else if s.starts_with(&Self::InternalServerError.name()) {
            Ok(Self::InternalServerError)
        } else {
            Err(anyhow!(format!("Unsupported message: {}",s)))
        }
    }
}

impl PoolError {
    pub fn name(&self) -> String {
        match self {
            PoolError::StaleProof => "StaleProof".to_string(),
            PoolError::InvalidProof(..) => "InvalidProof".to_string(),
            PoolError::InternalServerError => "InternalServerError".to_string(),
        }
    }
}

#[test]
fn test_pool_errors() {
    let e1 = PoolError::InternalServerError;
    let m1 = e1.to_string();
    let e1_r = PoolError::from_str(&m1).unwrap();
    assert_eq!(e1, e1_r);

    let e2 = PoolError::InvalidProof(None);
    let m2 = e2.to_string();
    let e2_r = PoolError::from_str(&m2).unwrap();
    assert_eq!(e2, e2_r);

    let e3 = PoolError::InvalidProof(Some("test error".to_string()));
    let m3 = e3.to_string();
    let e3_r = PoolError::from_str(&m3).unwrap();
    assert_eq!(e3, e3_r);

    let e4 = PoolError::StaleProof;
    let m4 = e4.to_string();
    let e4_r = PoolError::from_str(&m4).unwrap();
    assert_eq!(e4, e4_r);

    let res = PoolError::from_str("test");
    assert!(res.is_err())
}