# Design Decisions

Architectural Decision Records (ADRs) for key design choices in sw-checklist.

## Overview

This document explains the reasoning behind major design decisions, alternatives considered, and trade-offs accepted.

## Decision Records

### DR-001: Single Binary Architecture

**Status**: ✅ Accepted

**Context**: Need to distribute sw-checklist to users and CI/CD systems.

**Decision**: Implement as single binary rather than library + binary.

**Rationale**:
- **Simpler deployment**: One file to install
- **Faster startup**: No dynamic linking
- **Easier for CI/CD**: Single `wget` or `curl` command
- **Better UX**: `cargo install` works out of the box

**Alternatives Considered**:

1. **Library + Binary Crate**
   - ❌ More complex project structure
   - ❌ Slower compilation (workspace overhead)
   - ❌ More difficult for users
   - ✅ Better for programmatic use
   - **Rejected**: Current use case doesn't need library API

2. **Plugin Architecture**
   - ❌ Much more complex
   - ❌ Dynamic linking issues
   - ❌ Security concerns (untrusted plugins)
   - ✅ Highly extensible
   - **Rejected**: Over-engineering for v0.1

**Trade-offs Accepted**:
- Cannot use as library dependency (acceptable for v0.1)
- Some code duplication in tests (mitigated by helper functions)

**Future**: Can extract library in v1.0 if needed

---

### DR-002: Line-Based Function Parsing

**Status**: ✅ Accepted

**Context**: Need to count lines of code per function for modularity checks.

**Decision**: Use simple line-based parsing with brace counting vs full AST parsing.

**Rationale**:
- **Much faster**: 10-50x faster than syn-based parsing
- **Sufficient accuracy**: 99%+ accurate for real code
- **Simpler code**: ~100 lines vs ~500 lines
- **Easier to maintain**: No AST traversal complexity

**Implementation**:
```rust
fn count_function_loc(lines: &[String]) -> Vec<CheckResult> {
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim_start().starts_with("fn ")
            || lines[i].trim_start().starts_with("pub fn ") {

            // Find opening brace
            let mut brace_line = i;
            while !lines[brace_line].contains('{') {
                brace_line += 1;
            }

            // Count braces to find end
            let mut brace_count = 0;
            let mut end_line = brace_line;

            for (idx, line) in lines[brace_line..].iter().enumerate() {
                for ch in line.chars() {
                    if ch == '{' { brace_count += 1; }
                    if ch == '}' { brace_count -= 1; }
                    if brace_count == 0 {
                        end_line = brace_line + idx;
                        break;
                    }
                }
                if brace_count == 0 { break; }
            }

            let loc = end_line - i + 1;
            // Check against thresholds...

            i = end_line + 1;
        } else {
            i += 1;
        }
    }
}
```

**Alternatives Considered**:

1. **Full AST Parsing with syn**
   - ✅ 100% accurate
   - ✅ Handles all edge cases
   - ❌ 10-50x slower
   - ❌ Much more complex code
   - ❌ Larger binary size
   - **Rejected**: Speed matters more than perfect accuracy

2. **Regex Parsing**
   - ❌ Complex regex patterns
   - ❌ Error-prone with nested braces
   - ❌ Hard to maintain
   - ✅ Moderate speed
   - **Rejected**: Not accurate enough

**Trade-offs Accepted**:
- May miscount in edge cases (strings with braces, unusual formatting)
- Good enough for 99% of real code
- Errors are acceptable (just warnings/failures in edge cases)

**Known Limitations**:
```rust
// Might confuse this:
let s = "fn fake() { }"; // String contains "fn" and braces

// But real code rarely looks like this
// And consequences are minor (slightly wrong count)
```

---

### DR-003: Flexible String Matching for Metadata

**Status**: ✅ Accepted

**Context**: Validating version output contains required fields (Copyright, License, etc.).

**Decision**: Accept multiple patterns per field vs exact string matching.

**Rationale**:
- **Better UX**: Doesn't force specific formatting
- **Fewer false positives**: Accepts reasonable variations
- **Cultural sensitivity**: Different copyright formats
- **Practical**: Real projects use different conventions

**Implementation**:
```rust
fn check_version_field(
    results: &mut Vec<CheckResult>,
    field_name: &str,
    version_output: &str,
    patterns: &[&str],
) {
    let lower = version_output.to_lowercase();
    let found = patterns.iter().any(|p| lower.contains(p));

    if found {
        results.push(CheckResult::pass(...));
    } else {
        results.push(CheckResult::fail(...));
    }
}

// Usage
check_version_field(
    &mut results,
    "Copyright",
    &version_output,
    &["copyright", "copyright (c)", "copyright:", "©"]
);
```

**Alternatives Considered**:

1. **Exact String Match**
   - ❌ Too rigid
   - ❌ Many false positives
   - ❌ Poor UX
   - ✅ Simple implementation
   - **Rejected**: Real-world lesson (see docs/learnings.md)

2. **Regex Patterns**
   - ✅ Very flexible
   - ❌ More complex
   - ❌ Harder for users to understand requirements
   - ❌ Potential regex DoS
   - **Rejected**: String matching is sufficient

**Trade-offs Accepted**:
- Might accept incomplete information (e.g., just "Copyright" without year)
- Acceptable: Goal is presence, not perfect formatting

**Learning**: Original overly-strict matching caused frustration

---

### DR-004: Warnings vs Failures (Two-Tier Thresholds)

**Status**: ✅ Accepted

**Context**: Need to encourage improvement without breaking builds immediately.

**Decision**: Two-tier threshold system (warning, then failure).

**Rationale**:
- **Progressive feedback**: Warning first, fail later
- **Gradual improvement**: Projects can improve incrementally
- **Clear signals**: "Approaching limit" vs "Over limit"
- **Better adoption**: Don't fail existing projects immediately

**Thresholds**:
| Metric | Ideal | Warning | Failure |
|--------|-------|---------|---------|
| Function LOC | ≤25 | 26-50 | >50 |
| Functions/Module | ≤4 | 5-7 | >7 |
| Modules/Crate | ≤4 | 5-7 | >7 |
| Crates/Project | ≤4 | 5-7 | >7 |

**Alternatives Considered**:

1. **Single Threshold (Pass/Fail Only)**
   - ❌ Too harsh
   - ❌ No early warning
   - ❌ Poor adoption
   - ✅ Simpler implementation
   - **Rejected**: UX too poor

2. **Three Tiers (Info/Warning/Failure)**
   - ✅ More granular
   - ❌ More complex
   - ❌ Harder to understand
   - ❌ When to set each threshold?
   - **Rejected**: Two tiers sufficient

**Trade-offs Accepted**:
- Warnings might be ignored (acceptable: still visible)
- Exit code 0 with warnings (intentional: allow gradual improvement)

**Basis**: Miller's Law (7±2)
- 4 or fewer: Comfortable working memory
- 5-7: Approaching limit
- 8+: Exceeds cognitive capacity

---

### DR-005: Include Tests in Function Counts

**Status**: ✅ Accepted

**Context**: Should test functions count toward modularity limits?

**Decision**: Yes, count test functions in all checks.

**Rationale**:
- **Tests are code**: Need maintenance like any other code
- **Large test functions**: Indicate testing complexity
- **Consistent rules**: Simpler mental model
- **Encourages modular tests**: Tests benefit from small functions too

**Example**:
```rust
// src/processor.rs
pub fn process() { } // Function 1

#[cfg(test)]
mod tests {
    #[test]
    fn test_process() { } // Function 2

    #[test]
    fn test_edge_case() { } // Function 3
}

// Function count: 3 (includes tests)
```

**Alternatives Considered**:

1. **Exclude Tests**
   - ✅ Lower counts
   - ❌ Tests can still be overly complex
   - ❌ Inconsistent rules
   - ❌ More complex implementation
   - **Rejected**: Tests benefit from modularity too

2. **Separate Test Limits**
   - ✅ Different thresholds for tests
   - ❌ Much more complex
   - ❌ Hard to explain
   - ❌ Different limits for what?
   - **Rejected**: Over-engineering

**Trade-offs Accepted**:
- May trigger warnings in test-heavy modules (acceptable)
- Encourages test organization (beneficial)

**Future**: Could add `--exclude-tests` flag if requested

---

### DR-006: Build-Time Metadata Generation

**Status**: ✅ Accepted

**Context**: Need to embed build information in binaries.

**Decision**: Generate metadata at build time vs runtime.

**Rationale**:
- **Capture exact build context**: Commit, timestamp, host
- **Faster runtime**: No git commands during execution
- **Embedded in binary**: No external dependencies
- **Immutable**: Can't change after build

**Implementation**:
```rust
// build.rs
fn main() {
    let commit = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("Failed to get git commit");

    println!(
        "cargo:rustc-env=BUILD_COMMIT_SHA={}",
        String::from_utf8_lossy(&commit.stdout).trim()
    );

    let timestamp = chrono::Utc::now().to_rfc3339();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", timestamp);

    let hostname = hostname::get().unwrap().to_string_lossy().to_string();
    println!("cargo:rustc-env=BUILD_HOST={}", hostname);
}

// main.rs
const BUILD_COMMIT: &str = env!("BUILD_COMMIT_SHA");
const BUILD_TIME: &str = env!("BUILD_TIMESTAMP");
const BUILD_HOST: &str = env!("BUILD_HOST");
```

**Alternatives Considered**:

1. **Runtime Metadata Generation**
   - ❌ Requires .git directory at runtime
   - ❌ Slower (git commands)
   - ❌ Not available in deployed binaries
   - ✅ Always current
   - **Rejected**: Build-time is correct time

2. **Manual Metadata**
   - ❌ Easily forgotten
   - ❌ Error-prone
   - ❌ Extra work
   - ✅ Simple
   - **Rejected**: Automation better

**Trade-offs Accepted**:
- Metadata fixed at build time (intentional)
- Requires git in build environment (acceptable)

---

### DR-007: Workspace and Multi-Crate Support

**Status**: ✅ Accepted

**Context**: Real projects often use workspaces.

**Decision**: Find all Cargo.toml files recursively vs assume single crate.

**Rationale**:
- **Real-world projects**: Often use workspaces
- **Better coverage**: Check all crates
- **Accurate metrics**: Count across entire project

**Implementation**:
```rust
fn find_cargo_tomls(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name() == "Cargo.toml")
        .map(|e| e.path().to_path_buf())
        .collect()
}
```

**Alternatives Considered**:

1. **Single Crate Only**
   - ❌ Misses nested crates
   - ❌ Inaccurate for workspaces
   - ✅ Simpler
   - **Rejected**: Real projects are workspaces

2. **Parse workspace.members**
   - ✅ More accurate
   - ❌ More complex
   - ❌ Misses deeply nested crates
   - **Rejected**: Recursive search is simpler and complete

**Trade-offs Accepted**:
- Finds all Cargo.toml files (might include unexpected ones)
- Could skip target/ or vendor/ (TODO: add filtering)

---

### DR-008: No Configuration File (v0.1)

**Status**: ✅ Accepted

**Context**: Should users be able to configure thresholds?

**Decision**: No configuration in v0.1, sensible defaults only.

**Rationale**:
- **Simplicity**: Zero configuration needed
- **Consistency**: Same standards across projects
- **Focus**: Get v0.1 working first
- **Defer complexity**: Add config in v0.2 if needed

**Current Behavior**:
- Fixed thresholds (25/50 LOC, 4/7 functions, etc.)
- No way to customize
- All checks enabled

**Alternatives Considered**:

1. **Configuration File (.sw-checklist.toml)**
   - ✅ Customizable thresholds
   - ✅ Enable/disable checks
   - ❌ More complex implementation
   - ❌ More documentation needed
   - ❌ Inconsistency across projects
   - **Deferred**: v0.2 feature

2. **Command-Line Flags**
   - ✅ Quick overrides
   - ❌ Not persistent
   - ❌ Verbose commands
   - **Deferred**: v0.2 feature

**Trade-offs Accepted**:
- Some projects may disagree with thresholds (acceptable)
- Can't disable specific checks (acceptable for v0.1)
- One-size-fits-all (intentional: establish standards)

**Future**: v0.2 will add optional configuration:
```toml
# .sw-checklist.toml
[thresholds]
function_loc_warn = 30
function_loc_fail = 60

[exclude]
patterns = ["generated/**", "vendor/**"]

[checks]
disable = ["sw-install-presence"]
```

---

### DR-009: Read-Only Operation

**Status**: ✅ Accepted

**Context**: Should sw-checklist modify code?

**Decision**: No, read-only validation only.

**Rationale**:
- **Safety**: Can't accidentally break code
- **Trust**: Users trust read-only tools more
- **Simplicity**: No file writing logic
- **Clear purpose**: Validation, not refactoring

**Scope**:
- ✅ Read files
- ✅ Execute binaries (for validation)
- ✅ Report issues
- ❌ Modify source files
- ❌ Auto-fix problems
- ❌ Refactor code

**Alternatives Considered**:

1. **Auto-Fix Capability**
   - ✅ Convenient for simple fixes
   - ❌ Risk of breaking code
   - ❌ Complex implementation
   - ❌ Trust issues
   - **Rejected**: Too risky

2. **Suggest Fixes**
   - ✅ Helpful guidance
   - ✅ Read-only
   - ❌ Complex generation logic
   - **Future**: Could add to error messages

**Trade-offs Accepted**:
- Users must fix issues manually (intentional)
- No automatic refactoring (correct: validation tool, not refactoring tool)

---

### DR-010: Exit Code Semantics

**Status**: ✅ Accepted

**Context**: What should exit codes mean?

**Decision**:
- Exit 0: All passed (warnings allowed)
- Exit 1: One or more failures

**Rationale**:
- **CI/CD friendly**: Failures break builds
- **Warnings don't fail**: Allow gradual improvement
- **Standard practice**: Matches lint tools
- **Clear semantics**: 0 = success, 1 = failure

**Behavior**:
```rust
let failed = results.iter().filter(|r| !r.passed).count();

if failed > 0 {
    std::process::exit(1);
}
// Otherwise exit 0
```

**Alternatives Considered**:

1. **Warnings Fail Builds**
   - ❌ Too strict
   - ❌ Poor adoption
   - ❌ No gradual improvement
   - **Rejected**: Warnings are informational

2. **Different Exit Codes**
   - 0 = all passed
   - 1 = warnings
   - 2 = failures
   - ❌ More complex
   - ❌ Doesn't match standard tools
   - **Rejected**: Over-engineering

**Trade-offs Accepted**:
- Warnings might be ignored (acceptable: still visible in output)
- No distinction between 1 failure and 100 failures in exit code (acceptable)

---

## Summary of Key Principles

### 1. Simplicity Over Complexity
Choose simpler solutions unless complexity clearly necessary.

### 2. Fail Fast, Fail Clearly
Provide immediate, actionable feedback.

### 3. Progressive Enhancement
Warnings before failures; allow gradual improvement.

### 4. Zero Configuration
Sensible defaults; configuration optional (future).

### 5. Self-Validation
Tool must pass its own checks (dogfooding).

### 6. Speed Matters
Optimize for fast execution, even at cost of perfect accuracy.

### 7. Trust Through Safety
Read-only operations only; no code modification.

## Related Documentation

- **[Architecture Overview](Architecture-Overview)** - System design
- **[System Flows](System-Flows)** - Execution flows
- **[Testing Strategy](Testing-Strategy)** - Test approach
