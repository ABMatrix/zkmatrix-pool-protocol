use anyhow::anyhow;

pub fn decode_hash_leaves(leaves_string: &Vec<String>) -> anyhow::Result<Vec<Vec<u8>>> {
    let mut leaves = Vec::new();
    for l in leaves_string {
        match hex::decode(l) {
            Ok(b) => leaves.push(b),
            Err(e) => {
                return Err(anyhow!("decode leaves {} failed with error: {}", l, e));
            }
        }
    }
    Ok(leaves)
}


pub fn decode_block_header_root(block_header_root: &str) -> anyhow::Result<Vec<u8>> {
    let block_header_root_b = match hex::decode(block_header_root) {
        Ok(b) => b,
        Err(e) => {
            return Err(anyhow!("decode block_header_root failed with error: {}", e));
        }
    };

    Ok(block_header_root_b)
}