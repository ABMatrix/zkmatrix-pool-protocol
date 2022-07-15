use snarkvm::dpc::Network;
use snarkos::environment::{Environment, helpers::NodeType};
use std::marker::PhantomData;

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
