use shared::death::DeathState;

#[test]
fn ghoststate_life_states_round_trip() {
    for state in [DeathState::Alive, DeathState::Dead, DeathState::Ghost] {
        assert_eq!(
            bitcode::decode::<DeathState>(&bitcode::encode(&state)).unwrap(),
            state
        );
        let config = bincode::config::standard();
        let bytes = bincode::serde::encode_to_vec(state, config).unwrap();
        let (decoded, read): (DeathState, usize) =
            bincode::serde::decode_from_slice(&bytes, config).unwrap();
        assert_eq!(decoded, state);
        assert_eq!(read, bytes.len());
    }
}
