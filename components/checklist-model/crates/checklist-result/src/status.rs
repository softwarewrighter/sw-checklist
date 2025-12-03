//! Check status enum

/// Status of a check result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    /// Check passed
    Pass,
    /// Check failed
    Fail,
    /// Check passed with warning
    Warn,
    /// Informational (neither pass nor fail)
    Info,
}

impl CheckStatus {
    /// Returns true if the check passed (Pass, Warn, or Info)
    pub fn passed(self) -> bool {
        matches!(
            self,
            CheckStatus::Pass | CheckStatus::Warn | CheckStatus::Info
        )
    }

    /// Returns true if this is a warning
    pub fn is_warning(self) -> bool {
        matches!(self, CheckStatus::Warn)
    }

    /// Returns true if this is informational
    pub fn is_info(self) -> bool {
        matches!(self, CheckStatus::Info)
    }
}
