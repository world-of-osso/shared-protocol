use shared::components::{UnitFlags, UnitFlags2, UnitFlags3};

#[test]
fn executed_unit_flag_fields_round_trip_independently_without_losing_bits() {
    let flags = UnitFlags(UnitFlags::IMMUNE_TO_PC | UnitFlags::PREVENT_EMOTES_FROM_CHAT_TEXT);
    let flags2 = UnitFlags2(4 | UnitFlags2::FEIGN_DEATH);
    let flags3 = UnitFlags3(2 | UnitFlags3::FAKE_DEAD);
    assert_eq!(
        bitcode::decode::<UnitFlags>(&bitcode::encode(&flags)).unwrap(),
        flags
    );
    assert_eq!(
        bitcode::decode::<UnitFlags2>(&bitcode::encode(&flags2)).unwrap(),
        flags2
    );
    assert_eq!(
        bitcode::decode::<UnitFlags3>(&bitcode::encode(&flags3)).unwrap(),
        flags3
    );
    assert_eq!(flags.0, 0x2000_0100);
    assert_eq!(flags2.0, 5);
    assert_eq!(flags3.0, 0x2002);
}
