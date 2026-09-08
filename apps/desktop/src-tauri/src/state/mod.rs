use serde::{Deserialize, Serialize};
use std::fmt;
use crate::errors::{AppError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionState {
    LoggedOut,
    ShiftReady,
    Working,
    ManualBreak,
    ShiftCompleting,
    ShiftSummary,
}

impl fmt::Display for SessionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionState::LoggedOut => write!(f, "LOGGED_OUT"),
            SessionState::ShiftReady => write!(f, "SHIFT_READY"),
            SessionState::Working => write!(f, "WORKING"),
            SessionState::ManualBreak => write!(f, "MANUAL_BREAK"),
            SessionState::ShiftCompleting => write!(f, "SHIFT_COMPLETING"),
            SessionState::ShiftSummary => write!(f, "SHIFT_SUMMARY"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StateMachine {
    current_state: SessionState,
}

impl StateMachine {
    pub fn new(initial_state: SessionState) -> Self {
        Self {
            current_state: initial_state,
        }
    }

    pub fn current_state(&self) -> SessionState {
        self.current_state
    }

    pub fn can_transition_to(&self, next_state: SessionState) -> bool {
        match (self.current_state, next_state) {
            (SessionState::LoggedOut, SessionState::ShiftReady) => true,
            (SessionState::ShiftReady, SessionState::Working) => true,
            (SessionState::Working, SessionState::ManualBreak) => true,
            (SessionState::ManualBreak, SessionState::Working) => true,
            (SessionState::Working, SessionState::ShiftCompleting) => true,
            (SessionState::ManualBreak, SessionState::ShiftCompleting) => true, // End shift during break
            (SessionState::ShiftCompleting, SessionState::ShiftSummary) => true,
            (SessionState::ShiftSummary, SessionState::ShiftReady) => true,
            (SessionState::ShiftReady, SessionState::LoggedOut) => true,
            _ => false,
        }
    }

    pub fn transition_to(&mut self, next_state: SessionState) -> Result<SessionState> {
        if self.can_transition_to(next_state) {
            self.current_state = next_state;
            Ok(self.current_state)
        } else {
            Err(AppError::InvalidStateTransition(format!(
                "Cannot transition from {} to {}",
                self.current_state, next_state
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        let mut sm = StateMachine::new(SessionState::LoggedOut);

        assert!(sm.transition_to(SessionState::ShiftReady).is_ok());
        assert_eq!(sm.current_state(), SessionState::ShiftReady);

        assert!(sm.transition_to(SessionState::Working).is_ok());
        assert_eq!(sm.current_state(), SessionState::Working);

        assert!(sm.transition_to(SessionState::ManualBreak).is_ok());
        assert_eq!(sm.current_state(), SessionState::ManualBreak);

        assert!(sm.transition_to(SessionState::Working).is_ok());
        assert_eq!(sm.current_state(), SessionState::Working);

        assert!(sm.transition_to(SessionState::ShiftCompleting).is_ok());
        assert_eq!(sm.current_state(), SessionState::ShiftCompleting);

        assert!(sm.transition_to(SessionState::ShiftSummary).is_ok());
        assert_eq!(sm.current_state(), SessionState::ShiftSummary);

        assert!(sm.transition_to(SessionState::ShiftReady).is_ok());
        assert_eq!(sm.current_state(), SessionState::ShiftReady);

        assert!(sm.transition_to(SessionState::LoggedOut).is_ok());
        assert_eq!(sm.current_state(), SessionState::LoggedOut);
    }

    #[test]
    fn test_invalid_transitions() {
        let mut sm = StateMachine::new(SessionState::LoggedOut);
        assert!(sm.transition_to(SessionState::Working).is_err());
        assert!(sm.transition_to(SessionState::ManualBreak).is_err());

        sm = StateMachine::new(SessionState::ShiftReady);
        assert!(sm.transition_to(SessionState::ManualBreak).is_err());
        assert!(sm.transition_to(SessionState::ShiftSummary).is_err());

        sm = StateMachine::new(SessionState::Working);
        assert!(sm.transition_to(SessionState::LoggedOut).is_err());
        assert!(sm.transition_to(SessionState::ShiftReady).is_err());
    }

    #[test]
    fn test_end_shift_during_break() {
        let mut sm = StateMachine::new(SessionState::ManualBreak);
        assert!(sm.transition_to(SessionState::ShiftCompleting).is_ok());
    }
}
