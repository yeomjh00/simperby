use crate::{crypto::*, reserved::ReservedState};
use serde::{Deserialize, Serialize};

pub type VotingPower = u64;
/// A UNIX timestamp measured in milliseconds.
pub type Timestamp = u64;
/// A block height. The genesis block is at height 0.
pub type BlockHeight = u64;
pub type ConsensusRound = u64;
pub type FinalizationProof = Vec<TypedSignature<BlockHeader>>;
pub type MemberName = String;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Member {
    pub public_key: PublicKey,
    /// The name of the member that will be used in human-readable interfaces.
    /// This must be unique.
    pub name: MemberName,
    pub governance_voting_power: VotingPower,
    pub consensus_voting_power: VotingPower,
    pub governance_delegations: Option<PublicKey>,
    pub consensus_delegations: Option<PublicKey>,
    // TODO: add various conditions for each delegation.
    // - Unlock-Automatically-After-N-Blocks
    // - Unlock-Automatically-After-T-Seconds
    // - Unlock-If-The-Delegatee-Is-Not-Active
    // - Unlock-If-The-Validator-Set-Changes
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct BlockHeader {
    /// The author of this block.
    pub author: PublicKey,
    /// The signature of the previous block.
    pub prev_block_finalization_proof: FinalizationProof,
    /// The hash of the previous block.
    pub previous_hash: Hash256,
    /// The height of this block.
    pub height: BlockHeight,
    /// The timestamp of this block.
    pub timestamp: Timestamp,
    /// The hash of aggregation of all the commits for this block.
    pub commit_hash: Hash256,
    /// The Merkle root of transactions.
    pub tx_merkle_root: Hash256,
    /// The Merkle root of the chat log.
    pub chat_merkle_root: Hash256,
    /// The Merkle root of the non-essential state.
    pub repository_merkle_root: Hash256,
    /// The effective validator set (delegation-applied) for the next block.
    ///
    /// The order here is the consensus leader selection order.
    pub validator_set: Vec<(PublicKey, VotingPower)>,
    /// The protocol version that must be used from next block.
    ///
    /// It must be a valid semantic version (e.g., `0.2.3`).
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Agenda {
    pub author: PublicKey,
    pub timestamp: Timestamp,
    pub hash: Hash256,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ChatLog {
    // TODO
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct AgendaProof {
    pub agenda_hash: Hash256,
    pub proof: Vec<(PublicKey, TypedSignature<Agenda>)>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum Diff {
    /// Nothing changed in the repository; an empty commit.
    None,
    /// Only changes non-reserved areas. Contains the hash of the diff.
    ///
    /// The actual content of the diff is not covered by this crate; see `simperby-repository`.
    General(Hash256),
    /// Changes the reserved area. Contains the new reserved state and the hash of the diff.
    /// It holds the reserved state as a `Box` to flatten the variant size.
    /// (see https://rust-lang.github.io/rust-clippy/master/index.html#large_enum_variant)
    Reserved(Box<ReservedState>, Hash256),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Transaction {
    pub author: PublicKey,
    pub timestamp: Timestamp,
    pub head: String,
    pub body: String,
    pub diff: Diff,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum ExtraAgendaTransaction {
    Delegate(TxDelegate),
    Undelegate(TxUndelegate),
    Report(TxReport),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct TxDelegate {
    pub delegator: PublicKey,
    pub delegatee: PublicKey,
    /// Whether to delegate the governance voting power too.
    pub governance: bool,
    pub proof: TypedSignature<(PublicKey, PublicKey, bool, BlockHeight)>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct TxUndelegate {
    pub delegator: PublicKey,
    pub proof: TypedSignature<(PublicKey, BlockHeight)>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct TxReport {
    // TODO
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct GenesisInfo {
    pub header: BlockHeader,
    pub genesis_proof: FinalizationProof,
    pub chain_name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum Commit {
    Block(BlockHeader),
    Transaction(Transaction),
    Agenda(Agenda),
    AgendaProof(AgendaProof),
    ExtraAgendaTransaction(ExtraAgendaTransaction),
    ChatLog(ChatLog),
}
