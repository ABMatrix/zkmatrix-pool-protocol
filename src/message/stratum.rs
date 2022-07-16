use std::io;
use bytes::BytesMut;
use json_rpc_types::{Error, ErrorCode, Id, Request, Response, Version};
use json_rpc_types::Id::Num;
use serde_json::Value;
use tokio_util::codec::{AnyDelimiterCodec, Decoder, Encoder};
use super::response::ResponseMessage;
use serde::{Serialize, Deserialize};

pub enum StratumMessage {
    /// This first version doesn't support vhosts.
    /// (id, user_agent, protocol_version, session_id)
    Subscribe(Id, String, String, Option<String>),

    /// (id, account_name, miner_name, worker_password)
    Authorize(Id, String, Option<String>, Option<String>),

    /// This is the difficulty target for the next job.
    /// (difficulty_target)
    SetTarget(u64),

    /// New job from the mining pool.
    /// See protocol specification for details about the fields.
    /// (job_id, previous_block_hash, block_header_root, hashed_leaves_1, hashed_leaves_2, hashed_leaves_3,
    ///  hashed_leaves_4, clean_jobs)
    Notify(String, String, String, String, String, String, String, bool),

    /// Submit shares to the pool.
    /// See protocol specification for details about the fields.
    /// (id, account_name, worker_name, job_id, nonce, proof)
    Submit(Id, String, String, String, String, String),

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
    codec: AnyDelimiterCodec,
}

impl Default for StratumCodec {
    fn default() -> Self {
        Self {
            // Notify is ~400 bytes and submitt is about ~1750 bytes. 4096 should be enough for all messages.
            codec: AnyDelimiterCodec::new_with_max_length(vec![b'\n'], vec![b'\n'], 4096),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct NotifyParams(String, String, String, String, String, String, String, bool);

#[derive(Serialize, Deserialize)]
struct SubscribeParams(String, String, Option<String>);

#[derive(Serialize, Deserialize)]
struct AuthorizeParams(String, Option<String>, Option<String>);

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
                previous_block_hash,
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
                        previous_block_hash,
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
            StratumMessage::Submit(id, account_name, miner_name, job_id, nonce, proof) => {
                let request = Request {
                    jsonrpc: Version::V2,
                    method: "mining.submit",
                    params: Some(vec![account_name, miner_name, job_id, nonce, proof]),
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
                    let response = Response::<Option<ResponseMessage>, ()>::result(Version::V2, result, Some(id));
                    serde_json::to_vec(&response).unwrap_or_default()
                }
            },
        };
        let string =
            std::str::from_utf8(&bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
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
            let params = request.params.unwrap_or_default();
            match method {
                "mining.subscribe" => {
                    if params.len() != 3 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params"));
                    }
                    let user_agent = params[0].as_str().unwrap_or_default();
                    let protocol_version = params[1].as_str().unwrap_or_default();
                    let session_id = match &params[2] {
                        Value::String(s) => Some(s),
                        Value::Null => None,
                        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params")),
                    };
                    StratumMessage::Subscribe(
                        id.unwrap_or(Id::Num(0)),
                        user_agent.to_string(),
                        protocol_version.to_string(),
                        session_id.cloned(),
                    )
                }
                "mining.authorize" => {
                    if params.len() != 3 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params"));
                    }
                    let account_name = params[0].as_str().unwrap_or_default();
                    let miner_name = match &params[1] {
                        Value::String(s) => Some(s),
                        Value::Null => None,
                        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params")),
                    };
                    let worker_password = match &params[2] {
                        Value::String(s) => Some(s),
                        Value::Null => None,
                        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params")),
                    };
                    StratumMessage::Authorize(
                        id.unwrap_or(Id::Num(0)),
                        account_name.to_string(),
                        miner_name.cloned(),
                        worker_password.cloned(),
                    )
                }
                "mining.set_target" => {
                    if params.len() != 1 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params"));
                    }
                    let difficulty_target = params[0].as_u64().unwrap_or_default();
                    StratumMessage::SetTarget(difficulty_target)
                }
                "mining.notify" => {
                    if params.len() != 8 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params"));
                    }
                    let job_id = params[0].as_str().unwrap_or_default();
                    let previous_block_hash = params[1].as_str().unwrap_or_default();
                    let block_header_root = params[2].as_str().unwrap_or_default();
                    let hashed_leaves_1 = params[3].as_str().unwrap_or_default();
                    let hashed_leaves_2 = params[4].as_str().unwrap_or_default();
                    let hashed_leaves_3 = params[5].as_str().unwrap_or_default();
                    let hashed_leaves_4 = params[6].as_str().unwrap_or_default();
                    let clean_jobs = params[7].as_bool().unwrap_or(true);
                    StratumMessage::Notify(
                        job_id.to_string(),
                        previous_block_hash.to_string(),
                        block_header_root.to_string(),
                        hashed_leaves_1.to_string(),
                        hashed_leaves_2.to_string(),
                        hashed_leaves_3.to_string(),
                        hashed_leaves_4.to_string(),
                        clean_jobs,
                    )
                }
                "mining.submit" => {
                    if params.len() != 5 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params"));
                    }

                    let account_name = params[0].as_str().unwrap_or_default();
                    let worker_name = params[1].as_str().unwrap_or_default();
                    let job_id = params[2].as_str().unwrap_or_default();
                    let nonce = params[3].as_str().unwrap_or_default();
                    let proof = params[4].as_str().unwrap_or_default();
                    StratumMessage::Submit(
                        id.unwrap_or(Id::Num(0)),
                        account_name.to_string(),
                        worker_name.to_string(),
                        job_id.to_string(),
                        nonce.to_string(),
                        proof.to_string(),
                    )
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
                Ok(payload) => StratumMessage::Response(id.unwrap_or(Id::Num(0)), Some(payload), None),
                Err(error) => StratumMessage::Response(id.unwrap_or(Id::Num(0)), None, Some(error)),
            }
        };
        Ok(Some(result))
    }
}

#[test]
fn test_encode_decode() {
    let mut codec = StratumCodec::default();
    //Subscribe
    let msg = StratumMessage::Subscribe(Num(0), "ABMatrix_Aleo_Miner".to_string(), "ABMatrix_Aleo_Miner_4".to_string(), Some("session".to_string()));
    let mut buf1 = BytesMut::new();
    codec.encode(msg, &mut buf1).unwrap();
    let res = codec.decode(&mut buf1.clone()).unwrap().unwrap();
    let mut buf2 = BytesMut::new();
    codec.encode(res, &mut buf2).unwrap();
    assert_eq!(buf1, buf2);


    //Authorize
    let msg = StratumMessage::Authorize(Num(0), "account_name".to_string(), Some("worker_name".to_string()), None);
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
        "previous_block_hash".to_string(),
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
    let msg = StratumMessage::Submit(Num(2), "account_name".to_string(), "worker_name".to_string(), "job_id".to_string(), "nonce".to_string(), "proof".to_string());
    let mut buf1 = BytesMut::new();
    codec.encode(msg, &mut buf1).unwrap();
    let res = codec.decode(&mut buf1.clone()).unwrap().unwrap();
    let mut buf2 = BytesMut::new();
    codec.encode(res, &mut buf2).unwrap();
    assert_eq!(buf1, buf2);

    // Response(Id, Option<ResponseMessage>, Option<Error<()>>),
    let msg = StratumMessage::Response(Num(3), None, Some(Error::with_custom_msg(ErrorCode::InvalidParams, "test error")));
    let mut buf1 = BytesMut::new();
    codec.encode(msg, &mut buf1).unwrap();
    let res = codec.decode(&mut buf1.clone()).unwrap().unwrap();
    let mut buf2 = BytesMut::new();
    codec.encode(res, &mut buf2).unwrap();
    assert_eq!(buf1, buf2);
}