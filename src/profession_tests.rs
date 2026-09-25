use super::*;

// Bolt of Linen Cloth (2963): TrivialSkillLineRankLow 25, TrivialSkillLineRankHigh 50.

#[test]
fn bolt_of_linen_cloth_is_orange_below_its_low_trivial_rank() {
    assert_eq!(recipe_difficulty(1, 25, 50), RecipeDifficulty::Optimal);
    assert_eq!(recipe_difficulty(24, 25, 50), RecipeDifficulty::Optimal);
}

#[test]
fn bolt_of_linen_cloth_is_yellow_from_25_and_green_from_37() {
    assert_eq!(recipe_difficulty(25, 25, 50), RecipeDifficulty::Medium);
    assert_eq!(recipe_difficulty(36, 25, 50), RecipeDifficulty::Medium);
    assert_eq!(recipe_difficulty(37, 25, 50), RecipeDifficulty::Easy);
    assert_eq!(recipe_difficulty(49, 25, 50), RecipeDifficulty::Easy);
}

#[test]
fn bolt_of_linen_cloth_is_grey_at_50() {
    assert_eq!(recipe_difficulty(50, 25, 50), RecipeDifficulty::Trivial);
    assert_eq!(recipe_difficulty(300, 25, 50), RecipeDifficulty::Trivial);
}

#[test]
fn skill_up_chance_follows_trinitycore_defaults() {
    assert_eq!(RecipeDifficulty::Optimal.skill_up_chance_permille(), 1000);
    assert_eq!(RecipeDifficulty::Medium.skill_up_chance_permille(), 750);
    assert_eq!(RecipeDifficulty::Easy.skill_up_chance_permille(), 250);
    assert_eq!(RecipeDifficulty::Trivial.skill_up_chance_permille(), 0);
}
