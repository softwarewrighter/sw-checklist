# Product Requirements Document (PRD)
# sw-checklist

**Version**: 0.2.0
**Date**: 2025-12-03
**Status**: Active Development
**Owner**: Software Wrighter LLC

## Executive Summary

sw-checklist is a CLI tool that validates Rust projects against Software Wrighter LLC standards and best practices. It automatically detects project types and crate types, running appropriate validation checks based on what each crate actually contains. It supports both traditional single-workspace repositories and new-style multi-component repositories.

## Problem Statement

### Current Pain Points

1. **Inconsistent Standards**: Projects lack uniform structure, documentation, and metadata
2. **Poor Modularity**: Code grows into monolithic files and functions that are hard to maintain
3. **Missing Metadata**: CLI tools lack proper version information, help text, and build metadata
4. **Manual Validation**: No automated way to verify project conformance
5. **AI Agent Guidance**: AI coding agents need clear, programmatic standards to follow
6. **Multi-Component Projects**: New project structures with multiple independent components need proper support
7. **Over-enforcement**: CLI checks were running on library crates, WASM checks on non-WASM crates

### Target Users

1. **Human Developers**: Software Wrighter LLC team members
2. **AI Coding Agents**: Claude Code and similar tools that generate/modify code
3. **CI/CD Pipelines**: Automated validation in continuous integration workflows

## Goals and Objectives

### Primary Goals

1. **Automated Validation**: Programmatically check projects against standards
2. **Clear Feedback**: Provide actionable error messages and warnings
3. **Dogfooding**: Tool must validate itself and pass all its own checks
4. **AI-Friendly**: Output designed for both human and AI consumption

### Success Metrics

- All Software Wrighter LLC projects pass sw-checklist validation
- Zero manual effort required for standards enforcement
- <1 second execution time for small projects
- 100% test coverage for validation logic
- Clear, actionable feedback in all failure cases

## Features and Requirements

### Feature 1: Project and Crate Type Detection

**Priority**: P0 (Must Have)

**Description**: Automatically detect project structure and individual crate types from files and dependencies.

**Requirements**:

**Project Structure Detection**:
- Detect Rust projects via Cargo.toml presence
- Support three repository structures:
  1. Single-crate: Root Cargo.toml with package section
  2. Workspace: Root Cargo.toml with [workspace] section
  3. Multi-component: No root Cargo.toml, components/ directory with independent workspaces
- Handle nested crate structures

**Crate Type Detection** (per crate):
- Identify CLI crates via clap dependency or [[bin]] section
- Identify WASM/Web UI crates via:
  - wasm-bindgen dependency
  - yew dependency
  - `crate-type = ["cdylib"]` in [lib] section
  - Trunk.toml presence
- Identify library crates (no CLI/WASM markers)

**Acceptance Criteria**:
- Correctly identifies single-crate projects
- Correctly identifies workspace projects
- Correctly identifies multi-component projects
- Correctly classifies each crate type (CLI, WASM, Library) independently
- Reports accurate count of Cargo.toml files
- Displays detected project structure to user
- Only runs CLI checks on CLI crates
- Only runs WASM checks on WASM crates

### Feature 2: Clap CLI Validation

**Priority**: P0 (Must Have)

**Description**: Validate CLI tools built with clap for proper help and version output. **Only applies to crates that have the clap dependency**.

**Applicability**:
- Only runs on crates with `clap` in dependencies
- Does NOT run on library crates or WASM-only crates
- A multi-component project may have CLI crates in some components and not others

**Requirements**:

**Help Output**:
- `-h` flag produces short help
- `--help` flag produces extended help (must be longer than `-h`)
- `--help` includes "AI CODING AGENT INSTRUCTIONS" section
- Help text is clear and actionable

**Version Output**:
- `-V` and `--version` produce identical output
- Version output includes all metadata:
  - Version number
  - Copyright notice
  - License information
  - Repository URL
  - Build host information
  - Build commit SHA
  - Build timestamp

**Acceptance Criteria**:
- All checks pass for properly configured CLI tools
- Clear error messages when requirements not met
- Validates all binaries in multi-binary crates
- Works with debug and release builds
- Does NOT run on non-CLI crates (library, WASM-only)

### Feature 3: Modularity Validation

**Priority**: P0 (Must Have)

**Description**: Enforce code modularity following the 7±2 rule (Miller's Law).

**Requirements**:

**Function Lines of Code (LOC)**:
- Warn if function exceeds 25 lines
- Fail if function exceeds 50 lines
- Count actual code lines (signature to closing brace)
- Provide function name and file location in errors

**Module Function Count**:
- Warn if module has more than 4 functions
- Fail if module has more than 7 functions
- Count all function definitions in file
- Report module name (file name) in errors

**Crate Module Count**:
- Warn if crate has more than 4 modules (.rs files)
- Fail if crate has more than 7 modules
- Count all .rs files in src/ directory
- Report crate name in errors

**Component Crate Count** (for multi-component projects):
- Warn if a component has more than 4 crates
- Fail if a component has more than 7 crates
- Count Cargo.toml files per component (not project-wide)
- Report at component level with component name

**Project Crate Count** (for old-style projects):
- Warn if project has more than 4 crates
- Fail if project has more than 7 crates
- Count all Cargo.toml files found
- Report at project level

**Component Count** (for multi-component projects):
- Warn if project has more than 7 components
- No hard failure on component count (warning only)
- Unlimited components allowed, just advisory warning

**Acceptance Criteria**:
- Correctly counts functions in all Rust files
- Correctly counts modules (files) in each crate
- Correctly counts crates per component in multi-component projects
- Correctly counts crates project-wide in old-style projects
- Warning and failure thresholds work as specified
- Test functions are counted but don't cause false positives
- Clear, actionable error messages with file:line references

### Feature 4: WASM Project Validation

**Priority**: P1 (Should Have)

**Description**: Validate WASM frontend projects for required assets and metadata. **Only applies to crates that use wasm-bindgen or Yew**.

**Applicability**:
- Only runs on crates with `wasm-bindgen` or `yew` dependencies
- Only runs on crates with `crate-type = ["cdylib"]`
- Does NOT run on CLI crates or library crates
- A multi-component project may have WASM crates in some components and not others

**Requirements**:
- Verify index.html exists
- Verify index.html references favicon
- Verify favicon.ico exists
- Verify footer contains required metadata:
  - Copyright notice
  - License link
  - Repository link
  - Build host
  - Build commit
  - Build timestamp

**Acceptance Criteria**:
- All checks pass for properly configured WASM projects
- Works with Yew and other WASM frameworks
- Clear guidance on missing elements
- Does NOT run on non-WASM crates (CLI, library)

### Feature 5: Binary Freshness Check

**Priority**: P2 (Nice to Have)

**Description**: Warn if local build is newer than installed binary.

**Requirements**:
- Compare modification times of target/release binary vs ~/.local/softwarewrighter/bin binary
- Only warn if installed binary exists
- Suggest running acceptance tests and sw-install
- Don't fail, only warn

**Acceptance Criteria**:
- Warning appears when local build is newer
- No warning when binaries are in sync
- No warning when binary not yet installed
- Message includes clear next steps

### Feature 6: sw-install Presence Check

**Priority**: P2 (Nice to Have)

**Description**: Check if sw-install tool is installed.

**Requirements**:
- Check for sw-install in ~/.local/softwarewrighter/bin/
- Warn if not found
- Don't fail, only warn
- Provide installation instructions

**Acceptance Criteria**:
- Warning when sw-install not found
- Pass when sw-install is installed
- Clear installation URL in warning message

## User Experience

### Command Line Interface

```bash
# Check current directory
sw-checklist

# Check specific project
sw-checklist /path/to/project

# Verbose output
sw-checklist -v /path/to/project

# Help
sw-checklist --help

# Version
sw-checklist --version
```

### Output Format

**Success Case**:
```
Checking project: /path/to/project
Project type: CLI
Found 1 Cargo.toml file(s)

Check Results:
================================================================================
✓ PASS | Clap Dependency [my-crate]
       Found clap dependency in my-crate

✓ PASS | Function LOC [my-crate]
       All functions are under 25 lines

Summary: 12 passed, 0 failed
```

**Failure Case**:
```
Check Results:
================================================================================
✗ FAIL | Function LOC [my-crate]
       Function 'process_data' in lib.rs has 87 lines (max 50)

⚠ WARN | Module Function Count [my-crate]
       Module utils.rs has 5 functions (warning at >4, max 7)

Summary: 10 passed, 1 failed, 1 warnings
```

### AI Agent Integration

The `--help` output includes a dedicated "AI CODING AGENT INSTRUCTIONS" section that provides:
- Clear usage instructions for AI agents
- Current checks being performed
- Modularity philosophy and rationale
- Repository link for detailed documentation

## Technical Requirements

### Platform Support

- macOS (primary)
- Linux (tested)
- Windows (untested but should work)

### Performance

- Execution time <1s for projects with <10 crates
- Execution time <5s for large workspace projects
- Memory usage <100MB for typical projects

### Dependencies

**Runtime**:
- clap: CLI argument parsing
- toml: Cargo.toml parsing
- walkdir: File tree traversal
- anyhow: Error handling

**Build Time**:
- chrono: Build timestamp generation
- hostname: Build host detection

**Development**:
- tempfile: Test fixtures
- Standard Rust toolchain

### Error Handling

- All errors use anyhow::Result
- Clear error messages with context
- Graceful handling of missing files
- Skip unparseable files with warning
- Non-zero exit code on failures

## Non-Functional Requirements

### Reliability

- All code paths have test coverage
- Tests use TDD approach (red-green-refactor)
- Zero clippy warnings allowed
- All code formatted with rustfmt

### Maintainability

- Functions <50 LOC (enforced by tool itself)
- Modules <7 functions (enforced by tool itself)
- Clear separation of concerns
- Comprehensive inline documentation

### Security

- No unsafe code
- No external network access
- Read-only file operations (except test fixtures)
- No execution of user-provided code

## Out of Scope

The following are explicitly out of scope for version 0.1.0:

- Configuration files (.sw-checklist.toml)
- Custom check thresholds per-project
- Plugin system for custom checks
- IDE integration
- Test coverage analysis
- Dependency vulnerability scanning
- Code complexity metrics beyond LOC
- Automatic refactoring suggestions
- File size validation (planned for 0.2.0)

## Future Considerations

### Version 0.2.0

- File LOC validation (warn >350, fail >500)
- Configurable thresholds via config file
- Performance optimizations for large projects

### Version 0.3.0

- Test coverage validation
- Documentation completeness checks
- Dependency update warnings

### Version 1.0.0

- Stable API
- Plugin system
- CI/CD integration examples
- IDE extensions

## Appendix

### Modularity Philosophy

The 7±2 rule (Miller's Law) states that humans can hold approximately 7 (±2) items in working memory. This guides our modularity limits:

- **Functions (≤25 LOC ideal, 50 max)**: A function should do one thing and be comprehensible at a glance
- **Modules (≤4 functions ideal, 7 max)**: A module should have a clear, focused purpose
- **Crates (≤4 modules ideal, 7 max)**: A crate should be a cohesive unit of functionality
- **Components (≤4 crates ideal, 7 max)**: A component should have well-scoped boundaries
- **Projects**: May have unlimited components, but warn at >7

### Multi-Component Project Structure

New-style repositories organize code into independent components:

```
project/
├── components/
│   ├── spec/           # Shared data models (library crates)
│   ├── cli/            # Command-line interface (CLI crates)
│   └── web/            # Web UI (WASM crates)
└── docs/
```

Each component is a workspace with its own Cargo.toml, allowing:
- Independent versioning per component
- Separate CI/CD pipelines
- Clear ownership boundaries
- Cross-component dependencies via relative paths

### References

- [Miller's Law](https://en.wikipedia.org/wiki/The_Magical_Number_Seven,_Plus_or_Minus_Two)
- [The Cargo Book](https://doc.rust-lang.org/cargo/)
- [Clap Documentation](https://docs.rs/clap/)
- Software Wrighter LLC internal standards

### Revision History

| Version | Date | Changes |
|---------|------|---------|
| 0.1.0 | 2025-11-17 | Initial PRD with all implemented features |
| 0.2.0 | 2025-12-03 | Added multi-component support, per-crate type detection |
