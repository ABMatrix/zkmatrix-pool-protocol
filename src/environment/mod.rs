use snarkos::environment::{helpers::NodeType, Environment};
use snarkvm::dpc::testnet2::Testnet2;
use snarkvm::dpc::Network;
use std::marker::PhantomData;

pub type PoolNetwork = Testnet2;

#[derive(Clone, Debug, Default)]
pub struct ABMatrixPoolMiner<N: Network>(PhantomData<N>);

#[rustfmt::skip]
impl<N: Network> Environment for ABMatrixPoolMiner<N> {
    type Network = N;
    const NODE_TYPE: NodeType = NodeType::Miner;
    const MINIMUM_NUMBER_OF_PEERS: usize = 0;
    const MAXIMUM_NUMBER_OF_PEERS: usize = 0;
}

#[derive(Clone, Debug, Default)]
pub struct ABMatrixPoolAgent<N: Network>(PhantomData<N>);

#[rustfmt::skip]
impl<N: Network> Environment for ABMatrixPoolAgent<N> {
    type Network = N;
    const NODE_TYPE: NodeType = NodeType::Miner;
    const MINIMUM_NUMBER_OF_PEERS: usize = 0;
    const MAXIMUM_NUMBER_OF_PEERS: usize = 0;
}

#[derive(Clone, Debug, Default)]
pub struct ABMatrixPoolMinerTrial<N: Network>(PhantomData<N>);

#[rustfmt::skip]
impl<N: Network> Environment for ABMatrixPoolMinerTrial<N> {
    type Network = N;
    const NODE_TYPE: NodeType = NodeType::Miner;
    const MINIMUM_NUMBER_OF_PEERS: usize = 0;
    const MAXIMUM_NUMBER_OF_PEERS: usize = 0;
}

#[derive(Clone, Debug, Default)]
pub struct ABMatrixPoolAgentTrial<N: Network>(PhantomData<N>);

#[rustfmt::skip]
impl<N: Network> Environment for ABMatrixPoolAgentTrial<N> {
    type Network = N;
    const NODE_TYPE: NodeType = NodeType::Miner;
    const MINIMUM_NUMBER_OF_PEERS: usize = 0;
    const MAXIMUM_NUMBER_OF_PEERS: usize = 0;
}
