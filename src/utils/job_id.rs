use anyhow::anyhow;

pub fn get_height(job_id: String) -> anyhow::Result<u32> {
    let step1 = job_id.split('_').collect::<Vec<&str>>();
    if step1.len() != 2 {
        return Err(anyhow!("Invalid job_id"));
    };
    let step2 = match hex::decode(step1[0]) {
        Ok(height_bytes) => {
            if height_bytes.len() != 4 {
                return Err(anyhow!("Invalid job_id"));
            }
            u32::from_le_bytes([height_bytes[0], height_bytes[1], height_bytes[2], height_bytes[3]])
        }
        Err(e) => {
            return Err(anyhow!("decode height from job_id failed: {}", e));
        }
    };
    Ok(step2)
}

#[test]
fn test_get_height() {
    let height_raw = 685514u32;
    let height_s = hex::encode(height_raw.to_le_bytes());
    let job_id = height_s + "_ahsdkaehahdla4da4daw8ea5d48awa5e4a5";
    println!("{}", &job_id);
    let height = get_height(job_id).unwrap();
    assert_eq!(height, height_raw)
}