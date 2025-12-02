# Architecture Document
# sw-checklist

**Version**: 0.1.0
**Date**: 2025-11-17
**Status**: Active

## Overview

sw-checklist is a single-binary CLI tool written in Rust that validates project conformance to Software Wrighter LLC standards. It uses a modular, check-based architecture that is easily extensible for new validation types.

## System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        User/AI Agent                        │
└───────────────────────────┬─────────────────────────────────┘
                            │
                            │ CLI invocation
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                      sw-checklist binary                    │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              main() - Entry Point                   │   │
│  │  - Parse CLI args (clap)                            │   │
│  │  - Find Cargo.toml files                            │   │
│  │  - Detect project type                              │   │
│  │  - Run checks                                       │   │
│  │  - Print results                                    │   │
│  └──────────────────┬──────────────────────────────────┘   │
│                     │                                       │
│                     ▼                                       │
│  ┌─────────────────────────────────────────────────────┐   │
│  │          Check Orchestration Layer                  │   │
│  │                                                      │   │
│  │  run_checks() - Iterate crates and dispatch         │   │
│  └──────┬──────────────────────────────────┬───────────┘   │
│         │                                   │               │
│         ▼                                   ▼               │
│  ┌──────────────┐                   ┌──────────────┐       │
│  │ Clap Checks  │                   │  Modularity  │       │
│  │              │                   │    Checks    │       │
│  │ - Help flags │                   │              │       │
│  │ - Version    │                   │ - Fn LOC     │       │
│  │ - Metadata   │                   │ - Module fns │       │
│  │ - Binaries   │                   │ - Crate mods │       │
│  └──────────────┘                   └──────────────┘       │
│                                                             │
│         ▼                                   ▼               │
│  ┌──────────────┐                   ┌──────────────┐       │
│  │ WASM Checks  │                   │ Misc Checks  │       │
│  │              │                   │              │       │
│  │ - HTML files │                   │ - sw-install │       │
│  │ - Favicon    │                   │ - Freshness  │       │
│  │ - Footer     │                   │              │       │
│  └──────────────┘                   └──────────────┘       │
│                                                             │
│         │                                   │               │
│         └───────────────┬───────────────────┘               │
│                         ▼                                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │             Result Aggregation                      │   │
│  │  - Collect CheckResults                             │   │
│  │  - Count passes/failures/warnings                   │   │
│  │  - Format output                                    │   │
│  │  - Exit with appropriate code                       │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## Component Design

### Core Components

#### 1. CLI Interface (`Cli` struct)

**Responsibilities**:
- Parse command-line arguments
- Provide help and version information
- Define AI agent instructions

**Key Features**:
- Uses clap derive macros
- Supports verbose mode
- Default to current directory
- Long help includes AI coding agent instructions

**Code Location**: `src/main.rs:18-50`

#### 2. Project Discovery

**Responsibilities**:
- Find all Cargo.toml files in project tree
- Detect project type (CLI, WASM, Library)
- Handle workspace and multi-crate projects

**Key Functions**:
- `find_cargo_tomls(path)`: Recursively find all Cargo.toml files
- Uses walkdir for efficient traversal
- Returns paths to all discovered crate manifests

**Code Location**: `src/main.rs:197-206`

#### 3. Check Orchestration

**Responsibilities**:
- Iterate over all discovered crates
- Dispatch appropriate checks based on project type
- Aggregate results from all checks

**Key Functions**:
- `run_checks(project_root, cargo_tomls, verbose)`: Main orchestration
  - Parses each Cargo.toml
  - Determines crate type (clap/wasm/library)
  - Runs type-specific checks
  - Runs modularity checks on all crates
  - Returns aggregated results

**Code Location**: `src/main.rs:208-257`

#### 4. Check Result Model

**Data Structure**:
```rust
struct CheckResult {
    name: String,        // Check identifier
    passed: bool,        // Pass/fail status
    message: String,     // Detailed message
    is_warning: bool,    // True if warning vs hard failure
}
```

**Factory Methods**:
- `CheckResult::pass(name, message)`: Create passing result
- `CheckResult::fail(name, message)`: Create failing result
- `CheckResult::warn(name, message)`: Create warning result

**Code Location**: `src/main.rs:100-106`

### Check Implementations

#### Clap CLI Checks

**Module**: `check_rust_crate()` and related functions

**Checks Performed**:
1. **Dependency Check**: Verifies clap in Cargo.toml
2. **Binary Discovery**: Finds built binaries in target/
3. **Help Validation**: Compares -h vs --help output
4. **Version Validation**: Checks -V vs --version consistency
5. **Metadata Validation**: Verifies required fields in version output
6. **Binary Freshness**: Compares local vs installed binary timestamps

**Key Functions**:
- `check_rust_crate()`: Entry point for clap crates
- `check_crate_binaries()`: Find and validate all binaries
- `check_help_flags()`: Validate help output
- `check_version_flags()`: Validate version output
- `check_version_field()`: Flexible field matching

**Design Patterns**:
- Multiple acceptable patterns for version fields (flexible matching)
- Case-insensitive comparisons
- Handles multiple binaries per crate
- Supports both debug and release builds

**Code Location**: `src/main.rs:259-562`

#### WASM Checks

**Module**: `check_wasm_crate()` and related functions

**Checks Performed**:
1. **HTML Files**: Verify index.html exists and references favicon
2. **Favicon**: Verify favicon.ico exists
3. **Footer Metadata**: Scan source for footer with required fields

**Key Functions**:
- `check_wasm_crate()`: Entry point for WASM crates
- `check_wasm_html_files()`: Validate HTML and favicon references
- `check_wasm_favicon()`: Check favicon file
- `check_wasm_footer_in_source()`: Search Rust source for footer
- `check_footer_field()`: Validate individual footer fields

**Code Location**: `src/main.rs:568-784`

#### Modularity Checks

**Module**: `check_modularity()`

**Checks Performed**:
1. **Function LOC**: Count lines per function (warn >25, fail >50)
2. **Module Function Count**: Count functions per file (warn >4, fail >7)
3. **Crate Module Count**: Count .rs files per crate (warn >4, fail >7)
4. **Project Crate Count**: Count crates in project (warn >4, fail >7)

**Algorithm**:
```
For each .rs file in src/:
    Parse file line by line
    For each function definition:
        Find opening brace
        Count braces to find closing brace
        Calculate LOC = end_line - start_line + 1
        Check against thresholds
    Count total functions in file
    Check function count against thresholds
Count total modules (files) in crate
Check module count against thresholds
```

**Key Functions**:
- `check_modularity()`: Main entry point
- `extract_function_name()`: Parse function name from signature
- Uses simple brace counting (not full AST parsing)

**Design Decisions**:
- Simple line-based parsing (fast, good enough)
- Could use syn for full AST parsing in future
- Counts all functions including tests (intentional)
- Warnings before failures (progressive feedback)

**Code Location**: `src/main.rs:1013-1184`

#### Test Validation

**Module**: `check_tests()`

**Checks Performed**:
- Presence of tests/ directory
- Presence of #[test] or #[cfg(test)] annotations
- For WASM: Also check for Jest tests in package.json

**Code Location**: `src/main.rs:786-853`

#### Miscellaneous Checks

**sw-install Presence**:
- Check for sw-install in ~/.local/softwarewrighter/bin/
- Warning only, never fails
- `check_sw_install_presence()`

**Binary Freshness**:
- Compare local build vs installed binary timestamps
- Warning only, never fails
- `check_binary_freshness()`

**Code Location**: `src/main.rs:905-998`

## Data Flow

### Typical Execution Flow

```
1. Parse CLI arguments
   └─> Extract project_path and verbose flag

2. Find all Cargo.toml files
   └─> Recursively walk directory tree
   └─> Collect all manifest paths

3. Detect project type
   └─> Scan all Cargo.toml files for dependencies
   └─> Determine: CLI, WASM, Yew, or Library

4. For each crate:
   a. Parse Cargo.toml
   b. Extract crate name
   c. If has clap: run clap checks
   d. If has wasm: run wasm checks
   e. Always: run modularity checks

5. Add project-level checks
   └─> sw-install presence
   └─> Project crate count

6. Aggregate results
   └─> Count passes, failures, warnings
   └─> Format output
   └─> Exit with code (0 = pass, 1 = failures)
```

### Check Result Aggregation

```rust
// Results from all checks
Vec<CheckResult> results;

// Categorize
let passed = results.filter(passed && !is_warning).count();
let failed = results.filter(!passed).count();
let warnings = results.filter(is_warning).count();

// Exit code
exit_code = if failed > 0 { 1 } else { 0 };
```

## Build System

### Build-Time Metadata Generation

**File**: `build.rs`

**Generated Constants**:
- `BUILD_COMMIT_SHA`: Git commit hash
- `BUILD_TIMESTAMP`: ISO 8601 timestamp
- `BUILD_HOST`: Hostname of build machine

**Mechanism**:
- build.rs runs before compilation
- Executes git and system commands
- Writes to stdout as cargo:rustc-env directives
- Constants available via env!() macro in main.rs

**Code Location**: `build.rs`

### Dependencies

**Runtime Dependencies**:
```toml
clap = { version = "4.5", features = ["derive", "cargo", "wrap_help"] }
anyhow = "1.0"              # Error handling
toml = "0.8"                # Cargo.toml parsing
walkdir = "2.4"             # Directory traversal
const_format = "0.2"        # Compile-time string formatting
syn = "2.0"                 # Future: AST parsing
```

**Build Dependencies**:
```toml
chrono = "0.4"              # Timestamp generation
hostname = "0.4"            # Build host detection
```

**Dev Dependencies**:
```toml
tempfile = "3.10"           # Test fixtures
```

## Testing Strategy

### Test Organization

All tests are in `src/main.rs` within `#[cfg(test)] mod tests`.

### Test Categories

1. **Unit Tests**: Individual functions
   - `test_check_result_creation()`
   - `test_get_binary_names()`
   - `extract_function_name()` logic

2. **Integration Tests**: Full check flows
   - `test_workspace_structure()`
   - `test_multi_binary_crate()`
   - `test_crate_without_clap()`

3. **Modularity Tests**: TDD-created tests
   - `test_function_loc_*()` - 3 tests
   - `test_module_function_count_*()` - 3 tests
   - `test_crate_module_count_*()` - 2 tests

### Test Fixtures

All tests use `tempfile::tempdir()` to create temporary test projects:

```rust
let temp = tempdir().unwrap();
let crate_dir = temp.path().join("test-crate");

// Create Cargo.toml
fs::write(crate_dir.join("Cargo.toml"), "...");

// Create source files
fs::write(crate_dir.join("src/lib.rs"), "...");

// Run checks
let results = check_modularity(&crate_dir, "test-crate").unwrap();

// Assert expectations
assert!(results.iter().any(|r| r.passed));
```

### Test Coverage

Current: 26 tests, all passing

**Coverage by Feature**:
- Project discovery: 4 tests
- Binary name extraction: 3 tests
- Check result creation: 3 tests
- Version field validation: 3 tests
- Binary freshness: 3 tests
- Function LOC: 3 tests
- Module function count: 3 tests
- Crate module count: 2 tests
- Workspace/multi-crate: 2 tests

## Error Handling

### Strategy

- All fallible operations return `Result<T, anyhow::Error>`
- Errors include context via `.with_context(|| ...)`
- Graceful degradation: skip unparseable files, continue checks
- Clear error messages with file paths and line numbers

### Exit Codes

- `0`: All checks passed (warnings allowed)
- `1`: One or more checks failed

## Performance Considerations

### Current Performance

- Small projects (<5 crates): <1 second
- Medium projects (5-20 crates): 1-3 seconds
- Large projects (>20 crates): 3-5 seconds

### Optimization Opportunities

1. **Parallel Crate Processing**: Check crates in parallel
2. **Cached Parsing**: Cache parsed Cargo.toml files
3. **Incremental Checks**: Only check changed files
4. **AST Caching**: Cache syn parsed ASTs for large files

## Extensibility

### Adding New Checks

1. **Define Check Logic**: Create `check_xyz()` function
2. **Return CheckResults**: Use pass/fail/warn factory methods
3. **Integrate**: Call from `run_checks()` or appropriate dispatcher
4. **Test**: Write TDD tests before implementation
5. **Document**: Update README, --help, and PRD

### Check Pattern

```rust
fn check_xyz(crate_dir: &Path, crate_name: &str) -> Result<Vec<CheckResult>> {
    let mut results = Vec::new();

    // Perform validation
    if condition_failed {
        results.push(CheckResult::fail(
            format!("Check Name [{}]", crate_name),
            "Detailed failure message with guidance"
        ));
    } else if condition_warning {
        results.push(CheckResult::warn(
            format!("Check Name [{}]", crate_name),
            "Warning message with suggestion"
        ));
    } else {
        results.push(CheckResult::pass(
            format!("Check Name [{}]", crate_name),
            "Success message"
        ));
    }

    Ok(results)
}
```

## Security Considerations

### Threat Model

**Trusted Input**: Project directories are assumed to be under user control.

**Mitigations**:
- No unsafe code
- No arbitrary code execution
- Read-only file access (except test fixtures)
- No network access
- No shell command execution with user input

### Dependencies

All dependencies are from crates.io and regularly updated.

## Deployment

### Build Process

```bash
cargo build --release
```

### Installation

**Via sw-install**:
```bash
cd /path/to/sw-checklist
cargo build --release
sw-install -p .
```

Installs to: `~/.local/softwarewrighter/bin/sw-checklist`

**Manual**:
```bash
cp target/release/sw-checklist ~/.local/softwarewrighter/bin/
```

## Future Architecture Changes

### Version 0.2.0

- **File LOC Check**: Add to modularity checks
- **Configuration System**: Read .sw-checklist.toml for custom thresholds

### Version 0.3.0

- **Plugin System**: Load custom checks from shared libraries
- **Parallel Processing**: Check crates concurrently
- **Caching Layer**: Cache parsed files between runs

### Version 1.0.0

- **Language Server Protocol**: Real-time feedback in IDEs
- **Watch Mode**: Continuous validation during development
- **JSON Output**: Machine-readable results for CI/CD

## References

- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Clap Documentation](https://docs.rs/clap/)
- [Syn Documentation](https://docs.rs/syn/)
- [Walkdir Documentation](https://docs.rs/walkdir/)
