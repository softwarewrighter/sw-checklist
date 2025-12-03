# sw-checklist Development Plan

## Overview

This document outlines the development roadmap for sw-checklist, a CLI tool for validating Software Wrighter LLC project conformance requirements.

## Completed Features

### Phase 1: Core Infrastructure
- [x] Basic project structure
- [x] Build system with build-time metadata
- [x] CLI argument parsing with clap
- [x] Project type detection (Rust, CLI, WASM)

### Phase 2: Clap CLI Validation
- [x] Help output validation (-h vs --help)
- [x] Version output validation (-V vs --version)
- [x] AI Coding Agent instructions check
- [x] Version metadata validation (copyright, license, repository, build info)
- [x] Binary freshness checks
- [x] Multi-binary support
- [x] Workspace and multi-crate support

### Phase 3: WASM Project Validation
- [x] index.html presence and validation
- [x] favicon.ico checks
- [x] Footer metadata validation
- [x] Frontend build artifact checks

### Phase 4: Modularity Checks
- [x] Function LOC validation (warn >25, fail >50)
- [x] Module function count (warn >4, fail >7)
- [x] Crate module count (warn >4, fail >7)
- [x] Project crate count (warn >4, fail >7)
- [x] Support for all Rust projects (not just clap/WASM)

### Phase 5: File Size Validation
- [x] File LOC validation (warn >350, fail >500)

## Next Steps

### Phase 6: Multi-Component Project Support

**Objective**: Support new-style repositories with multiple independent component workspaces.

**Rationale**:
Modern projects may organize code into multiple independent components under a `components/` directory. Each component is a separate workspace with its own Cargo.toml. This structure allows:
- Independent versioning per component
- Clear separation of concerns (spec, cli, web)
- Scalability beyond the 7-crate limit

**Requirements**:

1. **Project Structure Detection**:
   - [ ] Detect multi-component projects (components/ dir, no root Cargo.toml)
   - [ ] Detect old-style projects (root Cargo.toml)
   - [ ] Support both structures in same tool

2. **Component Discovery**:
   - [ ] Find component directories under components/
   - [ ] Identify component workspace Cargo.toml files
   - [ ] Group crates by component for validation

3. **Per-Crate Type Detection**:
   - [ ] Detect CLI crates (clap dependency, [[bin]] section)
   - [ ] Detect WASM crates (wasm-bindgen, yew, cdylib)
   - [ ] Only run CLI checks on CLI crates
   - [ ] Only run WASM checks on WASM crates

4. **Component-Level Crate Counting**:
   - [ ] Apply 7-crate limit per component (not project-wide)
   - [ ] Warn at >4 crates per component
   - [ ] Fail at >7 crates per component
   - [ ] Warn (don't fail) if >7 components in project

5. **Testing**:
   - [ ] Test multi-component project detection
   - [ ] Test crate type detection accuracy
   - [ ] Test per-component crate counting
   - [ ] Test against alltalk-client-rs as reference

**Implementation Plan**:

1. **TDD Approach**:
   - Write tests for multi-component detection
   - Write tests for per-crate type detection
   - Write tests for component crate counting

2. **Discovery Module Updates** (`src/discovery.rs`):
   - Add `is_multi_component_project()` function
   - Add `discover_components()` function
   - Add `is_cli_crate()` function
   - Update `is_wasm_crate()` to be more robust

3. **Main Module Updates** (`src/main.rs`):
   - Update project structure detection
   - Conditionally run CLI checks only on CLI crates
   - Conditionally run WASM checks only on WASM crates
   - Update crate counting to be per-component

4. **Documentation**:
   - Update README.md with multi-component support
   - Update --help text
   - Update architecture, design, PRD docs

**Expected Output for Multi-Component Project**:

```
Checking project: /path/to/project
Project type: Multi-Component (3 components)

Components:
  - tts-spec (3 crates): Library
  - tts-cli (1 crate): CLI
  - tts-web (2 crates): CLI + WASM

Check Results:
================================================================================
✓ PASS | Component Crate Count [tts-spec]
       Component has 3 crates (4 or fewer)

✓ PASS | Component Crate Count [tts-cli]
       Component has 1 crate (4 or fewer)

✓ PASS | Component Crate Count [tts-web]
       Component has 2 crates (4 or fewer)

✓ PASS | Clap Dependency [ttsctl]
       Found clap dependency in ttsctl

✓ PASS | WASM Dependency [tts-web-ui]
       Found WASM dependencies in tts-web-ui

Summary: 25 passed, 0 failed, 2 warnings
```

### Phase 7: Additional Future Features

**Test Coverage Validation**:
- Check for presence of tests
- Validate test-to-code ratio
- Ensure critical paths have test coverage

**Documentation Checks**:
- Public API documentation completeness
- README.md quality and completeness
- CHANGELOG.md presence and format

**Dependency Validation**:
- Check for outdated dependencies
- Security vulnerability scanning
- License compatibility checks

**Code Quality Metrics**:
- Cyclomatic complexity analysis
- Code duplication detection
- Dead code identification

**Configuration**:
- Project-specific configuration via `.sw-checklist.toml`
- Custom check thresholds
- Exclude patterns for generated code

## Design Principles

1. **TDD First**: All features must have tests before implementation
2. **Dogfooding**: Tool must validate itself and pass all checks
3. **Clear Messaging**: Error messages must be actionable and helpful
4. **Progressive Validation**: Warnings before failures, allow gradual improvement
5. **Cognitive Limits**: Follow 7±2 rule for all quantitative checks
6. **Zero Configuration**: Sensible defaults, configuration optional
7. **Fast Feedback**: Checks should complete quickly (<1s for small projects)

## Release Strategy

### Version 0.2.0 (Next Release)
- Multi-component project support
- Per-crate type detection
- Per-component crate counting
- Updated documentation
- Bug fixes for erroneous "too many crates" errors

### Version 0.3.0
- Test coverage validation
- Documentation checks
- Configuration file support

### Version 1.0.0
- Stable API
- Comprehensive check suite
- Plugin system for custom checks
- CI/CD integration examples

## Contributing

When adding new checks:
1. Review this plan and update as needed
2. Follow TDD: Write tests first
3. Update docs/learnings.md with any issues encountered
4. Update README.md and --help text
5. Run full test suite and verify tool on itself
6. Update this plan with completion status
