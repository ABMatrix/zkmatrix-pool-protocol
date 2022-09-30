use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use anyhow::anyhow;
use rand::thread_rng;
use tokio::time::Instant;

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

// #[test]
// fn test_decode() {
//     use snarkvm_dpc::prelude::*;
//     use snarkvm_dpc::testnet2::Testnet2;
//     use snarkvm_utilities::ToBytes;
//
//     // Construct the block template.
//     let block = Testnet2::genesis_block();
//     let expected_template = BlockTemplate::new(
//         block.previous_block_hash(),
//         block.height(),
//         block.timestamp(),
//         block.difficulty_target(),
//         block.cumulative_weight(),
//         block.previous_ledger_root(),
//         block.transactions().clone(),
//         block.to_coinbase_transaction().unwrap().to_records().next().unwrap(),
//     );
//
//     let header_tree = expected_template.to_header_tree().unwrap();
//     let header_root = header_tree.root();
//     let hashed_leaves = header_tree.hashed_leaves();
//     let mut leaves_raw = vec![];
//     let leaves_string = {
//         || {
//             let mut v = vec![];
//             for i in hashed_leaves {
//                 let raw = i.to_bytes_le().unwrap();
//                 leaves_raw.push(raw.clone());
//                 v.push(hex::encode(raw));
//             }
//             v
//         }
//     }();
//
//     let header_root_raw = header_root.to_bytes_le().unwrap();
//     let block_header_root = hex::encode(header_root_raw.clone());
//
//     let hash_leaves_u8 = decode_hash_leaves(&leaves_string).unwrap();
//     let block_header_root_u8 = decode_block_header_root(&block_header_root).unwrap();
//     assert_eq!(leaves_raw, hash_leaves_u8);
//     assert_eq!(header_root_raw, block_header_root_u8);
//     let t = AtomicBool::new(false);
//
//     let result = BlockHeader::<Testnet2>::mine_once_unchecked(&expected_template, &t, &mut thread_rng()).unwrap();
//     let start = Instant::now();
//     let _ = result.proof().to_proof_difficulty().unwrap();
//     println!("{:?}", Instant::now().saturating_duration_since(start));
// }