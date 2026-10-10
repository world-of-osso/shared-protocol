use shared::protocol::*;

#[test]
fn petbattle_effect_hud_wire_preserves_weather_team_hazards_and_lockout() {
    let aura = serde_json::json!({"ability_id":189,"name":"Cyclone","icon":136022,"rounds_remaining":4,"is_buff":false});
    let weather = serde_json::json!({"ability_id":403,"name":"Sunny Day","icon":133784,"rounds_remaining":8,"is_buff":true});
    let ability = serde_json::json!({"id":404,"name":"Sunlight","icon":133784,"family":6,"cooldown":2,"lockout":3,"usable":false});
    let pet = serde_json::json!({"instance_id":41,"species_id":39,"name":"Mechanical Squirrel","display_id":7937,"icon":132145,"quality":3,"xp":0,"next_level_xp":100,"family":9,"level":7,"health":480,"max_health":480,"power":52,"speed":61,"abilities":[ability.clone(),ability.clone(),ability],"auras":[]});
    let input = serde_json::json!({"battle_id":100,"round":4,"teams":[[pet.clone()],[pet]],"active":[0,0],"wild_creature":99,"can_trap":false,"turn_time_ms":0,"initial_selection_required":false,"replacement_required":false,"feedback":[],"weather":weather,"team_auras":[[],[aura]]});
    let state: WildPetBattleSnapshot = serde_json::from_value(input.clone()).unwrap();
    let bytes = bincode::serde::encode_to_vec(&state, bincode::config::standard()).unwrap();
    let (decoded, consumed): (WildPetBattleSnapshot, usize) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(consumed, bytes.len());
    assert_eq!(serde_json::to_value(decoded).unwrap(), input);
}
