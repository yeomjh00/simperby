use super::*;

/// TODO: we have to implement the following missing logics
/// - on-proposal -> search TBC
/// - on-4f-favor-prevote-propose-step
/// - on-4f-favor-prevote-prevote-step
/// - on-4f-nil-prevote -> to be checked
/// - on-5f-precommit
/// - on-4f-favor-precommit
/// - OnTimeoutPrecommit -> to be checked
pub(super) fn progress(
    height_info: &HeightInfo,
    state: &mut ConsensusState,
    event: ConsensusEvent,
) -> Option<Vec<ConsensusResponse>> {
    let result = if state.waiting_for_proposal_creation {
        match event {
            ConsensusEvent::BlockProposalCreated {
                proposal, round, ..
            } => {
                if state.round != round {
                    return None;
                }
                state.waiting_for_proposal_creation = false;
                vec![ConsensusResponse::BroadcastProposal {
                    proposal,
                    round: state.round,
                }]
            }
            _ => {
                // Nothing to do; this state is waiting for a `BlockProposalCreated`.
                return None;
            }
        }
    } else {
        match event {
            ConsensusEvent::Start { time } => match start_round(height_info, state, 0, time) {
                StartRoundResponse::Normal(r) => r,
                StartRoundResponse::Pending { .. } => {
                    state.waiting_for_proposal_creation = true;
                    Vec::new()
                }
            },
            ConsensusEvent::BlockProposalCreated { .. } => return None,
            ConsensusEvent::BlockProposalReceived {
                proposal,
                proposal_round,
                proposer,
                round,
                ..
            } => {
                let current_proposer = decide_proposer(round, height_info);
                if proposer == current_proposer && state.step == ConsensusStep::Propose {
                    match proposal_round {
                        Some(vr) => {
                            if vr < round {
                                on_4f_favor_prevote_propose(proposal, height_info, state, round, vr)
                            } else {
                                Vec::new()
                            }
                        }
                        None => on_proposal(proposal, state, round),
                    }
                } else {
                    Vec::new()
                }
            }
            ConsensusEvent::ProposalFavor {
                proposal, favor, ..
            } => {
                if state.step == ConsensusStep::Propose {
                    on_proposal_favor(proposal, favor, height_info, state, state.round)
                } else {
                    Vec::new()
                }
            }
            // Time-trigger events are handled later
            ConsensusEvent::Timer { .. } => Vec::new(),
            ConsensusEvent::Prevote {
                proposal,
                signer,
                round,
                ..
            } => {
                let total_voting_power = height_info.validators.iter().sum::<VotingPower>();
                if round != state.round {
                    return None;
                }
                let voting_power = height_info.validators[signer as usize];
                state.votes.insert(round, {
                    let mut votes = state
                        .votes
                        .get(&round)
                        .unwrap_or(&Default::default())
                        .clone();
                    votes.prevotes_total += voting_power;
                    votes.prevotes_favor.insert(
                        proposal,
                        votes.prevotes_favor.get(&proposal).unwrap_or(&0) + voting_power,
                    );
                    votes
                });
                if state.votes[&round].prevotes_total * 6 > total_voting_power * 5
                    && state.step == ConsensusStep::Prevote
                {
                    on_5f_prevote(height_info, state, round)
                } else if state.votes[&round].prevotes_total * 3 > total_voting_power * 2
                    && (state.step == ConsensusStep::Prevote
                        || state.step == ConsensusStep::Precommit)
                {
                    on_4f_prevote(height_info, state, round)
                } else {
                    Vec::new()
                }
            }

            ConsensusEvent::NilPrevote { signer, round, .. } => {
                let total_voting_power = height_info.validators.iter().sum::<VotingPower>();
                if round != state.round {
                    return None;
                }
                let voting_power = height_info.validators[signer as usize];
                state.votes.insert(round, {
                    let mut votes = state
                        .votes
                        .get(&round)
                        .unwrap_or(&Default::default())
                        .clone();
                    votes.prevotes_total += voting_power;
                    votes
                });

                if state.votes[&round].prevotes_total * 6 > total_voting_power * 5
                    && state.step == ConsensusStep::Prevote
                {
                    on_5f_prevote(height_info, state, round)
                } else if state.votes[&round].prevotes_total * 3 > total_voting_power * 2
                    && state.step == ConsensusStep::Prevote
                {
                    on_4f_nilprevote(height_info, state, round)
                } else {
                    Vec::new()
                }
            }

            ConsensusEvent::Precommit {
                proposal,
                signer,
                round,
                time,
            } => {
                let total_voting_power = height_info.validators.iter().sum::<VotingPower>();
                if round != state.round {
                    return None;
                }
                let voting_power = height_info.validators[signer as usize];
                state.votes.insert(round, {
                    let mut votes = state
                        .votes
                        .get(&round)
                        .unwrap_or(&Default::default())
                        .clone();
                    votes.precommits_total += voting_power;
                    votes.precommits_favor.insert(
                        proposal,
                        votes.precommits_favor.get(&proposal).unwrap_or(&0) + voting_power,
                    );
                    votes
                });
                if state.votes[&round].precommits_total * 6 > total_voting_power * 5
                    && ConsensusStep::Precommit == state.step
                    && state.timeout_precommit == None
                {
                    on_5f_precommit(height_info, state, time)
                } else if state.votes[&round].precommits_total * 3 > total_voting_power * 2
                    && ConsensusStep::Precommit == state.step
                {
                    on_4f_favor_precommit(height_info, state, round)
                } else {
                    Vec::new()
                }
            }

            ConsensusEvent::NilPrecommit {
                signer,
                round,
                time,
            } => {
                let total_voting_power = height_info.validators.iter().sum::<VotingPower>();
                if round != state.round {
                    return None;
                }
                let voting_power = height_info.validators[signer as usize];
                state.votes.insert(round, {
                    let mut votes = state
                        .votes
                        .get(&round)
                        .unwrap_or(&Default::default())
                        .clone();
                    votes.precommits_total += voting_power;
                    votes
                });
                //No 4f+1 early termination.
                if state.votes[&round].precommits_total * 6 > total_voting_power * 5
                    && ConsensusStep::Precommit == state.step
                    && state.timeout_precommit == None
                {
                    on_5f_precommit(height_info, state, time)
                } else {
                    Vec::new()
                }
            }
        }
    };

    if !result.is_empty() {
        Some(result)
    // Handle timeout
    } else {
        let time = event.time();
        let mut responses = Vec::new();
        match state.step {
            ConsensusStep::Propose => {
                if let Some(timeout_propose) = state.timeout_propose {
                    if time >= timeout_propose {
                        responses.append(&mut on_timeout_propose(height_info, state, state.round));
                    }
                }
            }
            ConsensusStep::Precommit => {
                if let Some(timeout_precommit) = state.timeout_precommit {
                    if time >= timeout_precommit {
                        responses.append(&mut on_timeout_precommit(
                            height_info,
                            state,
                            state.round,
                            time,
                        ));
                    }
                }
            }
            _ => (),
        }
        Some(responses)
    }
}

enum StartRoundResponse {
    Normal(Vec<ConsensusResponse>),
    /// Emits a `CreateProposal`.
    Pending,
}

fn start_round(
    height_info: &HeightInfo,
    state: &mut ConsensusState,
    round: usize,
    time: Timestamp,
) -> StartRoundResponse {
    state.round = round;
    state.step = ConsensusStep::Propose;
    state.timeout_precommit = None;
    let proposer = decide_proposer(round, height_info);
    if proposer == height_info.this_node_index {
        if let Some(valid_value) = state.valid_value {
            StartRoundResponse::Normal(vec![ConsensusResponse::BroadcastProposal {
                proposal: valid_value,
                round,
            }])
        } else {
            StartRoundResponse::Pending
        }
    } else {
        state.timeout_propose = Some(time + height_info.consensus_params.timeout_ms as i64);
        StartRoundResponse::Normal(Vec::new())
    }
}

fn on_proposal(
    proposal: BlockIdentifier,
    state: &mut ConsensusState,
    round: Round,
) -> Vec<ConsensusResponse> {
    let some_favor = state.proposal_favors.get(&proposal);
    match some_favor {
        Some(favor) => {
            state.step = ConsensusStep::Prevote;
            if Some(proposal) == state.locked_value || (*favor && state.locked_round == None) {
                vec![ConsensusResponse::BroadcastPrevote { proposal, round }]
            } else {
                vec![ConsensusResponse::BroadcastNilPrevote { round }]
            }
        }
        None => Vec::new(),
    }
}

fn on_proposal_favor(
    proposal: BlockIdentifier,
    favor: bool,
    height_info: &HeightInfo,
    state: &mut ConsensusState,
    round: Round,
) -> Vec<ConsensusResponse> {
    state.step = ConsensusStep::Prevote;
    state.proposal_favors.insert(proposal, favor);
    let this_node_voting_power = height_info.validators[height_info.this_node_index];
    state.votes.insert(round, {
        let mut votes = state
            .votes
            .get(&round)
            .unwrap_or(&Default::default())
            .clone();
        votes.prevotes_total += this_node_voting_power;
        votes.precommits_total += this_node_voting_power;
        if favor {
            votes.prevotes_favor.insert(
                proposal,
                votes.prevotes_favor.get(&proposal).unwrap_or(&0) + this_node_voting_power,
            );
            votes.precommits_favor.insert(
                proposal,
                votes.precommits_favor.get(&proposal).unwrap_or(&0) + this_node_voting_power,
            );
        }
        votes
    });
    if Some(proposal) == state.locked_value || (favor && state.locked_round == None) {
        vec![ConsensusResponse::BroadcastPrevote { proposal, round }]
    } else {
        vec![ConsensusResponse::BroadcastNilPrevote { round }]
    }
}

fn on_4f_favor_prevote_propose(
    proposal: BlockIdentifier,
    height_info: &HeightInfo,
    state: &mut ConsensusState,
    round: Round,
    valid_round: Round,
) -> Vec<ConsensusResponse> {
    let total_voting_power = height_info.validators.iter().sum::<VotingPower>();
    let locekd_prevotes = state.votes[&valid_round]
        .prevotes_favor
        .get(&proposal)
        .unwrap_or(&0);
    let favor_proposal = state.proposal_favors.get(&proposal).unwrap_or(&false);

    if locekd_prevotes * 3 > total_voting_power * 2 {
        state.step = ConsensusStep::Prevote;
        if Some(proposal) == state.locked_value
            || (*favor_proposal && state.locked_round.unwrap_or(&valid_round + 1) < valid_round)
        {
            vec![ConsensusResponse::BroadcastPrevote { proposal, round }]
        } else {
            vec![ConsensusResponse::BroadcastNilPrevote { round }]
        }
    } else {
        Vec::new()
    }
}

fn on_4f_prevote(
    height_info: &HeightInfo,
    state: &mut ConsensusState,
    round: Round,
) -> Vec<ConsensusResponse> {
    let mut responses = Vec::new();
    // locked value is set only once in a height
    if state.locked_value == None {
        let total_voting_power = height_info.validators.iter().sum::<VotingPower>();
        for (proposal, prevotes_favor) in &state.votes[&round].prevotes_favor {
            if prevotes_favor * 3 > total_voting_power * 2 {
                state.valid_round = Some(round);
                state.valid_value = Some(*proposal);
                if state.step == ConsensusStep::Prevote {
                    state.step = ConsensusStep::Precommit;
                    state.locked_round = Some(round);
                    state.locked_value = Some(*proposal);
                    responses.append(&mut vec![ConsensusResponse::BroadcastPrecommit {
                        proposal: *proposal,
                        round: state.round,
                    }]);
                }
            }
        }
    }
    responses
}

fn on_4f_nilprevote(
    height_info: &HeightInfo,
    state: &mut ConsensusState,
    round: Round,
) -> Vec<ConsensusResponse> {
    let total_voting_power = height_info.validators.iter().sum::<VotingPower>();
    let current_prevotes = &state.votes[&round].prevotes_total;
    let current_favor_prevotes = state.votes[&round]
        .prevotes_favor
        .values()
        .into_iter()
        .sum::<VotingPower>();
    let current_nil_votes = current_prevotes - current_favor_prevotes;

    if current_nil_votes * 3 > total_voting_power * 2 {
        state.step = ConsensusStep::Precommit;
        vec![ConsensusResponse::BroadcastNilPrecommit { round: state.round }]
    } else {
        Vec::new()
    }
}

fn on_5f_prevote(
    height_info: &HeightInfo,
    state: &mut ConsensusState,
    round: Round,
) -> Vec<ConsensusResponse> {
    let total_voting_power = height_info.validators.iter().sum::<VotingPower>();
    state.step = ConsensusStep::Precommit;
    for (proposal, prevotes_favor) in &state.votes[&round].prevotes_favor {
        if prevotes_favor * 3 > total_voting_power * 2 {
            return vec![ConsensusResponse::BroadcastPrecommit {
                proposal: *proposal,
                round: state.round,
            }];
        }
    }
    vec![ConsensusResponse::BroadcastNilPrecommit { round: state.round }]
}

fn on_5f_precommit(
    height_info: &HeightInfo,
    state: &mut ConsensusState,
    time: Timestamp,
) -> Vec<ConsensusResponse> {
    state.timeout_precommit = Some(time + height_info.consensus_params.timeout_ms as i64);
    Vec::new()
}

fn on_4f_favor_precommit(
    height_info: &HeightInfo,
    state: &mut ConsensusState,
    round: Round,
) -> Vec<ConsensusResponse> {
    let total_voting_power = height_info.validators.iter().sum::<VotingPower>();
    for (proposal, precommits_favor) in &state.votes[&round].precommits_favor {
        if precommits_favor * 3 > total_voting_power * 2 {
            //update validator and state will be performed out of progress
            return vec![ConsensusResponse::FinalizeBlock {
                proposal: *proposal,
            }];
        }
    }
    Vec::new()
}

fn on_timeout_propose(
    _height_info: &HeightInfo,
    state: &mut ConsensusState,
    round: usize,
) -> Vec<ConsensusResponse> {
    if state.round == round && state.step == ConsensusStep::Propose {
        state.step = ConsensusStep::Prevote;
        state.timeout_propose = None;
        vec![ConsensusResponse::BroadcastNilPrevote { round }]
    } else {
        Vec::new()
    }
}

fn on_timeout_precommit(
    height_info: &HeightInfo,
    state: &mut ConsensusState,
    round: usize,
    time: Timestamp,
) -> Vec<ConsensusResponse> {
    if state.round == round && state.step == ConsensusStep::Precommit {
        state.step = ConsensusStep::Propose;
        state.timeout_precommit = None;
        match start_round(height_info, state, round + 1, time) {
            StartRoundResponse::Normal(r) => r,
            StartRoundResponse::Pending => Vec::new(),
        }
    } else {
        Vec::new()
    }
}
