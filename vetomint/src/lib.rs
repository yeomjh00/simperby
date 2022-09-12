mod progress;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// An index of the validator, which is for a single height. (Mapping from the actual public key to the index may differ for different heights.)
pub type ValidatorIndex = usize;
/// An identifier of the block, which is uniquely mapped to a block. Like `ValidatorIndex`, it is for a single height. (Mapping from the actual block to the index may differ for different heights.)
pub type BlockIdentifier = usize;
/// A round.
pub type Round = usize;
/// A voting power.
pub type VotingPower = u64;
/// A UNIX timestamp measured in milliseconds.
pub type Timestamp = i64;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ConsensusParams {
    pub timeout_ms: u64,
    pub repeat_round_for_first_leader: usize,
}

/// An event that (potentially) triggers a state transition of `StateMachine`.
///
/// Note that there is no cryptography-related info here, because it's
/// the lower layer's responsibility to verifiy and refine the raw messages (containing such cryptography-related info) into this abstracted data.
/// Also all the identifiers (for blocks and validators) become integer indices here, and
/// the lower layer will keep the mapping from the actual data to the indices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsensusEvent {
    /// Signals to start the process
    Start { time: Timestamp },
    /// Informs that the node has received a block proposal.
    BlockProposalReceived {
        proposal: BlockIdentifier,
        //Whether this proposal was valid or locked in this round.
        proposal_round: Option<Round>,
        proposer: ValidatorIndex,
        round: Round,
        time: Timestamp,
        /// Whether this node is in favor of the proposal.
        favor: bool,
    },
    /// Updates the block candidate which this node wants to propose in its turn.
    BlockCandidateUpdated {
        proposal: BlockIdentifier,
        time: Timestamp,
    },
    /// Informs that the node has received a block prevote.
    Prevote {
        proposal: BlockIdentifier,
        signer: ValidatorIndex,
        round: Round,
        time: Timestamp,
    },
    /// Informs that the node has received a block precommit.
    Precommit {
        proposal: BlockIdentifier,
        signer: ValidatorIndex,
        round: Round,
        time: Timestamp,
    },
    /// Informs that the node has received a nil prevote.
    NilPrevote {
        signer: ValidatorIndex,
        round: Round,
        time: Timestamp,
    },
    /// Informs that the node has received a nil precommit.
    NilPrecommit {
        signer: ValidatorIndex,
        round: Round,
        time: Timestamp,
    },
    /// Informs that time has passed.
    Timer {time: Timestamp },
}

impl ConsensusEvent {
    /// Return the time of the event
    fn time(&self) -> Timestamp {
        match self {
            ConsensusEvent::Start { time, .. } => *time,
            ConsensusEvent::BlockProposalReceived { time, .. } => *time,
            ConsensusEvent::BlockCandidateUpdated { time, .. } => *time,
            ConsensusEvent::Prevote { time, .. } => *time,
            ConsensusEvent::Precommit { time, .. } => *time,
            ConsensusEvent::NilPrevote { time, .. } => *time,
            ConsensusEvent::NilPrecommit { time, .. } => *time,
            ConsensusEvent::Timer { time, .. } => *time,
        }
    }

    fn round(&self) -> Option<Round> {
        match self {
            ConsensusEvent::Start { .. } => None,
            ConsensusEvent::BlockProposalReceived { round, .. } => Some(*round),
            ConsensusEvent::BlockCandidateUpdated { round, .. } => Some(*round),
            ConsensusEvent::Prevote { round, .. } => Some(*round),
            ConsensusEvent::Precommit { round, .. } => Some(*round),
            ConsensusEvent::NilPrevote { round, .. } => Some(*round),
            ConsensusEvent::NilPrecommit { round, .. } => Some(*round),
            ConsensusEvent::Timer { round, .. } => Some(*round),
        }
    }
}

/// A response that the consensus might emit for a given event, which must be properly handled by the lower layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsensusResponse {
    /// Creation of the actual proposal is not the role of the consensus; the lower layer will take care of it.
    CreateProposal {
        round: Round,
    },
    BroadcastProposal {
        proposal: BlockIdentifier,
        round: Round,
    },
    BroadcastPrevote {
        proposal: BlockIdentifier,
        round: Round,
    },
    BroadcastPrecommit {
        proposal: BlockIdentifier,
        round: Round,
    },
    BroadcastNilPrevote {
        round: Round,
    },
    BroadcastNilPrecommit {
        round: Round,
    },
    FinalizeBlock {
        proposal: BlockIdentifier,
    },
    ViolationReport {
        violator: ValidatorIndex,
        description: String,
    },
}

/// An immutable set of information that is used to perform the consensus for a single height.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeightInfo {
    /// The list of voting powers sorted by the leader order.
    ///
    /// Important note: `ValidatorIndex` is used to index this list.
    pub validators: Vec<VotingPower>,

    /// The index of this node
    pub this_node_index: Option<ValidatorIndex>,

    /// The timestamp of the beginning of the round 0.
    pub timestamp: Timestamp,

    /// The consensus parameters
    pub consensus_params: ConsensusParams,

    /// The initial block candidate that this node wants to propose.
    pub initial_block_candidate: BlockIdentifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ConsensusStep {
    Initial,
    Propose,
    Prevote,
    Precommit,
}

/// All vote information in a single round
/// prevote/precommit_total is sum of all casted voting power
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Votes {
    prevotes_total: VotingPower,
    prevotes_favor: BTreeMap<BlockIdentifier, VotingPower>,
    precommits_total: VotingPower,
    precommits_favor: BTreeMap<BlockIdentifier, VotingPower>,
}

/// The state of the consensus during a single height.
/// prevote/precommit history stores locked vote for veryfing did it really lock the value at that round
/// Some(BlockIdentifier) means validator already broadcasted BlockIdentifier
/// None means validator broadcasted NilPrevote/NilPrecommit
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsensusState {
    step: ConsensusStep,
    round: Round,
    locked_value: Option<BlockIdentifier>,
    locked_round: Option<Round>,
    valid_value: Option<BlockIdentifier>,
    valid_round: Option<Round>,
    timeout_propose: Option<Timestamp>,
    timeout_precommit: Option<Timestamp>,
    prevote_history: BTreeMap<Round, BTreeMap<ValidatorIndex, Option<BlockIdentifier>>>,
    precommit_history: BTreeMap<Round, BTreeMap<ValidatorIndex, Option<BlockIdentifier>>>,
    votes: BTreeMap<Round, Votes>,
    waiting_for_proposal_creation: bool,
    block_candidate: BlockIdentifier,
    height_info: HeightInfo,
}

impl ConsensusState {
    /// Prepares the initial state of the consensus.
    pub fn new(height_info: HeightInfo) -> Self {
        ConsensusState {
            step: ConsensusStep::Initial,
            round: 0,
            locked_value: None,
            locked_round: None,
            valid_value: None,
            valid_round: None,
            timeout_propose: None,
            timeout_precommit: None,
            prevote_history: Default::default(),
            precommit_history: Default::default(),
            votes: Default::default(),
            waiting_for_proposal_creation: false,
            block_candidate: height_info.initial_block_candidate,
            height_info,
        }
    }

    /// Makes a progress of the state machine with the given event.
    ///
    /// It returns `None` if the state machine is not ready to process the event.
    /// It returns `Some(Vec![])` if the state machine processed the event but did not emit any response.
    pub fn progress(&mut self, event: ConsensusEvent) -> Option<Vec<ConsensusResponse>> {
        progress::progress(self, event)
    }
}

pub fn decide_proposer(round: usize, height_info: &HeightInfo) -> ValidatorIndex {
    if round < height_info.consensus_params.repeat_round_for_first_leader {
        0
    } else {
        (round - height_info.consensus_params.repeat_round_for_first_leader + 1)
            % height_info.validators.len()
    }
}
