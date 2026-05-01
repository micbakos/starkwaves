pub(crate) const COMMIT_TIMEOUT_SECS: u64 = 120;
pub(crate) const TURN_TIMEOUT_SECS: u64 = 60;
pub(crate) const REVEAL_TIMEOUT_SECS: u64 = 120;

#[derive(Debug, Drop, Serde, Copy)]
pub enum GamePhase {
    Committing,
    Playing,
    Revealing,
}

#[generate_trait]
pub impl GamePhaseImpl of GamePhaseTrait {
    fn timeout(self: @GamePhase) -> u64 {
        match self {
            GamePhase::Committing => COMMIT_TIMEOUT_SECS,
            GamePhase::Playing => TURN_TIMEOUT_SECS,
            GamePhase::Revealing => REVEAL_TIMEOUT_SECS,
        }
    }

    fn is_timeout_reached(self: @GamePhase, now: u64, last_action_at: u64) -> bool {
        now > last_action_at + Self::timeout(self)
    }
}

#[derive(Debug, Drop, Serde, Copy)]
pub struct TimeoutConfig {
    pub committing: u64,
    pub playing: u64,
    pub revealing: u64,
}

impl TimeoutConfigDefault of Default<TimeoutConfig> {
    fn default() -> TimeoutConfig {
        TimeoutConfig {
            committing: COMMIT_TIMEOUT_SECS,
            playing: TURN_TIMEOUT_SECS,
            revealing: REVEAL_TIMEOUT_SECS,
        }
    }
}

