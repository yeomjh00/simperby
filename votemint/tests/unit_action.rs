use std::iter::*;
use votemint::*;
//TODO
//Change votes/commits, paramter of prevotes/precommits
//(index: usize, is_favor: bool, voting_power: u64, time: Timestamp) to (index: usize, is_favor: bool, time : Timestamp)
//get voting power from height_info

#[allow(dead_code)]
pub fn initialize(
    validators: Vec<u64>,
    this_node_index: Option<ValidatorIndex>,
    timestamp: Timestamp,
    consensus_params: ConsensusParams,
) -> (HeightInfo, ConsensusState) {
    let height_info = HeightInfo {
        validators,
        this_node_index,
        timestamp,
        consensus_params,
    };
    let state = ConsensusState::new(height_info.clone());
    (height_info, state)
}

// receive BlockProposal(Event) -progress-> ProposalFavor(Event)
#[allow(dead_code)]
#[warn(clippy::too_many_arguments)]
pub fn receive_and_favor_propose(
    height_info: &HeightInfo,
    state: &mut ConsensusState,
    proposal: BlockIdentifier,
    proposal_round: Option<Round>,
    proposer: ValidatorIndex,
    round: usize,
    time: Timestamp,
    favor: bool,
) -> (
    Option<Vec<ConsensusResponse>>,
    Option<Vec<ConsensusResponse>>,
) {
    let event = ConsensusEvent::BlockProposalReceived {
        proposal,
        proposal_round,
        proposer,
        round,
        time,
    };

    let proposal_response = state.progress(height_info, event);

    let event = ConsensusEvent::ProposalFavor {
        proposal,
        favor,
        time,
    };

    let favor_response = state.progress(height_info, event);
    (proposal_response, favor_response)
}

#[allow(dead_code)]
pub fn prevotes(
    height_info: &HeightInfo,
    state: &mut ConsensusState,
    favor_of_this_node: (bool, u64),
    votes: Vec<(ValidatorIndex, bool, u64, Timestamp)>,
    round: usize,
    proposal: usize,
) -> Vec<Option<Vec<ConsensusResponse>>> {
    let mut votes_time_sorted = votes;
    let total_voting_power: u64 = height_info.validators.iter().sum();
    votes_time_sorted.sort_by_key(|k| k.3);

    let early_termination_condition1 = total_voting_power * 2 / 3;
    let early_termination_condition2 = total_voting_power * 5 / 6;
    let mut current_prevoted = if favor_of_this_node.0 {
        favor_of_this_node.1
    } else {
        0
    };
    let mut current_nilvoted = if !favor_of_this_node.0 {
        favor_of_this_node.1
    } else {
        0
    };
    let mut current_voted;
    let mut return_responses = Vec::<Option<Vec<ConsensusResponse>>>::new();

    for (signer, favor, power, time) in votes_time_sorted {
        current_voted = current_prevoted + current_nilvoted;
        if current_prevoted > early_termination_condition1
            || current_nilvoted > early_termination_condition1
            || current_voted > early_termination_condition2
        {
            assertion_check(&return_responses);
            return return_responses;
        }

        if favor {
            let event = ConsensusEvent::Prevote {
                proposal,
                signer,
                round,
                time,
            };
            let response = state.progress(height_info, event);
            return_responses.push(response);
            current_prevoted += power;
        } else {
            let event = ConsensusEvent::NilPrevote {
                signer,
                round,
                time,
            };
            let response = state.progress(height_info, event);
            return_responses.push(response);
            current_nilvoted += power;
        }
    }
    assertion_check(&return_responses);
    return_responses
}

#[allow(dead_code)]
pub fn precommits(
    height_info: &HeightInfo,
    state: &mut ConsensusState,
    favor_of_this_node: (bool, u64),
    commits: Vec<(ValidatorIndex, bool, u64, Timestamp)>,
    round: usize,
    proposal: usize,
) -> Vec<Option<Vec<ConsensusResponse>>> {
    let mut commits_time_sorted = commits;
    let total_voting_power: u64 = height_info.validators.iter().sum();
    commits_time_sorted.sort_by_key(|k| k.3);

    let early_termination_condition1 = total_voting_power * 2 / 3;
    let mut current_precommitted = if favor_of_this_node.0 {
        favor_of_this_node.1
    } else {
        0
    };
    let mut current_nilcommitted = if !favor_of_this_node.0 {
        favor_of_this_node.1
    } else {
        0
    };
    let mut return_responses = Vec::<Option<Vec<ConsensusResponse>>>::new();

    for (signer, favor, power, time) in commits_time_sorted {
        if current_precommitted > early_termination_condition1
            || current_nilcommitted > early_termination_condition1
        {
            assertion_check(&return_responses);
            return return_responses;
        }
        if favor {
            let event = ConsensusEvent::Precommit {
                proposal,
                signer,
                round,
                time,
            };
            let response = state.progress(height_info, event);
            return_responses.push(response);
            current_precommitted += power;
        } else {
            let event = ConsensusEvent::NilPrecommit {
                signer,
                round,
                time,
            };
            let response = state.progress(height_info, event);
            return_responses.push(response);
            current_nilcommitted += power;
        }
    }
    assertion_check(&return_responses);
    return_responses
}

//Check formal n-1 responses are empty, on the other hand last one is not.
//By adding expected response as a parameter, can compare actual last response with expected response.
#[allow(dead_code)]
pub fn assertion_check(responses: &Vec<Option<Vec<ConsensusResponse>>>) {
    let length = responses.len();
    let formal_responses = &responses[0..length - 1];
    let last_response = &responses[length - 1];
    for response in formal_responses {
        assert_eq!(response, &Some(Vec::new()));
    }
    assert_ne!(last_response, &Some(Vec::new()));
}

#[allow(dead_code)]
pub fn bulk_prevote(
    height_info: &HeightInfo,
    state: &mut ConsensusState,
    proposal: BlockIdentifier,
    signers: Vec<ValidatorIndex>,
    round: usize,
    timestamps: Vec<Timestamp>,
) -> Option<Vec<ConsensusResponse>> {
    if signers.len() != timestamps.len() {
        panic!("Invalid lengths with signers and timestamps");
    }

    let idx = signers.len();
    let mut last_response: Option<Vec<ConsensusResponse>> = Some(Vec::new());

    for i in 0..idx {
        let signer = signers[i];
        let time = timestamps[i];
        let event = ConsensusEvent::Prevote {
            proposal,
            signer,
            round,
            time,
        };
        last_response = state.progress(height_info, event);
    }
    last_response
}

#[allow(dead_code)]
pub fn bulk_nilvote(
    height_info: &HeightInfo,
    state: &mut ConsensusState,
    signers: Vec<ValidatorIndex>,
    round: usize,
    timestamps: Vec<Timestamp>,
) -> Option<Vec<ConsensusResponse>> {
    if signers.len() != timestamps.len() {
        panic!("Invalid lengths with signers and timestamps");
    }

    let idx = signers.len();
    let mut last_response: Option<Vec<ConsensusResponse>> = Some(vec![]);

    for i in 0..idx {
        let signer = signers[i];
        let time = timestamps[i];
        let event = ConsensusEvent::NilPrevote {
            signer,
            round,
            time,
        };
        last_response = state.progress(height_info, event);
    }
    last_response
}

#[allow(dead_code)]
pub fn bulk_precommit(
    height_info: &HeightInfo,
    state: &mut ConsensusState,
    proposal: BlockIdentifier,
    signers: Vec<ValidatorIndex>,
    round: usize,
    timestamps: Vec<Timestamp>,
) -> Option<Vec<ConsensusResponse>> {
    if signers.len() != timestamps.len() {
        panic!("Invalid lengths with signers and timestamps");
    }

    let idx = signers.len();
    let mut last_response: Option<Vec<ConsensusResponse>> = Some(vec![]);

    for i in 0..idx {
        let signer = signers[i];
        let time = timestamps[i];
        let event = ConsensusEvent::Precommit {
            proposal,
            signer,
            round,
            time,
        };
        last_response = state.progress(height_info, event);
    }
    last_response
}

#[allow(dead_code)]
pub fn bulk_nilcommit(
    height_info: &HeightInfo,
    state: &mut ConsensusState,
    signers: Vec<ValidatorIndex>,
    round: usize,
    timestamps: Vec<Timestamp>,
) -> Option<Vec<ConsensusResponse>> {
    if signers.len() != timestamps.len() {
        panic!("Invalid lengths with signers and timestamps");
    }

    let idx = signers.len();
    let mut last_response: Option<Vec<ConsensusResponse>> = Some(vec![]);

    for i in 0..idx {
        let signer = signers[i];
        let time = timestamps[i];
        let event = ConsensusEvent::NilPrecommit {
            signer,
            round,
            time,
        };
        last_response = state.progress(height_info, event);
    }
    last_response
}
