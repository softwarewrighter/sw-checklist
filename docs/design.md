# Design Document
# sw-checklist

**Version**: 0.2.0
**Date**: 2025-12-03
**Status**: Active

## Design Philosophy

### Core Principles

1. **Simplicity Over Complexity**: Use straightforward solutions; avoid over-engineering
2. **Fail Fast, Fail Clearly**: Provide immediate, actionable feedback
3. **Progressive Enhancement**: Warnings before failures; allow gradual improvement
4. **Zero Configuration**: Sensible defaults; configuration optional
5. **Self-Validation**: Tool must pass its own checks (dogfooding)
6. **Per-Crate Type Detection**: Check types are determined per-crate, not per-project
7. **Multi-Component Awareness**: Support both old-style and new-style project structures

### Design Influences

- **Miller's Law (7±2 Rule)**: Cognitive limits guide all quantitative thresholds
- **UNIX Philosophy**: Do one thing well; compose with other tools
- **Test-Driven Development**: Tests before implementation
- **Rust Idioms**: Leverage type system, use std patterns

## Key Design Decisions

### Decision 1: Single Binary Architecture

**Decision**: Implement as single binary rather than library + binary

**Rationale**:
- Simpler deployment (one file to install)
- Faster startup time
- Easier for users and CI/CD
- Can extract library later if needed

**Trade-offs**:
- Cannot use as library dependency (acceptable for v0.1)
- Some code duplication in tests (mitigated by helper functions)

**Alternatives Considered**:
- Library crate + binary crate: Adds complexity, slower compilation
- Plugin architecture: Over-engineering for current needs

### Decision 2: Line-Based Function Parsing

**Decision**: Use simple line-based parsing with brace counting vs full AST parsing

**Rationale**:
- Much faster than full AST parsing with syn
- Sufficient accuracy for function LOC counting
- Simpler code, easier to maintain
- Already have syn as dependency for future use

**Implementation**:
```rust
// Find function signature
if line.starts_with("fn ") || line.starts_with("pub fn ") {
    // Count braces to find end
    let mut brace_count = 0;
    for line in lines {
        for ch in line.chars() {
            if ch == '{' { brace_count += 1; }
            if ch == '}' { brace_count -= 1; }
            if brace_count == 0 { break; }
        }
    }
}
```

**Trade-offs**:
- May miscount in edge cases (strings with braces, comments)
- Good enough for 99% of real code
- Can enhance with syn later if needed

**Alternatives Considered**:
- Full AST parsing with syn: Accurate but much slower
- Regex parsing: Complex, error-prone with nested braces

### Decision 3: Flexible String Matching for Version Fields

**Decision**: Use multiple acceptable patterns vs exact string matching

**Rationale**:
- Different projects use different formatting conventions
- Avoid false positives on valid metadata
- Better user experience

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
    // ...
}

// Usage
check_version_field(
    &mut results,
    "Copyright",
    &version_output,
    &["copyright", "copyright (c)", "copyright:"]
);
```

**Lessons Learned**: See docs/learnings.md - Overly Strict String Matching

### Decision 4: Warnings vs Failures

**Decision**: Two-tier threshold system (warning, then failure)

**Rationale**:
- Progressive feedback encourages improvement
- Warnings don't break builds initially
- Projects can improve incrementally
- Clear signal when limits are approached

**Thresholds**:
| Metric | Ideal | Warning | Failure |
|--------|-------|---------|---------|
| Function LOC | <25 | 26-50 | >50 |
| Functions/Module | ≤4 | 5-7 | >7 |
| Modules/Crate | ≤4 | 5-7 | >7 |
| Crates/Project | ≤4 | 5-7 | >7 |

**Exit Codes**:
- Warnings: Exit 0 (success)
- Failures: Exit 1 (failure)

### Decision 5: Include Tests in Function Counts

**Decision**: Count test functions in modularity metrics

**Rationale**:
- Test functions are still code to maintain
- Large test functions indicate test needs refactoring
- Encourages modular test design
- Consistent counting rules (simpler)

**Trade-offs**:
- May trigger warnings in projects with many tests
- Acceptable: tests should also be modular

**Future**: Could add `--exclude-tests` flag if needed

### Decision 6: Build-Time Metadata Generation

**Decision**: Generate version metadata at build time vs runtime

**Rationale**:
- Capture exact build context (commit, timestamp, host)
- Faster runtime (no git commands during execution)
- Embedded in binary (no external dependencies)

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
}

// main.rs
const BUILD_COMMIT: &str = env!("BUILD_COMMIT_SHA");
```

### Decision 7: Workspace and Multi-Crate Support

**Decision**: Find all Cargo.toml files recursively vs assume single crate

**Rationale**:
- Real projects often use workspaces
- Multi-crate projects are common
- Initial assumption of single crate caused failures

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

**Lessons Learned**: See docs/learnings.md - Project Structure Assumptions

### Decision 8: Multi-Component Project Support

**Decision**: Support new-style repositories with components/ directory structure

**Rationale**:
- New projects use independent component workspaces
- Each component can have different crate types (CLI, WASM, Library)
- Crate limits should apply per-component, not project-wide
- Allows scaling to larger projects without hitting arbitrary limits

**Detection Algorithm**:
```rust
fn is_multi_component_project(path: &Path) -> bool {
    let components_dir = path.join("components");
    let root_cargo = path.join("Cargo.toml");

    // Multi-component: has components/ but no root Cargo.toml
    components_dir.exists() && !root_cargo.exists()
}
```

**Crate Counting Strategy**:
- Old-style: Count all Cargo.toml files project-wide
- Multi-component: Count Cargo.toml files per component
- Each component gets its own 7-crate limit
- Warn if more than 7 components (but don't fail)

**Trade-offs**:
- More complex discovery logic
- Need to handle both styles correctly
- Benefits large-scale projects significantly

### Decision 9: Per-Crate Type Detection

**Decision**: Determine crate type (CLI, WASM, Library) individually for each crate

**Rationale**:
- A project may contain multiple crate types
- CLI checks should only run on CLI crates
- WASM checks should only run on WASM crates
- Previous approach checked project-wide types, causing false positives

**Detection Criteria**:
```rust
// CLI crate detection
fn is_cli_crate(cargo_toml: &str) -> bool {
    cargo_toml.contains("clap") ||
    cargo_toml.contains("[[bin]]")
}

// WASM crate detection
fn is_wasm_crate(cargo_toml: &str) -> bool {
    cargo_toml.contains("wasm-bindgen") ||
    cargo_toml.contains("yew") ||
    cargo_toml.contains("crate-type = [\"cdylib\"]")
}
```

**Benefits**:
- No more CLI checks on library crates
- No more WASM checks on pure libraries
- Correct behavior in multi-component projects
- Each crate gets exactly the checks it needs

## Data Structures

### CheckResult

**Purpose**: Represent result of single validation check

**Design**:
```rust
struct CheckResult {
    name: String,        // Identifies check (e.g., "Function LOC [crate-name]")
    passed: bool,        // True if check passed
    message: String,     // Detailed message (explains pass/fail/warn)
    is_warning: bool,    // True if warning vs hard failure
}
```

**Factory Methods**:
```rust
impl CheckResult {
    fn pass(name: impl Into<String>, message: impl Into<String>) -> Self;
    fn fail(name: impl Into<String>, message: impl Into<String>) -> Self;
    fn warn(name: impl Into<String>, message: impl Into<String>) -> Self;
}
```

**Rationale**:
- Simple, clear semantics
- Factory methods ensure consistency
- Easy to serialize (future: JSON output)
- Human-readable format

### CLI Structure

**Design**:
```rust
#[derive(Parser)]
#[command(name = "sw-checklist")]
#[command(long_version = LONG_VERSION)]
#[command(about = "...")]
#[command(long_about = "...")]
#[command(after_long_help = AI_AGENT_INSTRUCTIONS)]
struct Cli {
    #[arg(default_value = ".")]
    project_path: PathBuf,

    #[arg(short, long)]
    verbose: bool,
}
```

**Key Features**:
- Clap derive macros for type safety
- Long version with build metadata
- AI agent instructions in extended help
- Verbose flag for debugging

## Algorithm Design

### Modularity Check Algorithm

**Problem**: Count functions per file and LOC per function efficiently

**Approach**: Single-pass line-based parsing

**Algorithm**:
```
function check_modularity(crate_dir, crate_name):
    results = []
    module_counts = HashMap<filename, function_count>
    module_count = 0

    for each file in src/**/*.rs:
        module_count += 1
        function_count = 0
        lines = read_file_lines()

        i = 0
        while i < len(lines):
            if lines[i] matches function signature:
                function_count += 1
                fn_name = extract_name(lines[i])

                # Find opening brace
                brace_line = find_opening_brace(lines, i)

                # Count to closing brace
                end_line = count_braces(lines, brace_line)

                # Calculate LOC
                loc = end_line - i + 1

                # Check thresholds
                if loc > 50:
                    results.add(FAIL: function too long)
                else if loc > 25:
                    results.add(WARN: function getting long)

                i = end_line + 1
            else:
                i += 1

        # Check module function count
        if function_count > 7:
            results.add(FAIL: too many functions in module)
        else if function_count > 4:
            results.add(WARN: many functions in module)

    # Check crate module count
    if module_count > 7:
        results.add(FAIL: too many modules)
    else if module_count > 4:
        results.add(WARN: many modules)

    return results
```

**Complexity**:
- Time: O(n) where n = total lines of code
- Space: O(m) where m = number of modules

**Edge Cases Handled**:
- Functions without braces (trait declarations): Skip gracefully
- Nested braces in function body: Correct counting
- Multi-line function signatures: Look ahead for opening brace
- Empty files: Return pass results

### Check Orchestration Algorithm

**Problem**: Run appropriate checks for each crate type

**Approach**: Per-crate type detection with conditional check dispatch

**Algorithm**:
```
function run_checks(project_root, cargo_tomls, verbose):
    results = []

    for each cargo_toml in cargo_tomls:
        crate_dir = parent_dir(cargo_toml)
        cargo_data = parse_toml(cargo_toml)
        crate_name = cargo_data.package.name
        cargo_content = read_file(cargo_toml)

        # Per-crate type detection
        is_cli = is_cli_crate(cargo_content)      # has clap dependency
        is_wasm = is_wasm_crate(cargo_content)    # has wasm-bindgen/yew

        # Type-specific checks - only if this crate matches
        if is_cli:
            results.extend(check_rust_crate(crate_dir))

        if is_wasm:
            results.extend(check_wasm_crate(crate_dir))

        # Universal checks (all Rust crates)
        results.extend(check_modularity(crate_dir, crate_name))

    return results
```

**Design Rationale**:
- Per-crate detection prevents false positives
- Checks are independent (can run in any order)
- Type detection is inclusive (crate can be both CLI and WASM)
- Modularity checks always run (universal requirement)
- Library crates only get modularity checks

### Multi-Component Discovery Algorithm

**Problem**: Identify project structure and group crates by component

**Approach**: Check for components/ directory pattern

**Algorithm**:
```
function discover_project_structure(project_root):
    components_dir = project_root / "components"
    root_cargo = project_root / "Cargo.toml"

    if components_dir.exists() and not root_cargo.exists():
        # Multi-component project
        structure = MultiComponent
        components = []

        for dir in list_directories(components_dir):
            if (dir / "Cargo.toml").exists():
                component = Component {
                    name: dir.name,
                    root: dir,
                    crates: find_cargo_tomls(dir)
                }
                components.push(component)

        return (structure, components)
    else:
        # Old-style project
        structure = OldStyle
        crates = find_cargo_tomls(project_root)
        return (structure, crates)
```

### Crate Count Validation Algorithm

**Problem**: Apply 7-crate limit appropriately based on project structure

**Algorithm**:
```
function check_crate_counts(project_root, structure):
    results = []

    if structure == MultiComponent:
        # Per-component limits
        for component in components:
            crate_count = component.crates.len() - 1  # exclude workspace Cargo.toml
            if crate_count > 7:
                results.push(FAIL: component has too many crates)
            elif crate_count > 4:
                results.push(WARN: component approaching crate limit)
            else:
                results.push(PASS)

        # Component count warning (no hard limit)
        if components.len() > 7:
            results.push(WARN: many components, consider grouping)

    else:
        # Old-style: project-wide limit
        crate_count = all_crates.len()
        if crate_count > 7:
            results.push(FAIL: project has too many crates)
        elif crate_count > 4:
            results.push(WARN: project approaching crate limit)
        else:
            results.push(PASS)

    return results
```

## Output Design

### Terminal Output Format

**Design Goals**:
- Clear visual hierarchy
- Easy to scan for failures
- Actionable error messages
- Compatible with CI/CD logs

**Format**:
```
Checking project: <path>
Project type: <type>
Found <n> Cargo.toml file(s)

Check Results:
================================================================================
<status> | <check name>
       <message>

<status> | <check name>
       <message>

Summary: <passed> passed, <failed> failed, <warnings> warnings
```

**Status Symbols**:
- `✓ PASS`: Green checkmark (passed check)
- `✗ FAIL`: Red X (failed check)
- `⚠ WARN`: Yellow warning (warning)

**Indentation**:
- Check name: Left-aligned after status
- Message: Indented 7 spaces for alignment

### Error Message Design

**Principles**:
1. **Specificity**: Include file name, function name, line count
2. **Guidance**: Explain what's wrong and how to fix
3. **Context**: Show current value vs threshold
4. **Tone**: Neutral, helpful, not judgmental

**Examples**:

**Good**:
```
✗ FAIL | Function LOC [sw-checklist]
       Function 'check_version_flags' in main.rs has 103 lines (max 50)
```

**Better**:
```
✗ FAIL | Function LOC [sw-checklist]
       Function 'check_version_flags' in main.rs has 103 lines (max 50)
       Consider: Extract version field checks to separate functions
```

**Best** (future):
```
✗ FAIL | Function LOC [sw-checklist]
       Function 'check_version_flags' in main.rs:460-562 has 103 lines (max 50)
       Suggestion: Extract repeated check logic to check_version_field()
       See: docs/learnings.md#modularity-refactoring
```

## User Experience Design

### Common Workflows

#### Workflow 1: First Run on New Project

```bash
$ sw-checklist

# Tool discovers project type, runs checks, reports issues
# User sees clear list of what needs fixing
# User addresses issues one by one
# User re-runs until all checks pass
```

**UX Considerations**:
- Don't require build first (show helpful error if binary missing)
- Provide clear next steps in error messages
- Allow incremental improvement (warnings don't fail)

#### Workflow 2: CI/CD Integration

```yaml
# .github/workflows/check.yml
- name: Run sw-checklist
  run: |
    cargo build --release
    ./target/release/sw-checklist .
```

**UX Considerations**:
- Exit code 1 on failure (fails build)
- Clear, parseable output
- Fast execution (<5s for most projects)

#### Workflow 3: AI Agent Usage

```
AI Agent: Let me run sw-checklist to verify the project

[Runs: sw-checklist /path/to/project]

AI Agent: I see 3 failed checks:
1. Function 'process_data' is 87 lines (max 50)
2. Module 'utils.rs' has 12 functions (max 7)
3. Missing AI agent instructions in --help

Let me address these issues...
```

**UX Considerations**:
- Structured output (easy for AI to parse)
- Clear, actionable messages (AI knows what to fix)
- Comprehensive --help with AI instructions

### Help Text Design

**Structure**:
1. **Short Help (-h)**:
   - Brief description
   - Usage syntax
   - Common options
   - Fast to read

2. **Long Help (--help)**:
   - Full description
   - Detailed option descriptions
   - Current checks summary
   - AI CODING AGENT INSTRUCTIONS section

**AI Instructions Section**:
- Clear usage pattern
- Example workflow
- List of all checks
- Modularity philosophy explanation
- Link to documentation

## Error Handling Design

### Error Propagation

**Strategy**: Use `anyhow::Result` throughout

**Pattern**:
```rust
fn check_something(path: &Path) -> Result<Vec<CheckResult>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {:?}", path))?;

    // Process content
    Ok(results)
}
```

**Benefits**:
- Automatic error context propagation
- Clean error messages with call chain
- Easy to add context at each level

### Graceful Degradation

**Principle**: Skip unparseable content, continue with other checks

**Example**:
```rust
// Parse file
let syntax_tree = match syn::parse_file(&content) {
    Ok(tree) => tree,
    Err(_) => continue,  // Skip this file, continue with others
};
```

**Rationale**:
- One bad file shouldn't break all checks
- Report what we can validate
- Better UX than total failure

## Testing Design

### TDD Approach

**Process** (followed for all modularity checks):
1. **RED**: Write failing test
2. **GREEN**: Implement minimum code to pass
3. **REFACTOR**: Clean up while keeping tests green
4. **REPEAT**: Next feature

**Example**:
```rust
// RED: Test fails (check_modularity doesn't exist)
#[test]
fn test_function_loc_under_25_pass() {
    let results = check_modularity(&crate_dir, "test-crate").unwrap();
    assert!(results.iter().all(|r| r.passed));
}

// GREEN: Implement check_modularity (simplest version)
fn check_modularity(...) -> Result<Vec<CheckResult>> {
    // Basic implementation
}

// REFACTOR: Clean up, add features
```

### Test Fixture Pattern

**Pattern**: Use tempfile for isolated test environments

```rust
#[test]
fn test_something() {
    use tempfile::tempdir;

    // Create isolated directory
    let temp = tempdir().unwrap();
    let crate_dir = temp.path().join("test-crate");
    fs::create_dir_all(&crate_dir).unwrap();

    // Create test files
    fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"test-crate\"\n"
    ).unwrap();

    // Run test
    let results = check_something(&crate_dir).unwrap();

    // Verify
    assert!(results.iter().any(|r| r.passed));

    // temp automatically cleaned up when dropped
}
```

**Benefits**:
- No test pollution
- Parallel test execution safe
- Automatic cleanup
- Easy to reason about

### Test Coverage Strategy

**Goals**:
- All check functions have tests
- Happy path and error cases covered
- Edge cases identified and tested

**Test Types**:
1. **Unit Tests**: Individual functions in isolation
2. **Integration Tests**: Full check flows end-to-end
3. **Edge Case Tests**: Boundaries, empty inputs, malformed data

## Performance Design

### Current Performance Profile

**Measurements**:
- Small project (1 crate, <1000 LOC): ~100ms
- Medium project (5 crates, ~5000 LOC): ~500ms
- Large project (20 crates, ~20000 LOC): ~2s

**Bottlenecks**:
1. File I/O (reading source files)
2. Line-by-line parsing
3. Binary execution for clap checks

### Optimization Strategies (Future)

**Parallelization**:
```rust
// Current: Sequential
for cargo_toml in cargo_tomls {
    results.extend(check_crate(cargo_toml)?);
}

// Future: Parallel
use rayon::prelude::*;
let results: Vec<_> = cargo_tomls
    .par_iter()
    .flat_map(|cargo_toml| check_crate(cargo_toml).ok())
    .flatten()
    .collect();
```

**Caching**:
- Cache parsed Cargo.toml files
- Cache file modification times (skip unchanged files)
- Persist cache between runs

**Incremental Checks**:
- Only check modified files (git diff)
- Skip crates with no changes
- Track check results per-file

## Security Design

### Threat Model

**Assumptions**:
- User controls project directory
- User trusts dependencies from crates.io
- No network access needed
- No privileged operations

**Threats Mitigated**:
- Code injection: No eval, no shell with user input
- Path traversal: Canonicalize paths
- Symlink attacks: Use safe file operations

**Non-Threats** (out of scope):
- Malicious project contents (assumed trusted)
- Supply chain attacks on dependencies
- Physical access to build machine

### Safe Rust Usage

**Guidelines**:
- No `unsafe` blocks
- No raw pointer manipulation
- No FFI calls
- Use safe standard library abstractions

## Future Design Considerations

### Plugin Architecture (v1.0)

**Design**:
```rust
trait Check {
    fn name(&self) -> &str;
    fn run(&self, project: &Project) -> Result<Vec<CheckResult>>;
}

// User-provided check
#[derive(Check)]
struct CustomCheck { /* ... */ }

// Load plugins
let plugins = load_plugins_from("~/.sw-checklist/plugins/");
for plugin in plugins {
    results.extend(plugin.run(&project)?);
}
```

### Configuration System (v0.2)

**Design**:
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

### JSON Output (v1.0)

**Design**:
```rust
#[derive(Serialize)]
struct JsonOutput {
    project: String,
    project_type: String,
    checks: Vec<CheckResult>,
    summary: Summary,
}

// Usage
sw-checklist --format json > results.json
```

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1.0 | 2025-11-17 | Claude Code | Initial design document |
| 0.2.0 | 2025-12-03 | Claude Code | Added multi-component support, per-crate type detection |
