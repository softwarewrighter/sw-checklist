# Architecture Document
# sw-checklist

**Version**: 0.2.0
**Date**: 2025-12-03
**Status**: Active

## Overview

sw-checklist is a single-binary CLI tool written in Rust that validates project conformance to Software Wrighter LLC standards. It uses a modular, check-based architecture that is easily extensible for new validation types.

The tool supports three repository structures:
1. **Single-crate**: Traditional single Cargo.toml at root
2. **Workspace**: Root Cargo.toml with `[workspace]` section and member crates
3. **Multi-component**: No root Cargo.toml, multiple independent component workspaces under `components/`

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
│  │  - Discover project structure                       │   │
│  │  - Detect components (if multi-component)           │   │
│  │  - Run checks per component/crate                   │   │
│  │  - Print results                                    │   │
│  └──────────────────┬──────────────────────────────────┘   │
│                     │                                       │
│                     ▼                                       │
│  ┌─────────────────────────────────────────────────────┐   │
│  │          Project Structure Discovery                │   │
│  │                                                      │   │
│  │  - Detect old-style vs new-style (multi-component)  │   │
│  │  - Find component-level workspace Cargo.toml files  │   │
│  │  - Group crates by component for limit checks       │   │
│  └──────────────────┬──────────────────────────────────┘   │
│                     │                                       │
│                     ▼                                       │
│  ┌─────────────────────────────────────────────────────┐   │
│  │          Check Orchestration Layer                  │   │
│  │                                                      │   │
│  │  run_checks() - Iterate crates and dispatch         │   │
│  │  - Type detection per crate (CLI, WASM, Library)    │   │
│  │  - Conditional check dispatch                       │   │
│  └──────┬──────────────────────────────────┬───────────┘   │
│         │                                   │               │
│         ▼                                   ▼               │
│  ┌──────────────┐                   ┌──────────────┐       │
│  │ Clap Checks  │                   │  Modularity  │       │
│  │ (CLI only)   │                   │    Checks    │       │
│  │              │                   │              │       │
│  │ - Help flags │                   │ - Fn LOC     │       │
│  │ - Version    │                   │ - Module fns │       │
│  │ - Metadata   │                   │ - Crate mods │       │
│  │ - Binaries   │                   │ - Component  │       │
│  └──────────────┘                   │   crate cnt  │       │
│                                     └──────────────┘       │
│         ▼                                   ▼               │
│  ┌──────────────┐                   ┌──────────────┐       │
│  │ WASM Checks  │                   │ Misc Checks  │       │
│  │ (WASM only)  │                   │              │       │
│  │              │                   │ - sw-install │       │
│  │ - HTML files │                   │ - Freshness  │       │
│  │ - Favicon    │                   │ - Component  │       │
│  │ - Footer     │                   │   count warn │       │
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

## Repository Structure Detection

### Old-Style Repositories

Traditional repositories have a Cargo.toml at the root level:

```
project/
├── Cargo.toml           # Root (workspace or single-crate)
├── src/
│   └── main.rs          # For single-crate
└── crates/              # Optional: workspace members
    ├── crate1/
    │   └── Cargo.toml
    └── crate2/
        └── Cargo.toml
```

### New-Style Multi-Component Repositories

New-style repositories have no root Cargo.toml. Instead, each component under `components/` is an independent workspace:

```
project/
├── components/
│   ├── component-a/           # First component workspace
│   │   ├── Cargo.toml         # Workspace Cargo.toml
│   │   ├── crate1/
│   │   │   └── Cargo.toml
│   │   └── crate2/
│   │       └── Cargo.toml
│   ├── component-b/           # Second component workspace
│   │   ├── Cargo.toml
│   │   └── crate1/
│   │       └── Cargo.toml
│   └── component-c/           # Third component workspace
│       ├── Cargo.toml
│       └── crate1/
│           └── Cargo.toml
└── docs/
```

### Detection Algorithm

```
1. Check for components/ directory at project root
2. If components/ exists AND no root Cargo.toml:
   - Treat as multi-component project
   - Each direct child of components/ is a component
   - Each component has its own workspace Cargo.toml
3. Else:
   - Treat as old-style project
   - Root Cargo.toml (or workspace root) defines project scope
```

## Crate Type Detection

Each crate is classified based on its Cargo.toml contents:

| Type | Detection Criteria | Checks Applied |
|------|-------------------|----------------|
| CLI | Has `clap` dependency | Help, version, metadata, binary freshness |
| WASM/Web UI | Has `wasm-bindgen`, `yew`, or `crate-type = ["cdylib"]` | HTML, favicon, footer |
| Library | No CLI/WASM markers | Modularity only |
| CLI + WASM | Both CLI and WASM markers | All checks |

### Binary vs Library Crate Detection

```
CLI Binary Crate:
  - Has [[bin]] section in Cargo.toml, OR
  - Has src/main.rs file, OR
  - Has clap dependency

WASM UI Crate:
  - Has crate-type = ["cdylib", "rlib"] in [lib], OR
  - Has Trunk.toml file, OR
  - Has wasm-bindgen/yew dependencies

Library Crate:
  - Has [lib] section without cdylib, OR
  - Has only src/lib.rs (no main.rs)
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

**Code Location**: `src/main.rs`

#### 2. Project Discovery

**Responsibilities**:
- Find all Cargo.toml files in project tree
- Detect project structure (old-style vs multi-component)
- Identify component boundaries in multi-component projects
- Detect crate type (CLI, WASM, Library)
- Handle workspace and multi-crate projects

**Key Functions**:
- `find_cargo_tomls(path)`: Recursively find all Cargo.toml files
- `is_multi_component_project(path)`: Detect new-style project structure
- `discover_components(path)`: Find component workspaces in components/
- `is_wasm_crate(cargo_toml)`: Check for WASM markers
- `is_cli_crate(cargo_toml)`: Check for CLI markers (clap, [[bin]])
- Uses walkdir for efficient traversal
- Returns paths to all discovered crate manifests

**Code Location**: `src/discovery.rs`

#### 3. Check Orchestration

**Responsibilities**:
- Iterate over all discovered crates
- Dispatch appropriate checks based on crate type (not project type)
- Aggregate results from all checks
- Apply crate count limits per-component for multi-component projects

**Key Functions**:
- `run_checks(project_root, cargo_tomls, verbose)`: Main orchestration
  - Parses each Cargo.toml
  - Determines crate type (clap/wasm/library) individually
  - Runs type-specific checks only on matching crates:
    - CLI checks only on crates with clap dependency
    - WASM checks only on crates with wasm-bindgen/yew
  - Runs modularity checks on all crates
  - Returns aggregated results

**Code Location**: `src/main.rs`

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
4. **Component Crate Count**: Count crates per component (warn >4, fail >7)
5. **Component Count Warning**: Warn if project has >7 components

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

For multi-component projects:
    Group crates by component
    For each component:
        Count crates in that component
        Apply crate count limits (warn >4, fail >7)
    Count total components
    Warn if >7 components (no failure, just warning)
```

**Key Functions**:
- `check_modularity()`: Main entry point
- `extract_function_name()`: Parse function name from signature
- `check_component_crate_counts()`: Validate crate limits per component
- Uses simple brace counting (not full AST parsing)

**Design Decisions**:
- Simple line-based parsing (fast, good enough)
- Could use syn for full AST parsing in future
- Counts all functions including tests (intentional)
- Warnings before failures (progressive feedback)
- Crate limits are per-component, not project-wide in multi-component repos
- No hard limit on component count (warn at >7 only)

**Code Location**: `src/checks/modularity.rs`

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

2. Detect project structure
   └─> Check for components/ directory
   └─> Check for root Cargo.toml
   └─> Determine: old-style vs multi-component

3. Find all Cargo.toml files
   └─> Recursively walk directory tree
   └─> Collect all manifest paths
   └─> For multi-component: group by component

4. For each crate:
   a. Parse Cargo.toml
   b. Extract crate name
   c. Detect crate type (CLI, WASM, Library)
   d. If has clap (CLI crate): run clap checks
   e. If has wasm-bindgen/yew (WASM crate): run wasm checks
   f. Always: run modularity checks

5. Add project-level checks
   └─> sw-install presence
   └─> For old-style: project crate count
   └─> For multi-component:
       └─> Per-component crate counts
       └─> Component count warning (if >7)

6. Aggregate results
   └─> Count passes, failures, warnings
   └─> Format output
   └─> Exit with code (0 = pass, 1 = failures)
```

### Multi-Component Project Flow

```
1. Detect multi-component structure
   └─> components/ exists AND no root Cargo.toml

2. Discover components
   └─> List directories in components/
   └─> Each with Cargo.toml is a component

3. For each component:
   a. Find crates (Cargo.toml files) within component
   b. Identify component's workspace root
   c. Run crate-type-specific checks on each crate
   d. Count crates in this component
   e. Apply per-component crate limits (warn >4, fail >7)

4. Project-level component count
   └─> Count total components
   └─> Warn if >7 components (no hard failure)
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
