use futures_util::{SinkExt, StreamExt};
use json_rpc_types::Id;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::task;
use tokio::time::sleep;
use tokio_util::codec::Framed;
use zkmatrix_pool_protocol::message::response::ResponseMessage;
use zkmatrix_pool_protocol::message::stratum::{StratumCodec, StratumMessage};
use zkmatrix_pool_protocol::CURRENT_PROTOCOL_VERSION;

#[tokio::main]
async fn main() {
    let handle1 = task::spawn(async move {
        start_server().await;
    });
    let handle2 = task::spawn(async move {
        start_miner().await;
    });
    let _ = handle1.await;
    let _ = handle2.await;
}

async fn start_miner() {
    println!("start miner");
    // step1. connect
    let stream = TcpStream::connect("127.0.0.1:6666").await.unwrap();
    let mut framed = Framed::new(stream, StratumCodec::default());

    // step2. subscribe
    framed
        .send(StratumMessage::Subscribe(
            Id::Num(0),
            "user_agent".to_string(),
            CURRENT_PROTOCOL_VERSION.to_string(),
            None,
        ))
        .await
        .unwrap();

    // wait response
    match framed.next().await {
        Some(res) => match res {
            Ok(msg) => match msg {
                StratumMessage::Response(_id, _result, error) => {
                    if !error.is_none() {
                        println!("{}", error.unwrap().message);
                        return;
                    }
                }
                _ => println!("unexpected msg {}", msg.name()),
            },
            Err(e) => {
                println!("{}", e);
            }
        },
        None => {
            println!("disconnected");
        }
    }
    println!("subscribe ok");

    // step3. authorize
    framed
        .send(StratumMessage::Authorize(
            Id::Num(1),
            "account_name".to_string(),
            "miner_name".to_string(),
            None,
        ))
        .await
        .unwrap();
    match framed.next().await.unwrap().unwrap() {
        StratumMessage::Response(_id, _result, error) => {
            if !error.is_none() {
                println!("{}", error.unwrap().message);
                return;
            }
        }
        _ => println!("unexpected msg"),
    }
    println!("authorize ok");

    // step4. listening and mining
    let mut num: f32 = 0.1;
    loop {
        num += 1.0;
        framed.send(StratumMessage::LocalSpeed(Id::Num(num as u64), num.to_string())).await.unwrap();
        match framed.next().await.unwrap().unwrap() {
            StratumMessage::Notify(
                _job_id,
                _difficulty,
                _epoch_challenge,
                _address,
                _clean_jobs,
            ) => {
                println!("miner: received new job");
                println!("miner: mining....");
                println!("miner: mining done");
                println!("miner: sent share");
                let _ = framed
                    .send(StratumMessage::Submit(
                        Id::Num(num as u64),
                        "job_id".to_string(),
                        "prover_solution".to_string(),
                    ))
                    .await;
            }
            StratumMessage::Response(_id, _result, error) => {
                if !error.is_none() {
                    println!("{}", error.unwrap().message);
                } else {
                    println!("server: received ok");
                }
            }
            _ => println!("miner: unexpected msg"),
        }
    }
}

async fn start_server() {
    println!("start server");
    let listener = TcpListener::bind("0.0.0.0:6666").await.unwrap();
    match listener.accept().await {
        Ok((stream, client_addr)) => {
            println!("server: {} connected to server", client_addr);
            let mut framed = Framed::new(stream, StratumCodec::default());

            let duration = Duration::from_secs(1);
            let now = Instant::now();

            let (tx, mut ticker) = mpsc::channel(1024);
            task::spawn(async move {
                loop {
                    if now.elapsed().gt(&duration) {
                        let _ = tx.send(()).await;
                    }
                    sleep(Duration::from_secs(1)).await;
                }
            });

            loop {
                tokio::select! {
                    _ = ticker.recv() => {
                    framed.send(StratumMessage::Notify(
                            "job_id".to_string(),
                            1,
                            "epoch_challenge".to_string(),
                            "address".to_string(),
                            true,
                        )).await.unwrap()
                    }
                    Some(Ok(msg)) = framed.next() => {
                        match msg {
                            StratumMessage::Subscribe(id, _, _, _) => {
                                let _ = framed.send(StratumMessage::Response(id, None, None)).await;
                            }
                            StratumMessage::Authorize(id, _, _, _) => {
                                let _ = framed.send(StratumMessage::Response(id, Some(ResponseMessage::Bool(true)), None)).await;
                                framed.send(StratumMessage::Notify(
                                    "job_id".to_string(),
                                    1,
                                    "epoch_challenge".to_string(),
                                    "address".to_string(),
                                    true,
                                )
                                ).await.unwrap()
                            }
                            StratumMessage::Notify(..) => {
                                println!("server: Unsupported msg received from client");
                            }
                            StratumMessage::LocalSpeed(id, speed) => {
                                println!("server: Received local speed, id: {:?}, speed: {}", id, speed);
                            }
                            StratumMessage::Submit(id, _, _) => {
                                println!("server: received submit from miner");
                                println!("server: submit passed");
                                let _ = framed.send(StratumMessage::Response(id, Some(ResponseMessage::Bool(true)), None)).await;
                            }
                            StratumMessage::Response(..) => {
                                println!("server: Unsupported msg received from client");
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}
