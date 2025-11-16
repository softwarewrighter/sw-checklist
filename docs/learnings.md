# Lessons Learned

This document tracks issues encountered during development, their root causes, and strategies to prevent them in the future.

## Table of Contents

1. [Project Structure Assumptions](#project-structure-assumptions)
2. [Overly Strict String Matching](#overly-strict-string-matching)

---

## Project Structure Assumptions

**Date**: 2025-11-16
**Issue**: Tool failed on workspace and multi-crate projects like scan3data
**Severity**: High - Core functionality broken for common project structures

### What Went Wrong

The initial implementation of sw-checklist assumed:
1. Project has a single Cargo.toml at the root
2. The `target/` directory is adjacent to Cargo.toml
3. Each crate has only one binary
4. Binaries are named the same as the package

This caused failures when testing on scan3data, which:
- Has 6 Cargo.toml files (workspace structure)
- Has target/ at the workspace root
- Multiple crates in subdirectories
- Some crates with multiple binaries

### Root Cause

**Insufficient test coverage**. The code was only tested on a single, simple project (itself) before attempting to use it on a complex real-world project.

Specific gaps:
- No tests for workspace projects
- No tests for multi-crate projects
- No tests for projects with multiple binaries per crate
- No tests with different directory structures

### Why Wasn't It Caught Sooner?

1. **Limited dogfooding**: Only tested on sw-checklist itself, which is a simple single-crate project
2. **No negative tests**: Didn't test edge cases or different project structures
3. **No integration tests**: Only had unit tests for individual functions
4. **Assumptions not documented**: Didn't explicitly list assumptions for review

### Prevention Strategy

#### 1. Test-Driven Development

**Before writing code**, create tests for:
- Simple cases (single crate, single binary)
- Complex cases (workspaces, multi-crate)
- Edge cases (multiple binaries, nested structures)

#### 2. Comprehensive Test Suite

Add tests for various project structures:

```rust
#[test]
fn test_single_crate_project() {
    // Simple project: one Cargo.toml, one binary
}

#[test]
fn test_workspace_project() {
    // Workspace: multiple crates, target/ at root
}

#[test]
fn test_multi_binary_crate() {
    // Crate with multiple [[bin]] sections
}

#[test]
fn test_nested_crates() {
    // Crates in subdirectories
}
```

#### 3. Don't Assume File Locations

**Bad**:
```rust
let binary = path.join("target/release").join(&binary_name);
```

**Good**:
```rust
// Search for binaries starting at project root
let project_root = find_project_root(path);
let binary = project_root.join("target/release").join(&binary_name);
```

#### 4. Use File Discovery Instead of Assumptions

**Bad**:
```rust
let cargo_toml = path.join("Cargo.toml");
```

**Good**:
```rust
let cargo_tomls = find_all_cargo_tomls(path);
for cargo_toml in cargo_tomls { ... }
```

#### 5. Test on Real Projects Early

Before considering a feature complete:
1. Test on simple projects (single crate)
2. Test on complex projects (workspaces, multi-crate)
3. Test on edge cases (multiple binaries, nested structures)

#### 6. Document Assumptions

In code and design docs, explicitly list assumptions:
```rust
/// Finds binaries for a crate
///
/// # Assumptions
/// - Binaries are in `target/{debug,release}` at project root
/// - Binary names come from [[bin]] sections or package name
/// - Works with workspace and single-crate projects
```

### Process Changes

#### Pre-Commit Checklist Addition

Before committing code that handles file system operations:
- [ ] Tested with simple structure
- [ ] Tested with complex structure (workspace, multi-crate)
- [ ] Tested with edge cases (multiple binaries, nested dirs)
- [ ] No hardcoded path assumptions
- [ ] Uses discovery/search instead of direct paths
- [ ] Assumptions documented in code comments

#### Testing Strategy

For file system operations:
1. Create temporary test directories with various structures
2. Test all combinations:
   - Single crate vs workspace
   - Single binary vs multiple binaries
   - Flat structure vs nested structure
3. Use property-based testing where applicable

### Code Patterns to Avoid

#### Anti-Pattern: Direct Path Construction
```rust
// BAD: Assumes target/ is next to crate directory
let binary = crate_dir.join("target/release").join(name);
```

#### Better Pattern: Search from Root
```rust
// GOOD: Search from project root
let binary = project_root.join("target/release").join(name);
```

#### Best Pattern: Discovery
```rust
// BEST: Search multiple locations
let binary = find_binary(project_root, name)?;

fn find_binary(root: &Path, name: &str) -> Option<PathBuf> {
    for target_dir in ["target/release", "target/debug"] {
        let path = root.join(target_dir).join(name);
        if path.exists() {
            return Some(path);
        }
    }
    None
}
```

### Related Issues

- None yet (first issue documented)

### References

- [Cargo Workspaces Documentation](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [Cargo Manifest Format](https://doc.rust-lang.org/cargo/reference/manifest.html)
- Commit: `2fe9bf4` - Refactor to support multi-crate projects

---

## Overly Strict String Matching

**Date**: 2025-11-16
**Issue**: Version field checks failed on scan3data despite having required info
**Severity**: Medium - False negatives on valid projects

### What Went Wrong

The version output validation used exact string matching:
- Required `"Copyright (c)"` but scan3data had `"Copyright:"`
- Required `"MIT License"` but scan3data had `"License: MIT"`
- Case-sensitive matching failed on variations like `"Build-Host:"` vs `"Build Host:"`

This caused false failures on perfectly valid version output that used different formatting conventions.

### Root Cause

**Inflexible validation patterns**. The code used exact substring matching without considering:
1. Different formatting conventions (with/without colons)
2. Case variations
3. Word order variations
4. Different ways to express the same information

### Why Wasn't It Caught Sooner?

1. **Only tested on one project**: sw-checklist itself, which happened to use the exact format expected
2. **No tests for format variations**: Didn't test different valid ways to format version output
3. **Assumption of single format**: Assumed everyone would format version output identically

### Prevention Strategy

#### 1. Use Flexible Pattern Matching

Instead of exact strings, use multiple acceptable patterns:

```rust
// BAD: Exact match only
if version_output.contains("MIT License") { ... }

// GOOD: Multiple patterns, case-insensitive
let patterns = ["license", "mit", "apache", "gpl"];
let lower = version_output.to_lowercase();
if patterns.iter().any(|p| lower.contains(p)) { ... }
```

#### 2. Case-Insensitive Matching

Always convert to lowercase for comparisons:

```rust
// BAD: Case-sensitive
if output.contains("Copyright") { ... }

// GOOD: Case-insensitive
if output.to_lowercase().contains("copyright") { ... }
```

#### 3. Test Multiple Valid Formats

Add tests for various valid formatting styles:

```rust
#[test]
fn test_license_variations() {
    // Test "MIT License"
    // Test "License: MIT"
    // Test "License: Apache-2.0"
    // Test case variations
}
```

#### 4. Provide Helpful Error Messages

When validation fails, be clear about what's missing:

```rust
// BAD: Generic message
"Version output should contain License"

// GOOD: Specific, helpful message
"There does not appear to be license info present in the -V/--version output"
```

### Process Changes

#### Pre-Commit Checklist Addition

Before committing validation code:
- [ ] Tested with multiple valid format variations
- [ ] Used case-insensitive matching where appropriate
- [ ] Tested on projects with different conventions
- [ ] Error messages are clear and actionable
- [ ] No assumptions about exact formatting

#### Testing Strategy

For string matching/validation:
1. Test the expected format
2. Test common variations (with/without colons, hyphens, etc.)
3. Test case variations (lowercase, uppercase, mixed)
4. Test negative cases (missing info)
5. Test on multiple real projects with different conventions

### Code Patterns to Avoid

#### Anti-Pattern: Exact String Matching
```rust
// BAD: Requires exact format
if output.contains("Copyright (c)") {
    pass()
} else {
    fail()
}
```

#### Better Pattern: Multiple Patterns
```rust
// GOOD: Accepts variations
let patterns = ["copyright (c)", "copyright:", "copyright"];
if patterns.iter().any(|p| output.to_lowercase().contains(p)) {
    pass()
} else {
    fail()
}
```

#### Best Pattern: Dedicated Validation Function
```rust
// BEST: Reusable, testable, flexible
fn check_version_field(
    field_name: &str,
    output: &str,
    patterns: &[&str],
) -> CheckResult {
    let lower = output.to_lowercase();
    let found = patterns.iter().any(|p| lower.contains(p));

    if found {
        CheckResult::pass(...)
    } else {
        CheckResult::fail(...)
    }
}
```

### Related Issues

- [Project Structure Assumptions](#project-structure-assumptions) - Similar root cause (insufficient testing)

### References

- Commit: `[pending]` - Flexible version field validation
- Tests added:
  - `test_check_version_field_case_insensitive`
  - `test_check_version_field_license_variations`
  - `test_check_version_field_build_variations`

---

## Template for Future Issues

**Date**: YYYY-MM-DD
**Issue**: Brief description
**Severity**: Low / Medium / High

### What Went Wrong
Describe the bug or issue.

### Root Cause
Why did this happen?

### Why Wasn't It Caught Sooner?
What process gap allowed this to slip through?

### Prevention Strategy
Specific, actionable steps to prevent recurrence.

### Process Changes
Updates to development workflow or checklist.

### Code Patterns to Avoid
Examples of anti-patterns with better alternatives.

### Related Issues
Links to similar past issues.

### References
Links to commits, docs, or external resources.
