mod unit_action;
use unit_action::*;
use votemint::*;
//TODO
//Change votes/commits, paramter of prevotes/precommits
//(index: usize, is_favor: bool, voting_power: u64, time: Timestamp) to (index: usize, is_favor: bool, time : Timestamp)
// get voting power from height_info

#[ignore]
#[test]
//New block with 2/3+ early termination and precommitted
fn early_termination_by_polka_1() {
    let (height_info, mut state) = initialize(
        vec![1, 1, 1, 1, 1, 1, 1],
        6,
        0,
        ConsensusParams {
            timeout_ms: 1000,
            repeat_round_for_first_leader: 0,
        },
    );
    let favor_of_this_node = (true, height_info.validators[height_info.this_node_index]);
    // STEP 1: Proposal.
    let (proposal_response, favor_response) =
        receive_and_favor_propose(&height_info, &mut state, 0, 0, 0, 1, true);

    assert!(proposal_response.is_empty());
    assert_eq!(
        favor_response,
        vec![ConsensusResponse::BroadcastPrevote {
            proposal: 0,
            round: 0
        }]
    );
    // STEP 2: Prevote.
    let votes = Vec::<(usize, bool, u64, Timestamp)>::from([
        (0, true, 1, 2),
        (1, true, 1, 2),
        (2, true, 1, 2),
        (3, true, 1, 2),
    ]);
    let mut response_seq = prevotes(&height_info, &mut state, favor_of_this_node, votes, 0, 0);
    let last_response = response_seq.pop();

    for response in response_seq {
        assert!(response.is_empty());
    }
    assert_eq!(
        last_response.unwrap(),
        vec![ConsensusResponse::BroadcastPrecommit {
            proposal: 0,
            round: 0
        }]
    );
    // STEP 3: Precommit.
    let commits = Vec::<(usize, bool, u64, Timestamp)>::from([
        (0, true, 1, 3),
        (1, true, 1, 3),
        (2, true, 1, 3),
        (3, true, 1, 3),
    ]);
    let mut response_seq = precommits(&height_info, &mut state, favor_of_this_node, commits, 0, 0);
    let last_response = response_seq.pop();

    for response in response_seq {
        assert!(response.is_empty());
    }
    assert_eq!(
        last_response.unwrap(),
        vec![ConsensusResponse::FinalizeBlock { proposal: 0 }]
    );
}

#[ignore]
#[test]
//New block with 2/3+ early termination with ununiformed voting power and precommitted
fn early_termination_by_polka_2() {
    let (height_info, mut state) = initialize(
        vec![10, 8, 6, 5, 4, 2, 2],
        3,
        0,
        ConsensusParams {
            timeout_ms: 1000,
            repeat_round_for_first_leader: 0,
        },
    );
    let favor_of_this_node = (true, height_info.validators[height_info.this_node_index]);
    // STEP 1: Proposal.
    let (proposal_response, favor_response) =
        receive_and_favor_propose(&height_info, &mut state, 0, 0, 0, 1, true);

    assert!(proposal_response.is_empty());
    assert_eq!(
        favor_response,
        vec![ConsensusResponse::BroadcastPrevote {
            proposal: 0,
            round: 0
        }]
    );

    // STEP 2: Prevote.
    let votes = Vec::<(usize, bool, u64, Timestamp)>::from([
        (0, true, 10, 2),
        (1, true, 8, 2),
        (2, true, 6, 2),
    ]);
    let mut response_seq = prevotes(&height_info, &mut state, favor_of_this_node, votes, 0, 0);
    let last_response = response_seq.pop();

    for response in response_seq {
        assert!(response.is_empty());
    }
    assert_eq!(
        last_response.unwrap(),
        vec![ConsensusResponse::BroadcastPrecommit {
            proposal: 0,
            round: 0
        }]
    );

    // STEP 3: Precommit.
    let commits = Vec::<(usize, bool, u64, Timestamp)>::from([
        (0, true, 10, 4),
        (1, true, 8, 4),
        (2, true, 6, 4),
    ]);
    let mut response_seq = precommits(&height_info, &mut state, favor_of_this_node, commits, 0, 0);
    let last_response = response_seq.pop();

    for response in response_seq {
        assert!(response.is_empty());
    }
    assert_eq!(
        last_response.unwrap(),
        vec![ConsensusResponse::FinalizeBlock { proposal: 0 }]
    );
}

#[ignore]
#[test]

//New block with 5/6+ e.t. and precommitted
fn ternination_by_polka() {
    let (height_info, mut state) = initialize(
        vec![1, 1, 1, 1, 1, 1, 1],
        6,
        0,
        ConsensusParams {
            timeout_ms: 1000,
            repeat_round_for_first_leader: 0,
        },
    );
    let favor_of_this_node = (true, height_info.validators[height_info.this_node_index]);

    // STEP 1: Proposal.
    let (proposal_response, favor_response) =
        receive_and_favor_propose(&height_info, &mut state, 0, 0, 0, 1, true);

    assert!(proposal_response.is_empty());
    assert_eq!(
        favor_response,
        vec![ConsensusResponse::BroadcastPrevote {
            proposal: 0,
            round: 0
        }]
    );

    // STEP 2: Prevote.
    let votes = Vec::<(usize, bool, u64, Timestamp)>::from([
        (0, true, 1, 2),
        (1, true, 1, 2),
        (2, true, 1, 2),
        (3, true, 1, 3),
        (4, false, 1, 2),
    ]);
    let mut response_seq = prevotes(&height_info, &mut state, favor_of_this_node, votes, 0, 0);
    let last_response = response_seq.pop();

    for response in response_seq {
        assert!(response.is_empty());
    }
    assert_eq!(
        last_response.unwrap(),
        vec![ConsensusResponse::BroadcastPrecommit {
            proposal: 0,
            round: 0
        }]
    );

    // STEP 3: Precommit.
    let commits = Vec::<(usize, bool, u64, Timestamp)>::from([
        (0, true, 1, 5),
        (1, true, 1, 5),
        (2, true, 1, 5),
        (3, true, 1, 6),
        (4, false, 1, 5),
    ]);
    let mut response_seq = precommits(&height_info, &mut state, favor_of_this_node, commits, 0, 0);
    let last_response = response_seq.pop();

    for response in response_seq {
        assert!(response.is_empty());
    }
    assert_eq!(
        last_response.unwrap(),
        vec![ConsensusResponse::FinalizeBlock { proposal: 0 }]
    );
}

#[ignore]
#[test]
// New round with 2/3+ prevote e.t. and nil committed due to timeout.
fn early_termination_and_timeout() {
    let (height_info, mut state) = initialize(
        vec![1, 1, 1, 1, 1, 1, 1],
        6,
        0,
        ConsensusParams {
            timeout_ms: 1000,
            repeat_round_for_first_leader: 0,
        },
    );
    let favor_of_this_node = (true, height_info.validators[height_info.this_node_index]);

    // STEP 1: Proposal.
    let (proposal_response, favor_response) =
        receive_and_favor_propose(&height_info, &mut state, 0, 0, 0, 1, true);

    assert!(proposal_response.is_empty());
    assert_eq!(
        favor_response,
        vec![ConsensusResponse::BroadcastPrevote {
            proposal: 0,
            round: 0
        }]
    );

    // STEP 2: Prevote.
    let votes = Vec::<(usize, bool, u64, Timestamp)>::from([
        (0, true, 1, 2),
        (1, true, 1, 2),
        (2, true, 1, 2),
        (3, true, 1, 2),
    ]);
    let mut response_seq = prevotes(&height_info, &mut state, favor_of_this_node, votes, 0, 0);
    let last_response = response_seq.pop();

    for response in response_seq {
        assert!(response.is_empty());
    }
    assert_eq!(
        last_response.unwrap(),
        vec![ConsensusResponse::BroadcastPrecommit {
            proposal: 0,
            round: 0
        }]
    );

    //Step3: Timeout
    let timeout = height_info.consensus_params.timeout_ms as i64;
    let event = ConsensusEvent::Timer { time: timeout };
    let response = progress(&height_info, &mut state, event);
    assert_eq!(
        response,
        vec![ConsensusResponse::BroadcastPrevote {
            proposal: 0,
            round: 1
        }]
    );
}

#[ignore]
#[test]
// New round with 5/6+ prevote e.t. and not committed.
fn termination_and_timeout() {
    let (height_info, mut state) = initialize(
        vec![1, 1, 1, 1, 1, 1, 1],
        6,
        0,
        ConsensusParams {
            timeout_ms: 1000,
            repeat_round_for_first_leader: 0,
        },
    );
    let favor_of_this_node = (true, height_info.validators[height_info.this_node_index]);

    // STEP 1: Proposal.
    let (proposal_response, favor_response) =
        receive_and_favor_propose(&height_info, &mut state, 0, 0, 0, 1, true);

    assert!(proposal_response.is_empty());
    assert_eq!(
        favor_response,
        vec![ConsensusResponse::BroadcastPrevote {
            proposal: 0,
            round: 0
        }]
    );

    let event = ConsensusEvent::NilPrevote {
        signer: 4,
        round: 0,
        time: 3,
    };

    let response = progress(&height_info, &mut state, event);
    assert!(response.is_empty());

    // STEP 2: Prevote.
    let votes = Vec::<(usize, bool, u64, Timestamp)>::from([
        (0, true, 1, 2),
        (1, true, 1, 2),
        (2, true, 1, 2),
        (3, true, 1, 3),
        (4, false, 1, 2),
    ]);
    let mut response_seq = prevotes(&height_info, &mut state, favor_of_this_node, votes, 0, 0);
    let last_response = response_seq.pop();

    for response in response_seq {
        assert!(response.is_empty());
    }
    assert_eq!(
        last_response.unwrap(),
        vec![ConsensusResponse::BroadcastPrecommit {
            proposal: 0,
            round: 0
        }]
    );

    //Step3: Timeout
    let timeout = height_info.consensus_params.timeout_ms as i64;
    let event = ConsensusEvent::Timer { time: timeout };
    let response = progress(&height_info, &mut state, event);

    assert_eq!(
        response,
        vec![ConsensusResponse::BroadcastPrevote {
            proposal: 0,
            round: 1
        }]
    );
}

#[ignore]
#[test]
//New round with 2/3+ e.t. and nil committed
fn early_termination_by_nil_polka() {
    let (height_info, mut state) = initialize(
        vec![1, 1, 1, 1, 1, 1, 1],
        6,
        0,
        ConsensusParams {
            timeout_ms: 1000,
            repeat_round_for_first_leader: 0,
        },
    );
    let favor_of_this_node = (false, height_info.validators[height_info.this_node_index]);

    // STEP 1: Proposal.
    let (proposal_response, favor_response) =
        receive_and_favor_propose(&height_info, &mut state, 0, 0, 0, 1, true);

    assert!(proposal_response.is_empty());
    assert_eq!(
        favor_response,
        vec![ConsensusResponse::BroadcastNilPrevote { round: 0 }]
    );

    // STEP 2: Prevote.
    let votes = Vec::<(usize, bool, u64, Timestamp)>::from([
        (0, false, 1, 2),
        (1, false, 1, 2),
        (2, false, 1, 2),
        (3, false, 1, 2),
    ]);
    let mut response_seq = prevotes(&height_info, &mut state, favor_of_this_node, votes, 0, 0);
    let last_response = response_seq.pop();

    for response in response_seq {
        assert!(response.is_empty());
    }
    assert_eq!(
        last_response.unwrap(),
        vec![ConsensusResponse::BroadcastNilPrecommit { round: 0 }]
    );

    // STEP 3: Precommit.
    let commits = Vec::<(usize, bool, u64, Timestamp)>::from([
        (0, false, 1, 5),
        (1, false, 1, 5),
        (2, false, 1, 5),
        (3, false, 1, 5),
    ]);
    let mut response_seq = precommits(&height_info, &mut state, favor_of_this_node, commits, 0, 0);
    let last_response = response_seq.pop();

    for response in response_seq {
        assert!(response.is_empty());
    }
    assert_ne!(
        last_response.unwrap(),
        vec![ConsensusResponse::FinalizeBlock { proposal: 0 }]
    );
}

#[ignore]
#[test]
//Next round with 5/6+ e.t. and nil committed
fn early_termination_and_prevote_to_nil() {
    let (height_info, mut state) = initialize(
        vec![1, 1, 1, 1, 1, 1, 1],
        6,
        0,
        ConsensusParams {
            timeout_ms: 1000,
            repeat_round_for_first_leader: 0,
        },
    );
    let favor_of_this_node = (false, height_info.validators[height_info.this_node_index]);

    // STEP 1: Proposal.
    // STEP 1: Proposal.
    let (proposal_response, favor_response) =
        receive_and_favor_propose(&height_info, &mut state, 0, 0, 0, 1, true);

    assert!(proposal_response.is_empty());
    assert_eq!(
        favor_response,
        vec![ConsensusResponse::BroadcastNilPrevote { round: 0 }]
    );

    // STEP 2: Prevote.
    let votes = Vec::<(usize, bool, u64, Timestamp)>::from([
        (0, false, 1, 2),
        (1, false, 1, 2),
        (2, false, 1, 2),
        (3, false, 1, 3),
        (4, true, 1, 2),
    ]);
    let mut response_seq = prevotes(&height_info, &mut state, favor_of_this_node, votes, 0, 0);
    let last_response = response_seq.pop();

    for response in response_seq {
        assert!(response.is_empty());
    }
    assert_eq!(
        last_response.unwrap(),
        vec![ConsensusResponse::BroadcastNilPrecommit { round: 0 }]
    );

    // STEP 3: Precommit.
    let commits = Vec::<(usize, bool, u64, Timestamp)>::from([
        (0, false, 1, 5),
        (1, false, 1, 5),
        (2, false, 1, 5),
        (3, false, 1, 6),
        (4, true, 1, 5),
    ]);
    let mut response_seq = precommits(&height_info, &mut state, favor_of_this_node, commits, 0, 0);
    let last_response = response_seq.pop();

    for response in response_seq {
        assert!(response.is_empty());
    }
    assert_ne!(
        last_response.unwrap(),
        vec![ConsensusResponse::FinalizeBlock { proposal: 0 }]
    );
}

#[ignore]
#[test]
//TODO
// Case when node which is not leader broadcasts its proposal.
fn violation_diff_proposer() {
    unimplemented!();
}

#[ignore]
#[test]
// Violation that same node proposes multiple proposal
fn violation_same_proposer() {
    let (height_info, mut state) = initialize(
        vec![1, 1, 1, 1, 1, 1, 1],
        6,
        0,
        ConsensusParams {
            timeout_ms: 1000,
            repeat_round_for_first_leader: 0,
        },
    );

    // STEP 1: Proposal.
    let (proposal_response, favor_response) =
        receive_and_favor_propose(&height_info, &mut state, 0, 0, 0, 1, true);

    assert!(proposal_response.is_empty());
    assert_eq!(
        favor_response,
        vec![ConsensusResponse::BroadcastPrevote {
            proposal: 0,
            round: 0
        }]
    );

    let fake_proposal_response = get_response_from_event(
        &height_info,
        &mut state,
        ConsensusEvent::BlockProposal {
            proposal: 1,
            proposer: 0,
            round: 0,
            time: 1,
        },
    );
    assert_eq!(
        fake_proposal_response,
        vec![ConsensusResponse::ViolationReport {
            violator: 0,
            description: "".to_string(),
        }]
    );
}

//TODO
//Case when next block finalized. On the otherhand, this node did not recognize finalized block due to network error.
//In this case, state would be in Precommit stage.
//I'd like to check whether state transition occurs from Precommit to Proposal when BlockProposal Event occured.
#[ignore]
#[test]
fn get_informed_new_block() {
    unimplemented!();
}

//TODO
//Case when node that does not know finalized block asserts outdated proposal.
//Detailed Description as follows
//state of this node would be pended in Precommit Stage(or Prevote Stage) due to network disconnection
//On the contrary, other nodes finalized block, and they are now in Prevote Stage.
//When this node receives new proposal, it will update height information, round, ...etc
//I want to check whether updated state is Prevote with correct information.
#[ignore]
#[test]
fn create_new_block() {
    unimplemented!();
}
