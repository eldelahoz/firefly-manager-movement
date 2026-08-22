use chrono::NaiveDateTime;

/// Clock for testable time logic.
pub trait Clock: Send + Sync {
    fn now(&self) -> NaiveDateTime;
}
