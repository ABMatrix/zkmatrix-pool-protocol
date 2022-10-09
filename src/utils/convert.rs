use anyhow::anyhow;

pub enum ConvertType {
    EpochChallenge(String),
    Address(String),
    ProverSolution(String),
}

impl ConvertType {
    fn name(&self) -> &str {
        match self {
            ConvertType::EpochChallenge(_) => "EpochChallenge",
            ConvertType::Address(_) => "Address",
            ConvertType::ProverSolution(_) => "ProverSolution"
        }
    }

    fn raw_data(&self) -> String {
        match self {
            ConvertType::EpochChallenge(data) => data.clone(),
            ConvertType::Address(data) => data.clone(),
            ConvertType::ProverSolution(data) => data.clone()
        }
    }
}

pub fn convert_to_u8(data: &ConvertType) -> anyhow::Result<Vec<u8>> {
    let data_s = data.raw_data();
    let data_u8 = match hex::decode(data_s) {
        Ok(data_u8) => data_u8,
        Err(e) => {
            return Err(anyhow!("decode {} failed with error: {}", data.name(), e));
        }
    };
    Ok(data_u8)
}

#[test]
fn test_decode() {
    use snarkvm_compiler::{EpochChallenge, CoinbasePuzzle, PuzzleConfig};
    use snarkvm_console::network::Testnet3;
    use snarkvm_console::account::{PrivateKey, Address};
    use rand;
    use rand::RngCore;
    use snarkvm_console::prelude::{ToBytes, FromBytes};
    use snarkvm_utilities::Uniform;

    let max_degree = 1 << 15;
    let mut rng = rand::thread_rng();
    let max_config = PuzzleConfig { degree: max_degree };
    let srs = CoinbasePuzzle::<Testnet3>::setup(max_config, &mut rng).unwrap();
    let degree = (1 << 5) - 1;
    let config = PuzzleConfig { degree };
    let (pk, _vk) = CoinbasePuzzle::<Testnet3>::trim(&srs, config).unwrap();

    let epoch_challenge = EpochChallenge::<Testnet3>::new(rng.next_u64(), Default::default(), degree).unwrap();
    let private_key = PrivateKey::<Testnet3>::new(&mut rng).unwrap();
    let address = Address::try_from(private_key).unwrap();
    let epoch_challenge_s = hex::encode(epoch_challenge.to_bytes_le().unwrap());
    let address_s = hex::encode(address.to_bytes_le().unwrap());
    let epoch_challenge_2 = EpochChallenge::<Testnet3>::from_bytes_le(convert_to_u8(&ConvertType::EpochChallenge(epoch_challenge_s)).unwrap().as_slice()).unwrap();
    assert_eq!(epoch_challenge, epoch_challenge_2);
    let address_2 = Address::<Testnet3>::from_bytes_le(convert_to_u8(&ConvertType::Address(address_s)).unwrap().as_slice()).unwrap();
    assert_eq!(address, address_2);
    let result = CoinbasePuzzle::prove(&pk, &epoch_challenge, &address, u64::rand(&mut rng)).unwrap();
    println!("{}", result.nonce())
}