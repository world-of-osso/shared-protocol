use shared::protocol::{PetBattleQueueRequest, PetBattleQueueState, PetBattleQueueUpdate};

#[test]
fn petbattle_pvp_queue_proposal_roundtrips_on_lightyear_wire() {
    for request in [
        PetBattleQueueRequest::Join,
        PetBattleQueueRequest::Leave,
        PetBattleQueueRequest::Accept { proposal_id: 41 },
        PetBattleQueueRequest::Decline { proposal_id: 41 },
    ] {
        let bytes = bincode::serde::encode_to_vec(request, bincode::config::standard()).unwrap();
        let (decoded, consumed): (PetBattleQueueRequest, _) =
            bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
        assert_eq!(decoded, request);
        assert_eq!(consumed, bytes.len());
    }
    for state in [
        PetBattleQueueState::Idle,
        PetBattleQueueState::Queued,
        PetBattleQueueState::Proposal {
            proposal_id: 41,
            accepted: false,
        },
        PetBattleQueueState::Proposal {
            proposal_id: 41,
            accepted: true,
        },
        PetBattleQueueState::InBattle,
    ] {
        let update = PetBattleQueueUpdate { state, error: None };
        let bytes = bincode::serde::encode_to_vec(&update, bincode::config::standard()).unwrap();
        let (decoded, consumed): (PetBattleQueueUpdate, _) =
            bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
        assert_eq!(decoded, update);
        assert_eq!(consumed, bytes.len());
    }
}
