# Architecture Overview

This document provides a comprehensive overview of the sw-checklist system architecture, including component diagrams and design patterns.

## High-Level Architecture

```mermaid
graph TB
    subgraph "User Layer"
        User[User/AI Agent]
        Terminal[Terminal/CLI]
    end

    subgraph "Entry Point"
        Main[main.rs<br/>CLI Parser]
        Args[Command Line<br/>Arguments]
    end

    subgraph "Discovery Layer"
        FindCargo[find_cargo_tomls<br/>Walk Directory Tree]
        DetectType[Detect Project Type<br/>CLI/WASM/Library]
    end

    subgraph "Orchestration Layer"
        RunChecks[run_checks<br/>Iterate & Dispatch]
        CheckRouter{Check Router}
    end

    subgraph "Check Modules"
        ClapMod[checks::clap<br/>CLI Validation]
        WASMMod[checks::wasm<br/>WASM Validation]
        ModMod[checks::modularity<br/>Size Validation]
        TestMod[checks::tests<br/>Test Validation]
        InstallMod[checks::install<br/>Installation Checks]
    end

    subgraph "Data Model"
        CheckResult[CheckResult<br/>pass/fail/warn]
    end

    subgraph "Output Layer"
        Aggregate[Result Aggregation<br/>Count Pass/Fail/Warn]
        Format[Format Output<br/>Terminal Display]
        Exit[Exit Code<br/>0=pass, 1=fail]
    end

    User --> Terminal
    Terminal --> Main
    Main --> Args
    Main --> FindCargo
    FindCargo --> DetectType
    DetectType --> RunChecks
    RunChecks --> CheckRouter

    CheckRouter -->|has clap| ClapMod
    CheckRouter -->|has wasm| WASMMod
    CheckRouter -->|always| ModMod
    CheckRouter -.->|optional| TestMod
    CheckRouter -.->|project-level| InstallMod

    ClapMod --> CheckResult
    WASMMod --> CheckResult
    ModMod --> CheckResult
    TestMod --> CheckResult
    InstallMod --> CheckResult

    CheckResult --> Aggregate
    Aggregate --> Format
    Format --> Exit
    Exit --> Terminal
    Terminal --> User

    style Main fill:#e1f5ff
    style RunChecks fill:#e1f5ff
    style CheckResult fill:#ffe1f5
    style Aggregate fill:#e1ffe1
```

## System Layers

### 1. Entry Point Layer

**Purpose**: Parse CLI arguments and initialize the application

**Components**:
- `main()` function in `src/main.rs`
- `Cli` struct using clap derive macros
- Build-time metadata (commit SHA, timestamp, build host)

**Responsibilities**:
- Parse command-line arguments (project path, verbose flag)
- Display version information with metadata
- Provide help text including AI agent instructions
- Canonicalize project path

### 2. Discovery Layer

**Purpose**: Find and identify all crates in the project

**Components**:
- `discovery::find_cargo_tomls()` - Find all Cargo.toml files
- `discovery::is_wasm_crate()` - Detect WASM projects
- Project type detection logic

**Algorithm**:
```mermaid
flowchart TD
    Start([Start Discovery]) --> Walk[Walk Directory Tree<br/>using walkdir]
    Walk --> Find{Found<br/>Cargo.toml?}
    Find -->|Yes| Collect[Collect Path]
    Find -->|No| Continue
    Collect --> Continue[Continue Walking]
    Continue --> More{More<br/>Entries?}
    More -->|Yes| Find
    More -->|No| Analyze[Analyze Dependencies]
    Analyze --> HasClap{Contains<br/>clap?}
    HasClap -->|Yes| MarkCLI[Mark as CLI]
    HasClap -->|No| HasWASM{Contains<br/>wasm-bindgen?}
    HasWASM -->|Yes| MarkWASM[Mark as WASM]
    HasWASM -->|No| MarkLib[Mark as Library]
    MarkCLI --> Return([Return Crates])
    MarkWASM --> Return
    MarkLib --> Return
```

**Key Features**:
- Recursive directory traversal
- Workspace support (finds all member crates)
- Multi-crate project support
- Automatic type detection

### 3. Orchestration Layer

**Purpose**: Coordinate execution of appropriate checks for each crate

**Component**: `run_checks()` function in `src/main.rs`

**Flow**:
```mermaid
sequenceDiagram
    participant Main
    participant RunChecks
    participant ClapCheck
    participant WASMCheck
    participant ModCheck
    participant Results

    Main->>RunChecks: run_checks(root, tomls, verbose)

    loop For each Cargo.toml
        RunChecks->>RunChecks: Parse Cargo.toml
        RunChecks->>RunChecks: Extract crate name
        RunChecks->>RunChecks: Detect dependencies

        alt Has clap dependency
            RunChecks->>ClapCheck: check_rust_crate()
            ClapCheck-->>Results: Vec<CheckResult>
        end

        alt Has wasm-bindgen
            RunChecks->>WASMCheck: check_wasm_crate()
            WASMCheck-->>Results: Vec<CheckResult>
        end

        RunChecks->>ModCheck: check_modularity()
        ModCheck-->>Results: Vec<CheckResult>
    end

    Results-->>Main: Aggregated Results
```

**Responsibilities**:
- Iterate over all discovered crates
- Parse each Cargo.toml file
- Determine which checks to run
- Aggregate results from all checks
- Handle errors gracefully

### 4. Check Module Layer

**Purpose**: Perform specific validation checks

**Modules**:

#### checks::clap (src/checks/clap.rs)
- Binary discovery and validation
- Help flag comparison (-h vs --help)
- Version flag comparison (-V vs --version)
- Metadata validation (copyright, license, repo, build info)
- Binary freshness checks

#### checks::wasm (src/checks/wasm.rs)
- HTML file validation
- Favicon presence and references
- Footer metadata in source code

#### checks::modularity (src/checks/modularity.rs)
- Function LOC counting
- Module function counting
- Crate module counting
- File LOC validation

#### checks::tests (src/checks/tests.rs)
- Test directory presence
- Test annotation detection
- Jest test detection (for WASM)

#### checks::install (src/checks/install.rs)
- sw-install tool presence
- Binary installation freshness

### 5. Data Model Layer

**Purpose**: Represent check results uniformly

**Structure**:
```rust
pub struct CheckResult {
    pub name: String,      // Check identifier
    pub passed: bool,      // True if check passed
    pub message: String,   // Detailed message
    pub is_warning: bool,  // True for warnings vs failures
}
```

**Factory Methods**:
```rust
CheckResult::pass(name, message)  // Passed check
CheckResult::fail(name, message)  // Failed check
CheckResult::warn(name, message)  // Warning (not a failure)
```

**Characteristics**:
- Immutable after creation
- Self-contained (includes all display info)
- Serializable (future: JSON output)
- Type-safe construction

### 6. Output Layer

**Purpose**: Present results to the user

**Components**:
- `print_results()` - Format and display results
- Summary calculation (passed, failed, warnings)
- Exit code determination

**Output Format**:
```
Checking project: /path/to/project
Project type: CLI
Found 3 Cargo.toml file(s)

Check Results:
================================================================================
✓ PASS | Clap Dependency
       Found clap dependency

⚠ WARN | Function LOC [my-crate]
       Function 'large_function' in utils.rs has 30 lines (warning at >25, max 50)

✗ FAIL | Module Function Count [my-crate]
       Module src/helpers.rs has 8 functions (max 7)

Summary: 12 passed, 1 failed, 2 warnings
```

## Component Interaction Diagram

```mermaid
graph LR
    subgraph "main.rs"
        M1[main function]
        M2[run_checks]
        M3[print_results]
    end

    subgraph "discovery.rs"
        D1[find_cargo_tomls]
        D2[is_wasm_crate]
    end

    subgraph "checks/mod.rs"
        C1[CheckResult struct]
        C2[pass/fail/warn]
    end

    subgraph "checks/clap.rs"
        CL1[check_rust_crate]
        CL2[check_help_flags]
        CL3[check_version_flags]
    end

    subgraph "checks/modularity.rs"
        MO1[check_modularity]
        MO2[count_function_loc]
        MO3[count_module_functions]
    end

    subgraph "checks/wasm.rs"
        W1[check_wasm_crate]
        W2[check_html_files]
        W3[check_favicon]
    end

    M1 --> D1
    M1 --> M2
    M2 --> CL1
    M2 --> MO1
    M2 --> W1
    CL1 --> CL2
    CL1 --> CL3
    CL2 --> C2
    CL3 --> C2
    MO1 --> MO2
    MO1 --> MO3
    MO2 --> C2
    MO3 --> C2
    W1 --> W2
    W1 --> W3
    W2 --> C2
    W3 --> C2
    C1 --> M3
```

## Design Patterns

### 1. Single Responsibility Principle
Each check module has a single domain of validation:
- `clap.rs` - Only CLI argument validation
- `wasm.rs` - Only WASM project validation
- `modularity.rs` - Only size/complexity validation

### 2. Factory Pattern
`CheckResult` uses factory methods for type-safe construction:
```rust
CheckResult::pass("Check Name", "Success message")
CheckResult::fail("Check Name", "Failure message")
CheckResult::warn("Check Name", "Warning message")
```

### 3. Strategy Pattern
Check selection is dynamic based on project type:
- Has clap → Run CLI checks
- Has WASM → Run WASM checks
- All projects → Run modularity checks

### 4. Aggregate Pattern
Results are collected and aggregated:
```rust
let mut results = Vec::new();
results.extend(check_clap(...)?);
results.extend(check_wasm(...)?);
results.extend(check_modularity(...)?);
```

## Dependency Graph

```mermaid
graph TD
    main[main.rs] --> discovery[discovery.rs]
    main --> checks_mod[checks/mod.rs]
    main --> checks_clap[checks/clap.rs]
    main --> checks_wasm[checks/wasm.rs]
    main --> checks_modularity[checks/modularity.rs]
    main --> checks_install[checks/install.rs]

    checks_clap --> checks_mod
    checks_wasm --> checks_mod
    checks_modularity --> checks_mod
    checks_install --> checks_mod
    checks_tests[checks/tests.rs] --> checks_mod

    main --> utils[utils.rs]
    checks_clap --> utils

    style main fill:#e1f5ff
    style checks_mod fill:#ffe1f5
```

## Build System

```mermaid
graph LR
    subgraph "Build Time"
        BuildRS[build.rs] --> GitCommit[Get Git Commit SHA]
        BuildRS --> Timestamp[Get Build Timestamp]
        BuildRS --> Hostname[Get Build Host]
        GitCommit --> EnvVars[Set cargo:rustc-env]
        Timestamp --> EnvVars
        Hostname --> EnvVars
    end

    subgraph "Compile Time"
        EnvVars --> MainRS[main.rs]
        MainRS --> ConstFormat[const_format!]
        ConstFormat --> LongVersion[LONG_VERSION const]
    end

    subgraph "Runtime"
        LongVersion --> VersionOutput[--version output]
    end
```

**Build-time Constants**:
- `BUILD_COMMIT_SHA` - Git commit hash
- `BUILD_TIMESTAMP` - ISO 8601 build time
- `BUILD_HOST` - Hostname of build machine

## Error Handling Strategy

```mermaid
graph TD
    Op[Fallible Operation] --> Try{Try Operation}
    Try -->|Success| UseValue[Use Value]
    Try -->|Error| AddContext[Add Context<br/>with_context]
    AddContext --> PropagateUp[Propagate with ?]
    PropagateUp --> Caller{Caller Handles}
    Caller -->|Can Handle| Recover[Recover/Skip]
    Caller -->|Cannot Handle| Fail[Report & Exit]

    UseValue --> Continue[Continue]
    Recover --> Continue
```

**Principles**:
- All fallible operations return `Result<T, anyhow::Error>`
- Context added at each level with `.with_context()`
- Graceful degradation where possible
- Clear error messages with file paths

## Performance Characteristics

**Time Complexity**:
- Project discovery: O(n) where n = number of files
- Check execution: O(m × l) where m = number of modules, l = lines per module
- Overall: Linear with codebase size

**Space Complexity**:
- Results storage: O(c) where c = number of checks
- File reading: O(f) where f = largest file size
- Overall: Linear with project size

**Typical Performance**:
- Small projects (1-5 crates, <5k LOC): <500ms
- Medium projects (5-10 crates, <20k LOC): <2s
- Large projects (>10 crates, >50k LOC): <5s

## Extensibility Points

### Adding New Checks

1. Create new module in `src/checks/`
2. Implement check function returning `Vec<CheckResult>`
3. Call from `run_checks()` in main.rs
4. Add tests in module

**Example**:
```rust
// src/checks/documentation.rs
use super::CheckResult;

pub fn check_documentation(crate_dir: &Path, crate_name: &str)
    -> anyhow::Result<Vec<CheckResult>>
{
    let mut results = Vec::new();

    // Perform checks...
    if has_readme {
        results.push(CheckResult::pass(
            format!("README [{}]", crate_name),
            "README.md found"
        ));
    } else {
        results.push(CheckResult::fail(
            format!("README [{}]", crate_name),
            "Missing README.md"
        ));
    }

    Ok(results)
}
```

### Adding New Project Types

1. Add detection logic in main.rs project type detection
2. Create new check module if needed
3. Add to check orchestration in `run_checks()`

## Security Considerations

**Threat Model**:
- Assumes trusted project directories
- No network access
- No arbitrary code execution
- Read-only operations (except tests)

**Mitigations**:
- No `unsafe` code
- Path canonicalization
- Safe file operations
- No shell command execution with user input

## Related Documentation

- **[System Flows](System-Flows)** - Sequence diagrams and data flows
- **[Component Details](Component-Details)** - Detailed component documentation
- **[Design Decisions](Design-Decisions)** - Architectural decision records
