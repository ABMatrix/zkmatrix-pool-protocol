use anyhow::anyhow;

/// job_id should contains server_agent, block_height, epoch_num and  server_id
#[must_use]
pub fn new_job_id(
    server_agent: String,
    block_height: u32,
    epoch_num: u32,
    server_id: String,
) -> String {
    let height_s = hex::encode(block_height.to_le_bytes());
    let epoch_num_s = hex::encode(epoch_num.to_le_bytes());
    format!(
        "{}/{}_{}_{}",
        server_agent, height_s, epoch_num_s, server_id
    )
}

/// return server_agent, block_height, epoch_num and server_id
#[must_use]
pub fn get_height_and_epoch_num(job_id: String) -> anyhow::Result<(String, u32, u32, String)> {
    // step1: split server_agent
    let step1 = job_id.split('/').collect::<Vec<&str>>();
    let (server_agent, remains) = if step1.len() == 2 && !step1[0].trim().is_empty() {
        (step1[0], step1[1])
    } else {
        return Err(anyhow!("Invalid job_id"));
    };

    // step2: split block_height, epoch_num and server_id
    let step2 = remains.split('_').collect::<Vec<&str>>();
    if step2.len() != 3 {
        return Err(anyhow!("Invalid job_id"));
    };
    let height = match hex::decode(step2[0]) {
        Ok(height_bytes) => {
            if height_bytes.len() != 4 {
                return Err(anyhow!("Invalid job_id"));
            }
            u32::from_le_bytes([
                height_bytes[0],
                height_bytes[1],
                height_bytes[2],
                height_bytes[3],
            ])
        }
        Err(_) => {
            return Err(anyhow!("Invalid job_id"));
        }
    };

    let epoch_num = match hex::decode(step2[1]) {
        Ok(height_bytes) => {
            if height_bytes.len() != 4 {
                return Err(anyhow!("Invalid job_id"));
            }
            u32::from_le_bytes([
                height_bytes[0],
                height_bytes[1],
                height_bytes[2],
                height_bytes[3],
            ])
        }
        Err(_) => {
            return Err(anyhow!("Invalid job_id"));
        }
    };

    let server_id = step2[2];

    Ok((
        server_agent.to_string(),
        height,
        epoch_num,
        server_id.to_string(),
    ))
}

#[test]
fn test_get_height_and_epoch_num() {
    let server_agent = "zkmatrix".to_string();
    let height_raw = 685514u32;
    let epoch_num_raw = 1000u32;
    let server_id = "ahsdkaehahdla4da4daw8ea5d48awa5e4a5".to_string();
    let job_id = new_job_id(
        server_agent.clone(),
        height_raw,
        epoch_num_raw,
        server_id.clone(),
    );
    println!("{}", &job_id);
    let (server_agent_r, height, epoch_num, server_id_r) =
        get_height_and_epoch_num(job_id).unwrap();
    assert_eq!(height, height_raw);
    assert_eq!(epoch_num, epoch_num_raw);
    assert_eq!(server_agent_r, server_agent);
    assert_eq!(server_id_r, server_id)
}
