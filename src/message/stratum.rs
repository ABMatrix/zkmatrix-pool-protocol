use super::response::ResponseMessage;
use bytes::BytesMut;
use json_rpc_types::{Error, Id, Request, Response, Version};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io;
use tokio_util::codec::{AnyDelimiterCodec, Decoder, Encoder};

pub enum StratumMessage {
    /// (id, user_agent, protocol_version, session_id)
    Subscribe(Id, String, String, Option<String>),

    /// (id, account_name, miner_name, worker_password)
    Authorize(Id, String, String, Option<String>),

    #[deprecated(since="0.2.0", note="difficulty_target will be sent with Notify")]
    /// This is the difficulty target for the next job.
    /// (difficulty_target)
    SetTarget(u64),

    /// New job from the mining pool.
    /// (job_id, difficulty_target, block_header_root, hashed_leaves_1, hashed_leaves_2, hashed_leaves_3,
    ///  hashed_leaves_4, clean_jobs)
    Notify(String, u64, String, String, String, String, String, bool),

    /// Submit shares to the pool.
    /// (id, job_id, nonce, proof)
    Submit(Id, String, String, String),

    /// (id, result, error)
    Response(Id, Option<ResponseMessage>, Option<Error<()>>),
}

impl StratumMessage {
    pub fn name(&self) -> &'static str {
        match self {
            StratumMessage::Subscribe(..) => "mining.subscribe",
            StratumMessage::Authorize(..) => "mining.authorize",
            StratumMessage::SetTarget(..) => "mining.set_target",
            StratumMessage::Notify(..) => "mining.notify",
            StratumMessage::Submit(..) => "mining.submit",
            StratumMessage::Response(..) => "mining.response",
        }
    }
}

pub struct StratumCodec {
    pub codec: AnyDelimiterCodec,
}

impl Default for StratumCodec {
    fn default() -> Self {
        Self {
            codec: AnyDelimiterCodec::new_with_max_length(vec![b'\n'], vec![b'\n'], 4096),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct NotifyParams(String, u64, String, String, String, String, String, bool);

#[derive(Serialize, Deserialize)]
struct SubscribeParams(String, String, Option<String>);

#[derive(Serialize, Deserialize)]
struct AuthorizeParams(String, String, Option<String>);

impl Encoder<StratumMessage> for StratumCodec {
    type Error = io::Error;

    fn encode(&mut self, item: StratumMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let bytes = match item {
            StratumMessage::Subscribe(id, user_agent, protocol_version, session_id) => {
                let request = Request {
                    jsonrpc: Version::V2,
                    method: "mining.subscribe",
                    params: Some(SubscribeParams(user_agent, protocol_version, session_id)),
                    id: Some(id),
                };
                serde_json::to_vec(&request).unwrap_or_default()
            }
            StratumMessage::Authorize(id, account_name, worker_name, worker_password) => {
                let request = Request {
                    jsonrpc: Version::V2,
                    method: "mining.authorize",
                    params: Some(AuthorizeParams(account_name, worker_name, worker_password)),
                    id: Some(id),
                };
                serde_json::to_vec(&request).unwrap_or_default()
            }
            StratumMessage::SetTarget(difficulty_target) => {
                let request = Request {
                    jsonrpc: Version::V2,
                    method: "mining.set_target",
                    params: Some(vec![difficulty_target]),
                    id: None,
                };
                serde_json::to_vec(&request).unwrap_or_default()
            }
            StratumMessage::Notify(
                job_id,
                difficulty_target,
                block_header_root,
                hashed_leaves_1,
                hashed_leaves_2,
                hashed_leaves_3,
                hashed_leaves_4,
                clean_jobs,
            ) => {
                let request = Request {
                    jsonrpc: Version::V2,
                    method: "mining.notify",
                    params: Some(NotifyParams(
                        job_id,
                        difficulty_target,
                        block_header_root,
                        hashed_leaves_1,
                        hashed_leaves_2,
                        hashed_leaves_3,
                        hashed_leaves_4,
                        clean_jobs,
                    )),
                    id: None,
                };
                serde_json::to_vec(&request).unwrap_or_default()
            }
            StratumMessage::Submit(id, job_id, nonce, proof) => {
                let request = Request {
                    jsonrpc: Version::V2,
                    method: "mining.submit",
                    params: Some(vec![job_id, nonce, proof]),
                    id: Some(id),
                };
                serde_json::to_vec(&request).unwrap_or_default()
            }
            StratumMessage::Response(id, result, error) => match error {
                Some(error) => {
                    let response = Response::<(), ()>::error(Version::V2, error, Some(id));
                    serde_json::to_vec(&response).unwrap_or_default()
                }
                None => {
                    let response = Response::<Option<ResponseMessage>, ()>::result(
                        Version::V2,
                        result,
                        Some(id),
                    );
                    serde_json::to_vec(&response).unwrap_or_default()
                }
            },
        };
        let string = std::str::from_utf8(&bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        self.codec
            .encode(string, dst)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        Ok(())
    }
}

impl Decoder for StratumCodec {
    type Item = StratumMessage;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let string = self
            .codec
            .decode(src)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        if string.is_none() {
            return Ok(None);
        }
        let bytes = string.unwrap();
        let json = serde_json::from_slice::<Value>(&bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        if !json.is_object() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Not an object"));
        }
        let object = json.as_object().unwrap();
        let result = if object.contains_key("method") {
            let request = serde_json::from_value::<Request<Vec<Value>>>(json)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            let id = request.id;
            let method = request.method.as_str();
            let params = match request.params {
                Some(params) => params,
                None => return Err(io::Error::new(io::ErrorKind::InvalidData, "No params")),
            };

            match method {
                "mining.subscribe" => {
                    if params.len() != 3 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params"));
                    }
                    let user_agent = unwrap_str_value(&params[0])?;
                    let protocol_version = unwrap_str_value(&params[1])?;
                    let session_id = match &params[2] {
                        Value::String(s) => Some(s),
                        Value::Null => None,
                        _ => {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Invalid params",
                            ));
                        }
                    };
                    StratumMessage::Subscribe(
                        id.unwrap_or(Id::Num(0)),
                        user_agent,
                        protocol_version,
                        session_id.cloned(),
                    )
                }
                "mining.authorize" => {
                    if params.len() != 3 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params"));
                    }
                    let account_name = unwrap_str_value(&params[0])?;
                    let miner_name = unwrap_str_value(&params[1])?;
                    let worker_password = match &params[2] {
                        Value::String(s) => Some(s),
                        Value::Null => None,
                        _ => {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Invalid params",
                            ));
                        }
                    };
                    StratumMessage::Authorize(
                        id.unwrap_or(Id::Num(0)),
                        account_name,
                        miner_name,
                        worker_password.cloned(),
                    )
                }
                "mining.set_target" => {
                    if params.len() != 1 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params"));
                    }
                    let difficulty_target = unwrap_u64_value(&params[0])?;
                    StratumMessage::SetTarget(difficulty_target)
                }
                "mining.notify" => {
                    if params.len() != 8 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params"));
                    }
                    let job_id = unwrap_str_value(&params[0])?;
                    let difficulty_target = unwrap_u64_value(&params[1])?;
                    let block_header_root = unwrap_str_value(&params[2])?;
                    let hashed_leaves_1 = unwrap_str_value(&params[3])?;
                    let hashed_leaves_2 = unwrap_str_value(&params[4])?;
                    let hashed_leaves_3 = unwrap_str_value(&params[5])?;
                    let hashed_leaves_4 = unwrap_str_value(&params[6])?;
                    let clean_jobs = unwrap_bool_value(&params[7])?;

                    StratumMessage::Notify(
                        job_id,
                        difficulty_target,
                        block_header_root,
                        hashed_leaves_1,
                        hashed_leaves_2,
                        hashed_leaves_3,
                        hashed_leaves_4,
                        clean_jobs,
                    )
                }
                "mining.submit" => {
                    if params.len() != 3 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params"));
                    }

                    let job_id = unwrap_str_value(&params[0])?;
                    let nonce = unwrap_str_value(&params[1])?;
                    let proof = unwrap_str_value(&params[2])?;

                    StratumMessage::Submit(id.unwrap_or(Id::Num(0)), job_id, nonce, proof)
                }
                _ => {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown method"));
                }
            }
        } else {
            let response = serde_json::from_value::<Response<ResponseMessage, ()>>(json)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            let id = response.id;
            match response.payload {
                Ok(payload) => {
                    StratumMessage::Response(id.unwrap_or(Id::Num(0)), Some(payload), None)
                }
                Err(error) => StratumMessage::Response(id.unwrap_or(Id::Num(0)), None, Some(error)),
            }
        };
        Ok(Some(result))
    }
}

fn unwrap_str_value(value: &Value) -> Result<String, io::Error> {
    match value {
        Value::String(s) => Ok(s.clone()),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Param is not str",
        )),
    }
}

fn unwrap_bool_value(value: &Value) -> Result<bool, io::Error> {
    match value {
        Value::Bool(b) => Ok(*b),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Param is not bool",
        )),
    }
}

fn unwrap_u64_value(value: &Value) -> Result<u64, io::Error> {
    match value {
        Value::Number(n) => Ok(n
            .as_u64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Param is not u64"))?),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Param is not u64",
        )),
    }
}

#[test]
fn test_encode_decode() {
    use crate::message::error::PoolError::InvalidProof;
    use json_rpc_types::ErrorCode;

    let mut codec = StratumCodec::default();
    //Subscribe
    let msg = StratumMessage::Subscribe(
        Id::Num(0),
        "ABMatrix_Aleo_Miner".to_string(),
        "ABMatrix_Aleo_Miner_4".to_string(),
        Some("session".to_string()),
    );
    let mut buf1 = BytesMut::new();
    codec.encode(msg, &mut buf1).unwrap();
    let res = codec.decode(&mut buf1.clone()).unwrap().unwrap();
    let mut buf2 = BytesMut::new();
    codec.encode(res, &mut buf2).unwrap();
    assert_eq!(buf1, buf2);

    //Authorize
    let msg = StratumMessage::Authorize(
        Id::Num(0),
        "account_name".to_string(),
        "worker_name".to_string(),
        None,
    );
    let mut buf1 = BytesMut::new();
    codec.encode(msg, &mut buf1).unwrap();
    let res = codec.decode(&mut buf1.clone()).unwrap().unwrap();
    let mut buf2 = BytesMut::new();
    codec.encode(res, &mut buf2).unwrap();
    assert_eq!(buf1, buf2);

    // SetTarget
    let msg = StratumMessage::SetTarget(100);
    let mut buf1 = BytesMut::new();
    codec.encode(msg, &mut buf1).unwrap();
    let res = codec.decode(&mut buf1.clone()).unwrap().unwrap();
    let mut buf2 = BytesMut::new();
    codec.encode(res, &mut buf2).unwrap();
    assert_eq!(buf1, buf2);

    //Notify
    let msg = StratumMessage::Notify(
        "job_id".to_string(),
        u64::MAX / 2,
        "block_header_root".to_string(),
        "hashed_leaves_1".to_string(),
        "hashed_leaves_2".to_string(),
        "hashed_leaves_3".to_string(),
        "hashed_leaves_4".to_string(),
        false,
    );
    let mut buf1 = BytesMut::new();
    codec.encode(msg, &mut buf1).unwrap();
    let res = codec.decode(&mut buf1.clone()).unwrap().unwrap();
    let mut buf2 = BytesMut::new();
    codec.encode(res, &mut buf2).unwrap();
    assert_eq!(buf1, buf2);

    // Submit
    let msg = StratumMessage::Submit(
        Id::Num(0),
        "job_id".to_string(),
        "nonce".to_string(),
        "proof".to_string(),
    );
    let mut buf1 = BytesMut::new();
    codec.encode(msg, &mut buf1).unwrap();
    let res = codec.decode(&mut buf1.clone()).unwrap().unwrap();
    let mut buf2 = BytesMut::new();
    codec.encode(res, &mut buf2).unwrap();
    assert_eq!(buf1, buf2);

    // Response(Id, Option<ResponseMessage>, Option<Error<()>>),
    let error = Error::with_custom_msg(
        ErrorCode::InvalidParams,
        &InvalidProof(Some("test error".to_string())).to_string(),
    );
    let msg = StratumMessage::Response(Id::Num(0), None, Some(error));
    let mut buf1 = BytesMut::new();
    codec.encode(msg, &mut buf1).unwrap();
    let res = codec.decode(&mut buf1.clone()).unwrap().unwrap();
    let mut buf2 = BytesMut::new();
    codec.encode(res, &mut buf2).unwrap();
    assert_eq!(buf1, buf2);
}

#[test]
fn test_request() {
    use crate::{MIN_SUPPORTED_PROTOCOL_VERSION, PROTOCOL_PREFIX};
    let request = Request {
        jsonrpc: Version::V2,
        method: "mining.subscribe",
        params: Some(SubscribeParams(
            "ABMatrix_ZKWork_Miner".to_string(),
            format!("{}/{}", PROTOCOL_PREFIX, MIN_SUPPORTED_PROTOCOL_VERSION),
            None,
        )),
        id: Some(Id::Num(0)),
    };
    let vec1 = serde_json::to_vec(&request).unwrap_or_default();

    let string = std::str::from_utf8(&vec1)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
        .unwrap();
    println!("{:?}", string);
    let mut buf = bytes::BytesMut::new();
    StratumCodec::default()
        .codec
        .encode(string, &mut buf)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
        .unwrap();
    println!("{:?}", buf)
}
