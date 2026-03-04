//! # Fight Stats Entity
//!
//! Defines a struct which represents fight statistics for a fighter.

#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct FightStats {
    pub id: Option<i64>,
    pub fighter_id: Option<i64>,
    pub fight_id: Option<i64>,
    pub knock_downs: Option<i64>,
    pub total_strikes_hit: Option<i64>,
    pub total_strikes_missed: Option<i64>,
    pub sig_strikes: Option<i64>,
    pub head_strikes: Option<i64>,
    pub body_strikes: Option<i64>,
    pub leg_strikes: Option<i64>,
    pub time_in_control: Option<i64>,
    pub takedowns_hit: Option<i64>,
    pub takedowns_missed: Option<i64>,
    pub submissions_hit: Option<i64>,
    pub submissions_missed: Option<i64>,
}
