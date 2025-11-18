# Check Orchestration

This document details how sw-checklist coordinates and executes validation checks across different project types.

## Overview

Check orchestration is the process of determining which checks to run for each crate and coordinating their execution. The system uses a dynamic, type-based approach to select appropriate validations.

## Orchestration Architecture

```mermaid
graph TB
    subgraph "Entry"
        Main[main function]
    end

    subgraph "Discovery Phase"
        Find[find_cargo_tomls]
        Detect[detect_project_types]
    end

    subgraph "Orchestration Phase"
        RunChecks[run_checks]
        Router{Type Router}
    end

    subgraph "Execution Phase"
        ClapExec[Execute Clap Checks]
        WasmExec[Execute WASM Checks]
        ModExec[Execute Modularity Checks]
    end

    subgraph "Aggregation Phase"
        Collect[Collect Results]
        AddProject[Add Project-Level Checks]
    end

    Main --> Find
    Find --> Detect
    Detect --> RunChecks
    RunChecks --> Router

    Router -->|Has clap| ClapExec
    Router -->|Has wasm| WasmExec
    Router -->|Always| ModExec

    ClapExec --> Collect
    WasmExec --> Collect
    ModExec --> Collect

    Collect --> AddProject
    AddProject --> Main

    style Main fill:#e1f5ff
    style RunChecks fill:#ffe1f5
    style Collect fill:#e1ffe1
```

## Orchestration Flow

### 1. Discovery Phase

**Purpose**: Find all crates in the project

```rust
// In main()
let cargo_tomls = discovery::find_cargo_tomls(&project_path);

if cargo_tomls.is_empty() {
    // No Rust project found
    exit(1);
}
```

**Output**: `Vec<PathBuf>` containing paths to all Cargo.toml files

**Example**:
```
[
  "/project/Cargo.toml",
  "/project/cli/Cargo.toml",
  "/project/lib/Cargo.toml",
  "/project/wasm-ui/Cargo.toml"
]
```

### 2. Type Detection Phase

**Purpose**: Classify project types to determine which checks apply

```rust
let mut has_cli = false;
let mut has_wasm = false;
let mut has_yew = false;

for cargo_toml_path in &cargo_tomls {
    let cargo_toml = fs::read_to_string(cargo_toml_path)?;

    if cargo_toml.contains("clap") {
        has_cli = true;
    }
    if cargo_toml.contains("wasm-bindgen") {
        has_wasm = true;
    }
    if cargo_toml.contains("yew") {
        has_yew = true;
    }
}
```

**Type Classification**:

```mermaid
flowchart TD
    Start([Start Classification]) --> CLI{Has clap?}

    CLI -->|Yes| Yew1{Has yew?}
    CLI -->|No| Wasm1{Has wasm?}

    Yew1 -->|Yes| TypeCLIYew["CLI + Yew"]
    Yew1 -->|No| Wasm2{Has wasm?}

    Wasm2 -->|Yes| TypeCLIWasm["CLI + WASM"]
    Wasm2 -->|No| TypeCLI["CLI"]

    Wasm1 -->|Yes| Yew2{Has yew?}
    Wasm1 -->|No| TypeLib["Rust Library"]

    Yew2 -->|Yes| TypeYew["Yew (WASM)"]
    Yew2 -->|No| TypeWasm["WASM"]

    TypeCLIYew --> End([Project Type Determined])
    TypeCLIWasm --> End
    TypeCLI --> End
    TypeYew --> End
    TypeWasm --> End
    TypeLib --> End

    style End fill:#e1ffe1
```

**Project Type Matrix**:

| Has clap | Has wasm-bindgen | Has yew | Project Type |
|----------|------------------|---------|-------------|
| ✓ | ✓ | ✓ | CLI + Yew |
| ✓ | ✓ | ✗ | CLI + WASM |
| ✓ | ✗ | ✗ | CLI |
| ✗ | ✓ | ✓ | Yew (WASM) |
| ✗ | ✓ | ✗ | WASM |
| ✗ | ✗ | ✗ | Rust Library |

### 3. Check Execution Phase

**Purpose**: Run appropriate checks for each crate

```rust
fn run_checks(
    project_root: &Path,
    cargo_tomls: &[PathBuf],
    verbose: bool,
) -> Result<Vec<CheckResult>> {
    let mut results = Vec::new();

    for cargo_toml_path in cargo_tomls {
        // Parse Cargo.toml
        let cargo_toml = fs::read_to_string(cargo_toml_path)?;
        let cargo: toml::Value = toml::from_str(&cargo_toml)?;

        let crate_name = cargo
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown");

        let crate_dir = cargo_toml_path.parent().unwrap();

        // Type-specific checks
        let has_clap = cargo_toml.contains("clap");
        let is_wasm = discovery::is_wasm_crate(&cargo_toml);

        if has_clap {
            results.extend(
                checks::clap::check_rust_crate(project_root, crate_dir, verbose)?
            );
        } else if is_wasm {
            results.extend(
                checks::wasm::check_wasm_crate(project_root, crate_dir, verbose)?
            );
        }

        // Universal checks (all Rust crates)
        results.extend(
            checks::modularity::check_modularity(crate_dir, crate_name)?
        );
    }

    Ok(results)
}
```

### 4. Aggregation Phase

**Purpose**: Combine results and add project-level checks

```rust
// In main()
let mut results = run_checks(&project_root, &cargo_tomls, cli.verbose)?;

// Add project-level checks
results.push(checks::install::check_sw_install_presence());

// Project crate count check
let crate_count = cargo_tomls.len();
if crate_count > 7 {
    results.push(CheckResult::fail(
        "Project Crate Count",
        format!("Project has {} crates (max 7)", crate_count)
    ));
} else if crate_count > 4 {
    results.push(CheckResult::warn(
        "Project Crate Count",
        format!("Project has {} crates (warning at >4, max 7)", crate_count)
    ));
} else {
    results.push(CheckResult::pass(
        "Project Crate Count",
        format!("Project has {} crates (4 or fewer)", crate_count)
    ));
}
```

## Check Selection Matrix

### By Project Type

| Project Type | Clap Checks | WASM Checks | Modularity Checks | Test Checks | Install Checks |
|-------------|-------------|-------------|-------------------|-------------|----------------|
| CLI | ✓ | ✗ | ✓ | Optional | ✓ |
| CLI + WASM | ✓ | ✗ | ✓ | Optional | ✓ |
| CLI + Yew | ✓ | ✗ | ✓ | Optional | ✓ |
| WASM | ✗ | ✓ | ✓ | Optional | ✓ |
| Yew (WASM) | ✗ | ✓ | ✓ | Optional | ✓ |
| Rust Library | ✗ | ✗ | ✓ | Optional | ✓ |

### By Crate Level

```mermaid
graph TD
    subgraph "Crate-Level Checks"
        C1[Clap Checks]
        C2[WASM Checks]
        C3[Modularity Checks]
    end

    subgraph "Project-Level Checks"
        P1[sw-install Presence]
        P2[Project Crate Count]
    end

    subgraph "Per-Crate Execution"
        Crate1[Crate 1] --> C1
        Crate1 --> C3
        Crate2[Crate 2] --> C2
        Crate2 --> C3
        Crate3[Crate 3] --> C3
    end

    subgraph "Single Execution"
        Project[Project] --> P1
        Project --> P2
    end

    style C1 fill:#e1f5ff
    style C2 fill:#e1f5ff
    style C3 fill:#ffe1f5
    style P1 fill:#e1ffe1
    style P2 fill:#e1ffe1
```

## Detailed Check Workflows

### Clap Check Workflow

```mermaid
sequenceDiagram
    participant Orchestrator
    participant ClapCheck
    participant BinaryFinder
    participant Executor
    participant Results

    Orchestrator->>ClapCheck: check_rust_crate(root, dir, verbose)
    ClapCheck->>ClapCheck: Verify clap dependency

    ClapCheck->>BinaryFinder: get_binary_names(Cargo.toml)
    BinaryFinder-->>ClapCheck: Vec<String> (binary names)

    ClapCheck->>BinaryFinder: find_binaries(root, names)
    BinaryFinder-->>ClapCheck: HashMap<name, path>

    loop For each binary
        ClapCheck->>Executor: Execute binary -h
        Executor-->>ClapCheck: Short help output

        ClapCheck->>Executor: Execute binary --help
        Executor-->>ClapCheck: Long help output

        ClapCheck->>ClapCheck: Compare outputs
        ClapCheck->>Results: Add help check results

        ClapCheck->>Executor: Execute binary -V
        Executor-->>ClapCheck: Version output

        ClapCheck->>Executor: Execute binary --version
        Executor-->>ClapCheck: Version output

        ClapCheck->>ClapCheck: Compare outputs
        ClapCheck->>ClapCheck: Validate metadata fields
        ClapCheck->>Results: Add version check results

        ClapCheck->>ClapCheck: Check binary freshness
        ClapCheck->>Results: Add freshness check
    end

    Results-->>Orchestrator: Vec<CheckResult>
```

### WASM Check Workflow

```mermaid
sequenceDiagram
    participant Orchestrator
    participant WasmCheck
    participant FileSystem
    participant Results

    Orchestrator->>WasmCheck: check_wasm_crate(root, dir, verbose)

    WasmCheck->>FileSystem: Check index.html exists
    FileSystem-->>WasmCheck: exists: bool

    alt index.html exists
        WasmCheck->>FileSystem: Read index.html
        FileSystem-->>WasmCheck: HTML content

        WasmCheck->>WasmCheck: Check for favicon reference
        WasmCheck->>Results: Add HTML check result
    else
        WasmCheck->>Results: Add FAIL: Missing index.html
    end

    WasmCheck->>FileSystem: Check favicon.ico exists
    FileSystem-->>WasmCheck: exists: bool
    WasmCheck->>Results: Add favicon check result

    WasmCheck->>FileSystem: Find all .rs files
    FileSystem-->>WasmCheck: Vec<PathBuf>

    loop For each .rs file
        WasmCheck->>FileSystem: Read file content
        FileSystem-->>WasmCheck: Source code

        WasmCheck->>WasmCheck: Search for footer code
        alt Footer found
            WasmCheck->>WasmCheck: Validate footer fields
            WasmCheck->>Results: Add footer check results
        end
    end

    Results-->>Orchestrator: Vec<CheckResult>
```

### Modularity Check Workflow

```mermaid
sequenceDiagram
    participant Orchestrator
    participant ModCheck
    participant FileSystem
    participant Parser
    participant Results

    Orchestrator->>ModCheck: check_modularity(dir, name)

    ModCheck->>FileSystem: Find all .rs files in src/
    FileSystem-->>ModCheck: Vec<PathBuf>

    loop For each .rs file
        ModCheck->>FileSystem: Read file
        FileSystem-->>ModCheck: File content (lines)

        ModCheck->>Parser: Parse functions
        loop For each function
            Parser->>Parser: Find opening brace
            Parser->>Parser: Count braces to closing
            Parser->>Parser: Calculate LOC

            alt LOC > 50
                Parser->>Results: Add FAIL: Function too long
            else if LOC > 25
                Parser->>Results: Add WARN: Function getting long
            else
                Parser->>Results: Add PASS: Function size OK
            end
        end

        Parser->>Parser: Count functions in file

        alt function_count > 7
            Parser->>Results: Add FAIL: Too many functions
        else if function_count > 4
            Parser->>Results: Add WARN: Many functions
        else
            Parser->>Results: Add PASS: Function count OK
        end

        ModCheck->>ModCheck: Count file lines

        alt file_lines > 500
            ModCheck->>Results: Add FAIL: File too long
        else if file_lines > 350
            ModCheck->>Results: Add WARN: File getting long
        else
            ModCheck->>Results: Add PASS: File size OK
        end
    end

    ModCheck->>ModCheck: Count total modules

    alt module_count > 7
        ModCheck->>Results: Add FAIL: Too many modules
    else if module_count > 4
        ModCheck->>Results: Add WARN: Many modules
    else
        ModCheck->>Results: Add PASS: Module count OK
    end

    Results-->>Orchestrator: Vec<CheckResult>
```

## Error Handling in Orchestration

### Graceful Degradation Strategy

```mermaid
flowchart TD
    Start([Process Crate]) --> TryRead{Try read Cargo.toml}

    TryRead -->|Success| Parse{Try parse TOML}
    TryRead -->|Error| LogError1[Log error if verbose]

    LogError1 --> Skip1[Skip this crate]
    Skip1 --> NextCrate[Process next crate]

    Parse -->|Success| RunChecks[Run checks]
    Parse -->|Error| LogError2[Log error if verbose]

    LogError2 --> Skip2[Skip this crate]
    Skip2 --> NextCrate

    RunChecks --> CheckResult{Check succeeds?}

    CheckResult -->|Success| CollectResults[Collect results]
    CheckResult -->|Error| LogError3[Log error if verbose]

    LogError3 --> PartialResults[Use partial results]
    PartialResults --> CollectResults

    CollectResults --> NextCrate

    NextCrate --> More{More crates?}

    More -->|Yes| Start
    More -->|No| Aggregate[Aggregate all results]

    Aggregate --> Return([Return results])

    style Return fill:#e1ffe1
```

### Error Context Example

```rust
// In run_checks()
for cargo_toml_path in cargo_tomls {
    // Add context to file operations
    let cargo_toml = fs::read_to_string(cargo_toml_path)
        .with_context(|| {
            format!("Failed to read Cargo.toml at {:?}", cargo_toml_path)
        })?;

    // Add context to parsing
    let cargo: toml::Value = toml::from_str(&cargo_toml)
        .with_context(|| {
            format!("Failed to parse Cargo.toml at {:?}", cargo_toml_path)
        })?;

    // Check execution with context
    if has_clap {
        results.extend(
            checks::clap::check_rust_crate(project_root, crate_dir, verbose)
                .with_context(|| {
                    format!("Failed clap checks for crate at {:?}", crate_dir)
                })?
        );
    }
}
```

## Parallel Execution (Future)

### Current Sequential Execution

```mermaid
gantt
    title Current Sequential Processing
    dateFormat X
    axisFormat %L ms

    section Crate 1
    Clap checks     :c1, 0, 100
    Modularity      :c2, after c1, 50

    section Crate 2
    WASM checks     :c3, after c2, 80
    Modularity      :c4, after c3, 50

    section Crate 3
    Modularity      :c5, after c4, 50

    section Aggregate
    Collect results :c6, after c5, 10
```

**Total Time**: ~340ms

### Potential Parallel Execution

```mermaid
gantt
    title Potential Parallel Processing
    dateFormat X
    axisFormat %L ms

    section Crate 1
    Clap checks     :p1, 0, 100
    Modularity      :p2, 0, 50

    section Crate 2
    WASM checks     :p3, 0, 80
    Modularity      :p4, 0, 50

    section Crate 3
    Modularity      :p5, 0, 50

    section Aggregate
    Collect results :p6, after p1 p2 p3 p4 p5, 10
```

**Total Time**: ~110ms (3x speedup)

### Implementation Approach (Future)

```rust
use rayon::prelude::*;

fn run_checks_parallel(
    project_root: &Path,
    cargo_tomls: &[PathBuf],
    verbose: bool,
) -> Result<Vec<CheckResult>> {
    // Process crates in parallel
    let results: Vec<_> = cargo_tomls
        .par_iter()
        .map(|cargo_toml_path| {
            check_single_crate(project_root, cargo_toml_path, verbose)
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();

    Ok(results)
}

fn check_single_crate(
    project_root: &Path,
    cargo_toml_path: &Path,
    verbose: bool,
) -> Result<Vec<CheckResult>> {
    // Same logic as current run_checks, but for single crate
    let mut results = Vec::new();
    // ... existing check logic ...
    Ok(results)
}
```

## Integration with AI Agents

### AI Agent Workflow

```mermaid
sequenceDiagram
    actor AI as AI Agent
    participant CLI as sw-checklist
    participant Orchestrator
    participant Checks
    participant Output

    AI->>CLI: Run sw-checklist on project
    CLI->>Orchestrator: Execute checks

    loop For each check type
        Orchestrator->>Checks: Run checks
        Checks-->>Orchestrator: Results
    end

    Orchestrator->>Output: Format results
    Output-->>CLI: Formatted output
    CLI-->>AI: Display results

    AI->>AI: Parse output
    AI->>AI: Identify failures
    AI->>AI: Generate fixes

    AI->>AI: Apply fixes to code
    AI->>CLI: Run sw-checklist again
    CLI->>Orchestrator: Re-execute checks
    Orchestrator-->>CLI: New results
    CLI-->>AI: Verification results

    alt All passed
        AI->>AI: Commit changes
    else Some failed
        AI->>AI: Continue fixing
    end
```

### AI-Friendly Output Structure

The orchestrator ensures output is:
- **Deterministic**: Same checks, same order
- **Parseable**: Clear status indicators (PASS/FAIL/WARN)
- **Actionable**: Specific file/function names included
- **Comprehensive**: All issues reported at once

## Performance Characteristics

### Time Complexity

```
O(n × m × l)

Where:
- n = number of crates
- m = average modules per crate
- l = average lines per module
```

### Space Complexity

```
O(c × r)

Where:
- c = number of crates
- r = average results per crate
```

### Typical Performance

| Project Size | Crates | Checks | Time |
|-------------|--------|--------|------|
| Small | 1-3 | 10-30 | <500ms |
| Medium | 4-10 | 40-100 | 1-2s |
| Large | 11-20 | 100-200 | 3-5s |

## Related Documentation

- **[Architecture Overview](Architecture-Overview)** - System architecture
- **[System Flows](System-Flows)** - Detailed execution flows
- **[Modularity Checks](Modularity-Checks)** - Modularity validation details
- **[Clap CLI Checks](Clap-CLI-Checks)** - CLI validation details
- **[WASM Checks](WASM-Checks)** - WASM validation details
