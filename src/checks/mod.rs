//! Check modules for sw-checklist
//!
//! This module contains all validation checks organized by domain.

pub mod clap;
pub mod install;
pub mod modularity;
pub mod tests;
pub mod wasm;

/// Result of a validation check
#[derive(Debug)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub message: String,
    pub is_warning: bool,
}

impl CheckResult {
    pub fn pass(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: true,
            message: message.into(),
            is_warning: false,
        }
    }

    pub fn fail(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: false,
            message: message.into(),
            is_warning: false,
        }
    }

    pub fn warn(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: true,
            message: message.into(),
            is_warning: true,
        }
    }
}
