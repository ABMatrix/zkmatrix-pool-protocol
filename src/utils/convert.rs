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
    use snarkvm_synthesizer::CoinbasePuzzle;
    use snarkvm::prelude::{Address, EpochChallenge, FromBytes, PrivateKey, PuzzleConfig, Testnet3, ToBytes, Uniform};
    use snarkvm::curves::bls12_377::{fr, Bls12_377};
    use snarkvm::algorithms::{polycommit::kzg10::KZGProof, fft::{DensePolynomial, EvaluationDomain, Evaluations}};
    use rand;
    use rand::RngCore;
    use std::str::FromStr;

    let max_degree = 1 << 15;
    let mut rng = rand::thread_rng();
    let max_config = PuzzleConfig { degree: max_degree };
    let srs = CoinbasePuzzle::<Testnet3>::setup(max_config).unwrap();
    let degree = (1 << 5) - 1;
    let config = PuzzleConfig { degree };
    let puzzle = CoinbasePuzzle::<Testnet3>::trim(&srs, config).unwrap();

    let epoch_challenge = EpochChallenge::<Testnet3>::new(rng.next_u32(), Default::default(), degree).unwrap();
    println!("epoch_block_hash: {}", epoch_challenge.epoch_block_hash().to_string());

    let d = epoch_challenge.degree();

    let epoch_challenge_2 = EpochChallenge::<Testnet3>::new(epoch_challenge.epoch_number(), epoch_challenge.epoch_block_hash(), d).unwrap();
    assert_eq!(epoch_challenge, epoch_challenge_2);


    // test epoch_polynomial
    let coeffs_s = epoch_challenge.epoch_polynomial().coeffs().to_vec().iter().map(|c| c.to_string()).collect::<Vec<String>>();

    let mut coeffs = vec![];
    for c in coeffs_s {
        let data = fr::Fr::from_str(&c).unwrap();
        coeffs.push(data)
    }

    let polynomial = DensePolynomial::from_coefficients_vec(coeffs);
    assert_eq!(polynomial, epoch_challenge.epoch_polynomial().clone());

    // test epoch_polynomial_evaluations
    // println!("epoch_polynomial_evaluations: {:?}", epoch_challenge.epoch_polynomial_evaluations());
    let evaluations_s = epoch_challenge.epoch_polynomial_evaluations().evaluations.iter().map(|e| e.to_string()).collect::<Vec<String>>();
    let size = epoch_challenge.epoch_polynomial_evaluations().domain().size.to_string();
    let log_size_of_group = epoch_challenge.epoch_polynomial_evaluations().domain().log_size_of_group;
    let size_as_field_element = epoch_challenge.epoch_polynomial_evaluations().domain().size_as_field_element.to_string();
    let size_inv = epoch_challenge.epoch_polynomial_evaluations().domain().size_inv.to_string();
    let group_gen = epoch_challenge.epoch_polynomial_evaluations().domain().group_gen.to_string();
    let group_gen_inv = epoch_challenge.epoch_polynomial_evaluations().domain().group_gen_inv.to_string();
    let generator_inv = epoch_challenge.epoch_polynomial_evaluations().domain().generator_inv.to_string();
    let e_domain = EvaluationDomain {
        size: u64::from_str(&size).unwrap(),
        log_size_of_group,
        size_as_field_element: fr::Fr::from_str(&size_as_field_element).unwrap(),
        size_inv: fr::Fr::from_str(&size_inv).unwrap(),
        group_gen: fr::Fr::from_str(&group_gen).unwrap(),
        group_gen_inv: fr::Fr::from_str(&group_gen_inv).unwrap(),
        generator_inv: fr::Fr::from_str(&generator_inv).unwrap(),
    };
    let mut evaluations = vec![];
    for e in evaluations_s {
        let fp256 = fr::Fr::from_str(&e).unwrap();
        evaluations.push(fp256);
    }
    let evaluations1 = Evaluations::from_vec_and_domain(evaluations, e_domain);
    assert_eq!(evaluations1, epoch_challenge.epoch_polynomial_evaluations().clone());

    let private_key = PrivateKey::<Testnet3>::new(&mut rng).unwrap();
    let address = Address::try_from(private_key).unwrap();
    let epoch_challenge_s = hex::encode(epoch_challenge.to_bytes_le().unwrap());
    let address_s = hex::encode(address.to_bytes_le().unwrap());
    let epoch_challenge_2 = EpochChallenge::<Testnet3>::from_bytes_le(convert_to_u8(&ConvertType::EpochChallenge(epoch_challenge_s)).unwrap().as_slice()).unwrap();
    assert_eq!(epoch_challenge, epoch_challenge_2);
    let address_2 = Address::<Testnet3>::from_bytes_le(convert_to_u8(&ConvertType::Address(address_s)).unwrap().as_slice()).unwrap();
    assert_eq!(address, address_2);
    let result = CoinbasePuzzle::prove(&puzzle, &epoch_challenge, address, u64::rand(&mut rng)).unwrap();
    println!("{}", result.nonce());
    let proof_hex = hex::encode(result.proof().to_bytes_le().unwrap());
    println!("{:?}", proof_hex);
    let result1 = KZGProof::<Bls12_377>::from_bytes_le(&hex::decode(proof_hex).unwrap()).unwrap();

    assert_eq!(result1, result.proof().clone())
}