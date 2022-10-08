use anyhow::anyhow;

pub fn proving_key_u8(pk_hex: &str) -> anyhow::Result<Vec<u8>> {
    let pk = match hex::decode(pk_hex) {
        Ok(p) => p,
        Err(e) => {
            return Err(anyhow!("decode proving_key failed with error: {}", e));
        }
    };
    Ok(pk)
}

pub fn epoch_challenge_u8(epoch_challenge: &str) -> anyhow::Result<Vec<u8>> {
    let challenge = match hex::decode(epoch_challenge) {
        Ok(c) => c,
        Err(e) => {
            return Err(anyhow!("decode epoch_challenge failed with error: {}", e));
        }
    };
    Ok(challenge)
}

pub fn address_u8(address: &str) -> anyhow::Result<Vec<u8>> {
    let address = match hex::decode(address) {
        Ok(a) => a,
        Err(e) => {
            return Err(anyhow!("decode address failed with error: {}", e));
        }
    };
    Ok(address)
}

#[test]
fn test_decode() {
    use snarkvm_compiler::EpochChallenge;
    use snarkvm_console::network::Testnet3;
    use snarkvm_console::account::{PrivateKey,Address};
    use rand;
    use rand::RngCore;
    use snarkvm_console::prelude::{ToBytes, FromBytes};

    let mut rng = rand::thread_rng();
    let degree = (1 << 5) - 1;
    let epoch_challenge = EpochChallenge::<Testnet3>::new(rng.next_u64(), Default::default(), degree).unwrap();
    let private_key = PrivateKey::<Testnet3>::new(&mut rng).unwrap();
    let address = Address::try_from(private_key).unwrap();
    let epoch_challenge_s = hex::encode(epoch_challenge.to_bytes_le().unwrap());
    let address_s = hex::encode(address.to_bytes_le().unwrap());
    let epoch_challenge_2 = EpochChallenge::<Testnet3>::from_bytes_le(epoch_challenge_u8(&epoch_challenge_s).unwrap().as_slice()).unwrap();
    assert_eq!(epoch_challenge, epoch_challenge_2);
    let address_2 = Address::<Testnet3>::from_bytes_le(address_u8(&address_s).unwrap().as_slice()).unwrap();
    assert_eq!(address, address_2)
}