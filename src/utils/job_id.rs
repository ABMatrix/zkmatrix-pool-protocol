use anyhow::anyhow;

pub fn get_height_and_epoch_num(job_id: String) -> anyhow::Result<(u32, u32)> {
    let step1 = job_id.split('_').collect::<Vec<&str>>();
    if step1.len() != 3 {
        return Err(anyhow!("Invalid job_id"));
    };
    let height = match hex::decode(step1[0]) {
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

    let epoch_num = match hex::decode(step1[1]) {
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

    Ok((height, epoch_num))
}

#[test]
fn test_get_height_and_epoch_num() {
    let height_raw = 685514u32;
    let epoch_num_raw = 1000u32;
    let height_s = hex::encode(height_raw.to_le_bytes());
    let epoch_num_s = hex::encode(epoch_num_raw.to_le_bytes());
    let job_id = height_s + "_" + &epoch_num_s + "_ahsdkaehahdla4da4daw8ea5d48awa5e4a5";
    println!("{}", &job_id);
    let (height, epoch_num) = get_height_and_epoch_num(job_id).unwrap();
    assert_eq!(height, height_raw);
    assert_eq!(epoch_num, epoch_num_raw)
}
