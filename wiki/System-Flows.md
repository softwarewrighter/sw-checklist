# System Flows

This document details the execution flows through sw-checklist using sequence diagrams and flowcharts.

## Table of Contents

1. [Main Execution Flow](#main-execution-flow)
2. [Project Discovery Flow](#project-discovery-flow)
3. [Check Orchestration Flow](#check-orchestration-flow)
4. [Clap Check Flow](#clap-check-flow)
5. [Modularity Check Flow](#modularity-check-flow)
6. [Result Aggregation Flow](#result-aggregation-flow)
7. [Error Handling Flow](#error-handling-flow)

## Main Execution Flow

### Complete End-to-End Sequence

```mermaid
sequenceDiagram
    actor User
    participant CLI as CLI Parser
    participant Main as main()
    participant Discovery as Project Discovery
    participant Orchestrator as Check Orchestrator
    participant Checks as Check Modules
    participant Output as Output Formatter

    User->>CLI: sw-checklist /path/to/project
    CLI->>Main: Parse arguments
    Main->>Main: Canonicalize path

    Main->>Discovery: find_cargo_tomls(path)
    Discovery->>Discovery: Walk directory tree
    Discovery-->>Main: Vec<PathBuf> (Cargo.toml files)

    alt No Cargo.toml found
        Main->>Output: Print error
        Main->>User: Exit code 1
    end

    Main->>Main: Detect project types
    Main->>Output: Print project info

    Main->>Orchestrator: run_checks(root, tomls, verbose)

    loop For each crate
        Orchestrator->>Orchestrator: Parse Cargo.toml
        Orchestrator->>Orchestrator: Extract crate name

        alt Has clap dependency
            Orchestrator->>Checks: check_rust_crate()
            Checks-->>Orchestrator: Vec<CheckResult>
        end

        alt Has wasm dependency
            Orchestrator->>Checks: check_wasm_crate()
            Checks-->>Orchestrator: Vec<CheckResult>
        end

        Orchestrator->>Checks: check_modularity()
        Checks-->>Orchestrator: Vec<CheckResult>
    end

    Orchestrator-->>Main: Aggregated results

    Main->>Main: Add project-level checks
    Main->>Output: print_results(results)
    Output->>User: Display formatted output

    Main->>Main: Calculate summary
    Main->>Output: Print summary
    Output->>User: Display summary

    alt Has failures
        Main->>User: Exit code 1
    else All passed
        Main->>User: Exit code 0
    end
```

### Simplified Flow Diagram

```mermaid
flowchart TD
    Start([User runs sw-checklist]) --> Parse[Parse CLI Arguments]
    Parse --> Canon[Canonicalize Path]
    Canon --> Find[Find Cargo.toml Files]
    Find --> Empty{Any found?}

    Empty -->|No| Error[Print Error]
    Error --> Exit1[Exit Code 1]

    Empty -->|Yes| Detect[Detect Project Types]
    Detect --> Print1[Print Project Info]
    Print1 --> Loop{For each Cargo.toml}

    Loop -->|Process| ParseToml[Parse Cargo.toml]
    ParseToml --> ExtractName[Extract Crate Name]
    ExtractName --> HasClap{Has clap dependency?}

    HasClap -->|Yes| CheckClap[Run Clap Checks]
    HasClap -->|No| HasWasm{Has wasm dependency?}

    CheckClap --> AlwaysMod[Run Modularity Checks]
    HasWasm -->|Yes| CheckWasm[Run WASM Checks]
    HasWasm -->|No| AlwaysMod

    CheckWasm --> AlwaysMod
    AlwaysMod --> Collect[Collect Results]
    Collect --> Loop

    Loop -->|Done| AddProject[Add Project-Level Checks]
    AddProject --> PrintResults[Print Check Results]
    PrintResults --> Summary[Calculate Summary]
    Summary --> PrintSum[Print Summary]
    PrintSum --> Failed{Any failures?}

    Failed -->|Yes| Exit1
    Failed -->|No| Exit0[Exit Code 0]

    style Start fill:#e1f5ff
    style Exit0 fill:#e1ffe1
    style Exit1 fill:#ffe1e1
```

## Project Discovery Flow

### Detailed Discovery Sequence

```mermaid
sequenceDiagram
    participant Main
    participant Discovery as discovery::find_cargo_tomls
    participant WalkDir
    participant FileSystem

    Main->>Discovery: find_cargo_tomls(path)
    Discovery->>WalkDir: WalkDir::new(path)
    WalkDir->>WalkDir: into_iter()

    loop For each directory entry
        WalkDir->>FileSystem: Read entry
        FileSystem-->>WalkDir: DirEntry | Error

        alt Entry is Error
            WalkDir->>WalkDir: filter_map(skip)
        else Entry is Ok
            WalkDir->>WalkDir: Check file_name
            alt file_name == "Cargo.toml"
                WalkDir->>Discovery: Add to results
            else
                WalkDir->>WalkDir: Continue
            end
        end
    end

    Discovery-->>Main: Vec<PathBuf>
```

### Type Detection Flow

```mermaid
flowchart TD
    Start([Start Type Detection]) --> Init[Initialize flags: has_cli, has_wasm, has_yew]

    Init --> LoopStart{For each Cargo.toml}

    LoopStart -->|Process| Read[Read Cargo.toml content]
    Read --> CheckClap{Contains 'clap'?}
    CheckClap -->|Yes| SetCLI[has_cli = true]
    CheckClap -->|No| CheckWasm{Contains 'wasm-bindgen'?}

    SetCLI --> CheckWasm
    CheckWasm -->|Yes| SetWasm[has_wasm = true]
    CheckWasm -->|No| CheckYew{Contains 'yew'?}

    SetWasm --> CheckYew
    CheckYew -->|Yes| SetYew[has_yew = true]
    CheckYew -->|No| LoopStart

    SetYew --> LoopStart

    LoopStart -->|Done| Determine{Determine Type}

    Determine -->|has_cli && has_yew| TypeCLIYew[Type: CLI + Yew]
    Determine -->|has_cli && has_wasm| TypeCLIWasm[Type: CLI + WASM]
    Determine -->|has_yew| TypeYew[Type: Yew WASM]
    Determine -->|has_wasm| TypeWasm[Type: WASM]
    Determine -->|has_cli| TypeCLI[Type: CLI]
    Determine -->|otherwise| TypeLib[Type: Rust Library]

    TypeCLIYew --> Return([Return Type])
    TypeCLIWasm --> Return
    TypeYew --> Return
    TypeWasm --> Return
    TypeCLI --> Return
    TypeLib --> Return

    style Return fill:#e1ffe1
```

## Check Orchestration Flow

### Orchestration Sequence

```mermaid
sequenceDiagram
    participant Main
    participant RunChecks as run_checks()
    participant FileSystem as File System
    participant ClapCheck as checks::clap
    participant WasmCheck as checks::wasm
    participant ModCheck as checks::modularity

    Main->>RunChecks: run_checks(root, tomls, verbose)
    RunChecks->>RunChecks: Create results Vec

    loop For each Cargo.toml path
        alt verbose mode
            RunChecks->>Main: Print checking message
        end

        RunChecks->>FileSystem: Read Cargo.toml
        FileSystem-->>RunChecks: TOML content

        RunChecks->>RunChecks: Parse TOML
        RunChecks->>RunChecks: Extract crate name

        RunChecks->>RunChecks: Check dependencies

        alt Has clap dependency
            RunChecks->>ClapCheck: check_rust_crate(root, dir, verbose)
            ClapCheck-->>RunChecks: Vec<CheckResult>
            RunChecks->>RunChecks: Extend results
        else if Has wasm dependency
            RunChecks->>WasmCheck: check_wasm_crate(root, dir, verbose)
            WasmCheck-->>RunChecks: Vec<CheckResult>
            RunChecks->>RunChecks: Extend results
        end

        Note over RunChecks: Always run modularity checks

        RunChecks->>ModCheck: check_modularity(dir, name)
        ModCheck-->>RunChecks: Vec<CheckResult>
        RunChecks->>RunChecks: Extend results
    end

    alt No results collected
        RunChecks->>RunChecks: Add default pass result
    end

    RunChecks-->>Main: Vec<CheckResult>
```

## Clap Check Flow

### Binary Discovery and Validation

```mermaid
flowchart TD
    Start([check_rust_crate]) --> CheckDep{Has clap dependency?}

    CheckDep -->|No| SkipCheck[Return: Skipped]
    CheckDep -->|Yes| AddPass[Add: Dependency Check PASS]

    AddPass --> FindBin[Find binaries in target/]
    FindBin --> ParseToml[Parse Cargo.toml]
    ParseToml --> GetBinNames[Extract binary names]

    GetBinNames --> SearchDebug[Search target/debug/]
    SearchDebug --> SearchRelease[Search target/release/]

    SearchRelease --> AnyBin{Found any binaries?}

    AnyBin -->|No| WarnNoBin[Add: No Binary WARN]
    AnyBin -->|Yes| LoopBin{For each binary}

    WarnNoBin --> Return([Return Results])

    LoopBin -->|Process| ExecHelp[Execute: binary -h]
    ExecHelp --> ExecHelpLong[Execute: binary --help]
    ExecHelpLong --> CompareHelp{--help longer than -h?}

    CompareHelp -->|Yes| PassHelp[Add: Help Length PASS]
    CompareHelp -->|No| FailHelp[Add: Help Length FAIL]

    PassHelp --> CheckAI{--help contains AI instructions?}
    FailHelp --> CheckAI

    CheckAI -->|Yes| PassAI[Add: AI Instructions PASS]
    CheckAI -->|No| FailAI[Add: AI Instructions FAIL]

    PassAI --> ExecVersion[Execute: binary -V]
    FailAI --> ExecVersion
    ExecVersion --> ExecVersionLong[Execute: binary --version]

    ExecVersionLong --> CompareVer{-V equals --version?}

    CompareVer -->|Yes| PassVer[Add: Version Consistency PASS]
    CompareVer -->|No| FailVer[Add: Version Consistency FAIL]

    PassVer --> CheckFields[Check version fields]
    FailVer --> CheckFields

    CheckFields --> CheckCopy{Contains Copyright?}
    CheckCopy -->|Yes| PassCopy[Add: Copyright PASS]
    CheckCopy -->|No| FailCopy[Add: Copyright FAIL]

    PassCopy --> CheckLic{Contains License?}
    FailCopy --> CheckLic

    CheckLic -->|Yes| PassLic[Add: License PASS]
    CheckLic -->|No| FailLic[Add: License FAIL]

    PassLic --> CheckRepo{Contains Repository?}
    FailLic --> CheckRepo

    CheckRepo -->|Yes| PassRepo[Add: Repository PASS]
    CheckRepo -->|No| FailRepo[Add: Repository FAIL]

    PassRepo --> CheckHost{Contains Build Host?}
    FailRepo --> CheckHost

    CheckHost -->|Yes| PassHost[Add: Build Host PASS]
    CheckHost -->|No| FailHost[Add: Build Host FAIL]

    PassHost --> CheckCommit{Contains Build Commit?}
    FailHost --> CheckCommit

    CheckCommit -->|Yes| PassCommit[Add: Build Commit PASS]
    CheckCommit -->|No| FailCommit[Add: Build Commit FAIL]

    PassCommit --> CheckTime{Contains Build Time?}
    FailCommit --> CheckTime

    CheckTime -->|Yes| PassTime[Add: Build Time PASS]
    CheckTime -->|No| FailTime[Add: Build Time FAIL]

    PassTime --> CheckFresh[Check binary freshness]
    FailTime --> CheckFresh

    CheckFresh --> LoopBin

    LoopBin -->|Done| Return

    style Start fill:#e1f5ff
    style Return fill:#e1ffe1
```

## Modularity Check Flow

### Function LOC Counting Algorithm

```mermaid
flowchart TD
    Start([check_modularity]) --> FindRS[Find all .rs files in src/]
    FindRS --> InitResults[Initialize results Vec]
    InitResults --> InitCounts[Initialize module_counts HashMap]

    InitCounts --> LoopFiles{For each .rs file}

    LoopFiles -->|Process| ReadFile[Read file lines]
    ReadFile --> InitFuncCount[function_count = 0]
    InitFuncCount --> LoopLines{For each line}

    LoopLines -->|Process| CheckFunc{Line starts with fn or pub fn?}

    CheckFunc -->|No| NextLine[i++]
    CheckFunc -->|Yes| ExtractName[Extract function name]

    ExtractName --> FindBrace[Find opening brace line]
    FindBrace --> CountBraces[Count braces to find end]

    CountBraces --> CalcLOC[loc = end_line - start_line + 1]
    CalcLOC --> CheckLOC{loc > 50?}

    CheckLOC -->|Yes| FailLOC[Add: Function LOC FAIL]
    CheckLOC -->|No| WarnLOC{loc > 25?}

    WarnLOC -->|Yes| WarnLOCRes[Add: Function LOC WARN]
    WarnLOC -->|No| PassLOC[Add: Function LOC PASS]

    FailLOC --> IncFunc[function_count++]
    WarnLOCRes --> IncFunc
    PassLOC --> IncFunc

    IncFunc --> JumpEnd[i = end_line + 1]
    JumpEnd --> LoopLines

    NextLine --> LoopLines

    LoopLines -->|Done| CheckModFunc{function_count > 7?}

    CheckModFunc -->|Yes| FailModFunc[Add: Module Function Count FAIL]
    CheckModFunc -->|No| WarnModFunc{function_count > 4?}

    WarnModFunc -->|Yes| WarnModFuncRes[Add: Module Function Count WARN]
    WarnModFunc -->|No| PassModFunc[Add: Module Function Count PASS]

    FailModFunc --> StoreCount[Store function_count]
    WarnModFuncRes --> StoreCount
    PassModFunc --> StoreCount

    StoreCount --> CheckFileLOC{File lines > 500?}

    CheckFileLOC -->|Yes| FailFileLOC[Add: File LOC FAIL]
    CheckFileLOC -->|No| WarnFileLOC{File lines > 350?}

    WarnFileLOC -->|Yes| WarnFileLOCRes[Add: File LOC WARN]
    WarnFileLOC -->|No| PassFileLOC[Add: File LOC PASS]

    FailFileLOC --> LoopFiles
    WarnFileLOCRes --> LoopFiles
    PassFileLOC --> LoopFiles

    LoopFiles -->|Done| CountMods[module_count = files.len()]
    CountMods --> CheckCrateMod{module_count > 7?}

    CheckCrateMod -->|Yes| FailCrateMod[Add: Crate Module Count FAIL]
    CheckCrateMod -->|No| WarnCrateMod{module_count > 4?}

    WarnCrateMod -->|Yes| WarnCrateModRes[Add: Crate Module Count WARN]
    WarnCrateMod -->|No| PassCrateMod[Add: Crate Module Count PASS]

    FailCrateMod --> Return([Return Results])
    WarnCrateModRes --> Return
    PassCrateMod --> Return

    style Start fill:#e1f5ff
    style Return fill:#e1ffe1
```

### Brace Counting Detail

```mermaid
sequenceDiagram
    participant Parser
    participant Lines as Line Iterator
    participant Counter as Brace Counter

    Parser->>Parser: Found function signature
    Parser->>Lines: Find line with opening brace

    loop Search for opening brace
        Lines->>Parser: Next line
        Parser->>Parser: Check if contains '{'
        alt Contains '{'
            Parser->>Parser: brace_line = current
            Parser->>Counter: Start counting from this line
        end
    end

    Counter->>Counter: brace_count = 0

    loop Count braces
        Counter->>Lines: Get next line
        Lines-->>Counter: Line content

        loop For each character in line
            Counter->>Counter: Check character
            alt char == '{'
                Counter->>Counter: brace_count++
            else if char == '}'
                Counter->>Counter: brace_count--
            end

            alt brace_count == 0
                Counter->>Parser: end_line = current
                Note over Counter,Parser: Function body complete
            end
        end
    end

    Parser->>Parser: Calculate LOC = end_line - start_line + 1
```

## Result Aggregation Flow

### Result Collection and Summary

```mermaid
sequenceDiagram
    participant Main
    participant RunChecks
    participant PrintResults
    participant Terminal

    Main->>RunChecks: Get all check results
    RunChecks-->>Main: Vec<CheckResult>

    Main->>Main: Add sw-install check
    Main->>Main: Add project crate count check

    Main->>PrintResults: print_results(results)

    PrintResults->>Terminal: Print header
    Terminal-->>PrintResults:
    PrintResults->>Terminal: Print separator (80 =)

    loop For each result
        PrintResults->>PrintResults: Determine status symbol
        alt is_warning
            PrintResults->>PrintResults: status = "⚠ WARN"
        else if passed
            PrintResults->>PrintResults: status = "✓ PASS"
        else
            PrintResults->>PrintResults: status = "✗ FAIL"
        end

        PrintResults->>Terminal: Print status | name
        PrintResults->>Terminal: Print indented message
        PrintResults->>Terminal: Print blank line
    end

    PrintResults-->>Main: Done

    Main->>Main: Count passed results
    Main->>Main: Count failed results
    Main->>Main: Count warnings

    alt Has warnings
        Main->>Terminal: Print: "Summary: X passed, Y failed, Z warnings"
    else No warnings
        Main->>Terminal: Print: "Summary: X passed, Y failed"
    end

    alt failed > 0
        Main->>Main: exit_code = 1
    else
        Main->>Main: exit_code = 0
    end

    Main->>Terminal: Exit with code
```

## Error Handling Flow

### Error Context Propagation

```mermaid
flowchart TD
    Start([Function Call]) --> TryOp{Try Operation}

    TryOp -->|Success| UseValue[Use Result Value]
    TryOp -->|Error| AddContext[Add context with with_context]

    AddContext --> Propagate[Propagate with ? operator]
    Propagate --> Caller{Caller can handle?}

    Caller -->|Yes - Skip| Skip[Continue with other checks]
    Caller -->|Yes - Recover| Recover[Use default value]
    Caller -->|No| PropUp[Propagate up further]

    PropUp --> Main{Reached main?}

    Main -->|Yes| PrintErr[Print error with context]
    Main -->|No| Caller

    PrintErr --> Exit[Exit with code 1]

    UseValue --> Continue[Continue execution]
    Skip --> Continue
    Recover --> Continue

    Continue --> Return([Return])

    style Start fill:#e1f5ff
    style Exit fill:#ffe1e1
    style Return fill:#e1ffe1
```

### Graceful Degradation

```mermaid
sequenceDiagram
    participant Orchestrator
    participant CheckModule
    participant FileSystem
    participant Results

    Orchestrator->>CheckModule: Perform check

    CheckModule->>FileSystem: Read file
    alt File accessible
        FileSystem-->>CheckModule: File content
        CheckModule->>CheckModule: Process content
        CheckModule->>Results: Add check result
    else File not found
        FileSystem-->>CheckModule: Error
        CheckModule->>CheckModule: Log warning (if verbose)
        Note over CheckModule: Skip this file, continue
    end

    CheckModule->>FileSystem: Read next file
    alt File accessible
        FileSystem-->>CheckModule: File content
        CheckModule->>CheckModule: Process content
        CheckModule->>Results: Add check result
    else Parse error
        FileSystem-->>CheckModule: Error
        CheckModule->>CheckModule: Log warning (if verbose)
        Note over CheckModule: Skip this file, continue
    end

    CheckModule-->>Orchestrator: Partial results (what succeeded)
    Note over Orchestrator: Continue with other checks
```

## Performance Considerations

### Parallel Opportunities (Future)

```mermaid
graph TD
    subgraph "Current: Sequential"
        S1[Check Crate 1] --> S2[Check Crate 2]
        S2 --> S3[Check Crate 3]
        S3 --> S4[Aggregate Results]
    end

    subgraph "Future: Parallel"
        P1[Check Crate 1]
        P2[Check Crate 2]
        P3[Check Crate 3]
        P1 --> P4[Aggregate Results]
        P2 --> P4
        P3 --> P4
    end

    style S1 fill:#ffe1e1
    style P1 fill:#e1ffe1
    style P2 fill:#e1ffe1
    style P3 fill:#e1ffe1
```

## Related Documentation

- **[Architecture Overview](Architecture-Overview)** - System architecture and components
- **[Component Details](Component-Details)** - Detailed component documentation
- **[Check Orchestration](Check-Orchestration)** - Check coordination details
- **[Modularity Checks](Modularity-Checks)** - Modularity validation details
