mod unit_action;

use unit_action::*;
use votemint::*;

#[test]
fn early_termination_by_polka_1() {
    let (_, mut state) = initialize(
        vec![10, 8, 6, 5, 4, 2, 2],
        Some(3),
        0,
        ConsensusParams {
            timeout_ms: 1000,
            repeat_round_for_first_leader: 1,
        },
        0,
    );

    // STEP 1: Proposal.
    let event = ConsensusEvent::Start { time: 0 };
    let response = state.progress(event).unwrap();
    assert!(response.is_empty());

    let event = ConsensusEvent::BlockProposalReceived {
        proposal: 0,
        proposal_round: None,
        proposer: 0,
        round: 0,
        time: 1,
        favor: true,
    };
    let response = state.progress(event).unwrap();
    assert_eq!(
        response,
        vec![ConsensusResponse::BroadcastPrevote {
            proposal: 0,
            round: 0
        }]
    );

    // STEP 2: Prevote.
    for validator_index in 0..=1 {
        let event = ConsensusEvent::Prevote {
            proposal: 0,
            round: 0,
            signer: validator_index,
            time: 3,
        };
        let response = state.progress(event).unwrap();
        assert!(response.is_empty());
    }
    let event = ConsensusEvent::Prevote {
        proposal: 0,
        round: 0,
        signer: 2,
        time: 3,
    };
    let response = state.progress(event).unwrap();
    assert_eq!(
        response,
        vec![ConsensusResponse::BroadcastPrecommit {
            proposal: 0,
            round: 0
        }]
    );

    // STEP 3: Precommit.
    for validator_index in 0..=1 {
        let event = ConsensusEvent::Precommit {
            proposal: 0,
            round: 0,
            signer: validator_index,
            time: 4,
        };
        let response = state.progress(event).unwrap();
        assert!(response.is_empty());
    }
    let event = ConsensusEvent::Precommit {
        proposal: 0,
        round: 0,
        signer: 2,
        time: 4,
    };
    let response = state.progress(event).unwrap();
    assert_eq!(
        response,
        vec![ConsensusResponse::FinalizeBlock { proposal: 0 }]
    );
}

#[test]
fn duplicate_prevotes_and_precommits() {
    let height_info = HeightInfo {
        validators: vec![1, 1, 1, 1, 1, 1, 1],
        this_node_index: Some(6),
        timestamp: 0,
        consensus_params: ConsensusParams {
            timeout_ms: 1000,
            repeat_round_for_first_leader: 1,
        },
        initial_block_candidate: 0,
    };
    let mut state = ConsensusState::new(height_info.clone());

    // STEP 1: Proposal.

    let event = ConsensusEvent::Start { time: 0 };
    let response = state.progress(event).unwrap();
    assert!(response.is_empty());

    let event = ConsensusEvent::BlockProposalReceived {
        proposal: 0,
        proposal_round: None,
        proposer: 0,
        round: 0,
        time: 1,
        favor: true,
    };
    let response = state.progress(event).unwrap();
    assert_eq!(
        response,
        vec![ConsensusResponse::BroadcastPrevote {
            proposal: 0,
            round: 0
        }]
    );

    // STEP 2: Duplicate Prevote.
    for _ in 0..2 {
        for validator_index in 0..=2 {
            let event = ConsensusEvent::Prevote {
                proposal: 0,
                round: 0,
                signer: validator_index,
                time: 3,
            };
            let response = state.progress(event).unwrap();
            assert!(response.is_empty());
        }
    }
    let event = ConsensusEvent::Prevote {
        proposal: 0,
        round: 0,
        signer: 3,
        time: 3,
    };
    let response = state.progress(event).unwrap();
    assert_eq!(
        response,
        vec![ConsensusResponse::BroadcastPrecommit {
            proposal: 0,
            round: 0
        }]
    );

    // STEP 3: Duplicate Precommit.
    for _ in 0..2 {
        for validator_index in 0..=2 {
            let event = ConsensusEvent::Precommit {
                proposal: 0,
                round: 0,
                signer: validator_index,
                time: 4,
            };
            let response = state.progress(event).unwrap();
            assert!(response.is_empty());
        }
    }
    let event = ConsensusEvent::Precommit {
        proposal: 0,
        round: 0,
        signer: 3,
        time: 4,
    };
    let response = state.progress(event).unwrap();
    assert_eq!(
        response,
        vec![ConsensusResponse::FinalizeBlock { proposal: 0 }]
    );
}

#[test]
fn early_termination_by_polka_2() {
    let (_height_info, mut state) = initialize(
        vec![1, 1, 1, 1, 1, 1, 1],
        Some(6),
        0,
        ConsensusParams {
            timeout_ms: 1000,
            repeat_round_for_first_leader: 1,
        },
        0,
    );

    // STEP 1: Proposal.
    let event = ConsensusEvent::Start { time: 0 };
    let response = state.progress(event).unwrap();
    assert!(response.is_empty());

    let event = ConsensusEvent::BlockProposalReceived {
        proposal: 0,
        proposal_round: None,
        proposer: 0,
        round: 0,
        time: 1,
        favor: true,
    };
    let response = state.progress(event).unwrap();
    assert_eq!(
        response,
        vec![ConsensusResponse::BroadcastPrevote {
            proposal: 0,
            round: 0
        }]
    );

    // STEP 2: Prevote.
    for validator_index in 0..=2 {
        let event = ConsensusEvent::Prevote {
            proposal: 0,
            round: 0,
            signer: validator_index,
            time: 3,
        };
        let response = state.progress(event).unwrap();
        assert!(response.is_empty());
    }
    let event = ConsensusEvent::NilPrevote {
        round: 0,
        signer: 3,
        time: 3,
    };
    let response = state.progress(event).unwrap();
    assert!(response.is_empty());

    let event = ConsensusEvent::Prevote {
        proposal: 0,
        round: 0,
        signer: 4,
        time: 3,
    };
    let response = state.progress(event).unwrap();
    assert_eq!(
        response,
        vec![ConsensusResponse::BroadcastPrecommit {
            proposal: 0,
            round: 0
        }]
    );

    // STEP 3: Precommit.
    for validator_index in 0..=2 {
        let event = ConsensusEvent::Precommit {
            proposal: 0,
            round: 0,
            signer: validator_index,
            time: 4,
        };
        let response = state.progress(event).unwrap();
        assert!(response.is_empty());
    }
    let event = ConsensusEvent::Precommit {
        proposal: 0,
        round: 0,
        signer: 3,
        time: 4,
    };
    let response = state.progress(event).unwrap();
    assert_eq!(
        response,
        vec![ConsensusResponse::FinalizeBlock { proposal: 0 }]
    );
}

#[test]
fn early_termination_by_nilpolka() {
    let height_info = HeightInfo {
        validators: vec![1, 1, 1, 1, 1, 1, 1],
        this_node_index: Some(6),
        timestamp: 0,
        consensus_params: ConsensusParams {
            timeout_ms: 1000,
            repeat_round_for_first_leader: 1,
        },
        initial_block_candidate: 0,
    };
    let mut state = ConsensusState::new(height_info.clone());

    // STEP 1: Proposal.

    let event = ConsensusEvent::Start { time: 0 };
    let response = state.progress(event).unwrap();
    assert!(response.is_empty());

    let event = ConsensusEvent::BlockProposalReceived {
        proposal: 0,
        proposal_round: None,
        proposer: 0,
        round: 0,
        time: 1,
        favor: false,
    };
    let response = state.progress(event).unwrap();
    assert_eq!(
        response,
        vec![ConsensusResponse::BroadcastNilPrevote { round: 0 }]
    );

    // STEP 2: Prevote.
    for validator_index in 1..=3 {
        let event = ConsensusEvent::NilPrevote {
            round: 0,
            signer: validator_index,
            time: 3,
        };
        let response = state.progress(event).unwrap();
        assert!(response.is_empty());
    }

    let event = ConsensusEvent::NilPrevote {
        round: 0,
        signer: 4,
        time: 3,
    };
    let response = state.progress(event).unwrap();
    assert_eq!(
        response,
        vec![ConsensusResponse::BroadcastNilPrecommit { round: 0 }]
    );

    // STEP 3: Precommit
    for validator_index in 0..=2 {
        let event = ConsensusEvent::NilPrecommit {
            round: 0,
            signer: validator_index,
            time: 4,
        };
        let response = state.progress(event).unwrap();
        assert!(response.is_empty());
    }
    let event = ConsensusEvent::NilPrecommit {
        round: 0,
        signer: 3,
        time: 4,
    };
    let response = state.progress(event).unwrap();
    assert!(response.is_empty());
}

#[test]
fn propose_timeout() {
    let height_info = HeightInfo {
        validators: vec![1, 1, 1, 1, 1, 1, 1],
        this_node_index: Some(6),
        timestamp: 0,
        consensus_params: ConsensusParams {
            timeout_ms: 1000,
            repeat_round_for_first_leader: 1,
        },
        initial_block_candidate: 0,
    };
    let mut state = ConsensusState::new(height_info.clone());

    // STEP 1: Proposal.

    let event = ConsensusEvent::Start { time: 0 };
    let response = state.progress(event).unwrap();
    assert!(response.is_empty());

    let event = ConsensusEvent::Timer {
        round: 0,
        time: 1 + height_info.consensus_params.timeout_ms as i64,
    };
    let response = state.progress(event).unwrap();
    assert_eq!(
        response,
        vec![ConsensusResponse::BroadcastNilPrevote { round: 0 }]
    );
}

#[test]
fn precommit_timeout_and_broadcast_proposal() {
    let height_info = HeightInfo {
        validators: vec![1, 1, 1, 1, 1, 1, 1],
        this_node_index: Some(1),
        timestamp: 0,
        consensus_params: ConsensusParams {
            timeout_ms: 1000,
            repeat_round_for_first_leader: 1,
        },
        initial_block_candidate: 1,
    };
    let mut state = ConsensusState::new(height_info.clone());

    // STEP 1: Proposal.

    let event = ConsensusEvent::Start { time: 0 };
    let response = state.progress(event).unwrap();
    assert!(response.is_empty());

    let event = ConsensusEvent::BlockProposalReceived {
        proposal: 0,
        proposal_round: None,
        proposer: 0,
        round: 0,
        time: 1,
        favor: false,
    };
    let response = state.progress(event).unwrap();
    assert_eq!(
        response,
        vec![ConsensusResponse::BroadcastNilPrevote { round: 0 }]
    );

    // STEP 2: Prevote.
    let event = ConsensusEvent::Prevote {
        proposal: 0,
        round: 0,
        signer: 0,
        time: 3,
    };
    let response = state.progress(event).unwrap();
    assert!(response.is_empty());

    let event = ConsensusEvent::NilPrevote {
        round: 0,
        signer: 2,
        time: 3,
    };
    let response = state.progress(event).unwrap();
    assert!(response.is_empty());

    for validator_index in 3..=4 {
        let event = ConsensusEvent::Prevote {
            proposal: 0,
            round: 0,
            signer: validator_index,
            time: 3,
        };
        let response = state.progress(event).unwrap();
        assert!(response.is_empty());
    }

    let event = ConsensusEvent::Prevote {
        proposal: 0,
        round: 0,
        signer: 5,
        time: 3,
    };
    let response = state.progress(event).unwrap();
    assert_eq!(
        response,
        vec![ConsensusResponse::BroadcastNilPrecommit { round: 0 }]
    );

    // STEP 3: Precommit.
    let event = ConsensusEvent::Precommit {
        proposal: 0,
        round: 0,
        signer: 0,
        time: 4,
    };
    let response = state.progress(event).unwrap();
    assert!(response.is_empty());

    let event = ConsensusEvent::NilPrecommit {
        round: 0,
        signer: 2,
        time: 4,
    };
    let response = state.progress(event).unwrap();
    assert!(response.is_empty());

    for validator_index in 3..=5 {
        let event = ConsensusEvent::Precommit {
            proposal: 0,
            round: 0,
            signer: validator_index,
            time: 4,
        };
        let response = state.progress(event).unwrap();
        assert!(response.is_empty());
    }
    let event = ConsensusEvent::Timer {
        round: 0,
        time: 4 + height_info.consensus_params.timeout_ms as i64,
    };
    let response = state.progress(event).unwrap();
    assert_eq!(
        response,
        vec![ConsensusResponse::BroadcastProposal {
            proposal: 1,
            round: 1
        }]
    );
}

#[test]
fn double_vote_violation() {
    let (_height_info, mut state) = initialize(
        vec![1, 1, 1, 1, 1, 1, 1],
        Some(6),
        0,
        ConsensusParams {
            timeout_ms: 1000,
            repeat_round_for_first_leader: 1,
        },
        0,
    );

    // STEP 1: Proposal.
    let event = ConsensusEvent::Start { time: 0 };
    let response = state.progress(event).unwrap();
    assert!(response.is_empty());

    let event = ConsensusEvent::BlockProposalReceived {
        proposal: 0,
        proposal_round: None,
        proposer: 0,
        round: 0,
        time: 1,
        favor: true,
    };
    let response = state.progress(event).unwrap();
    assert_eq!(
        response,
        vec![ConsensusResponse::BroadcastPrevote {
            proposal: 0,
            round: 0
        }]
    );

    // STEP 2: Prevote.
    for validator_index in 0..=2 {
        let event = ConsensusEvent::Prevote {
            proposal: 0,
            round: 0,
            signer: validator_index,
            time: 3,
        };
        let response = state.progress(event).unwrap();
        assert!(response.is_empty());
    }
    let event = ConsensusEvent::Prevote {
        proposal: 1,
        round: 0,
        signer: 2,
        time: 3,
    };
    let response = state.progress(event).unwrap();
    assert_eq!(
        response,
        vec![ConsensusResponse::ViolationReport {
            violator: 2,
            description: String::from("Duplicate Prevote")
        }]
    );
}

#[test]
fn dominant_validator() {
    //case when one validator has dominant voting power.
    unimplemented!()
}
