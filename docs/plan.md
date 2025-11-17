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

## Next Steps

### Phase 5: File Size Validation

**Objective**: Prevent monolithic files by enforcing file size limits.

**Rationale**:
Even if individual functions are appropriately sized, a file can become unwieldy when it contains too many structs, implementations, or modules. Large files are harder to navigate, understand, and maintain. They often indicate that code should be better organized into separate files.

**Check Specification**:
- **File LOC Limits**:
  - ⚠️ **Warning**: Files with 351-500 lines
  - ❌ **Fail**: Files with >500 lines
  - ✓ **Pass**: Files with ≤350 lines

**Refactoring Guidance** (to be included in check messages):

When a file exceeds limits, consider these refactoring strategies in order:

1. **Extract Modules**: Move related functionality to separate module files
   - Example: Split `user.rs` into `user/mod.rs`, `user/auth.rs`, `user/profile.rs`

2. **Separate Structs and Implementations**:
   - Move large struct definitions and their implementations to dedicated files
   - Example: `database.rs` → `database/connection.rs`, `database/query_builder.rs`

3. **Use Traits for Organization**:
   - Define traits in one file, implementations in others
   - Example: `traits/repository.rs`, `repository/user_repo.rs`, `repository/post_repo.rs`

4. **Apply Composition Over Inheritance**:
   - Break large structs into smaller, composable pieces
   - Create focused types that can be combined
   - Example: Large `Config` struct → `DatabaseConfig`, `ServerConfig`, `LoggingConfig`

5. **Module Hierarchy**:
   - Use subdirectories for related files
   - Create `mod.rs` to re-export public API
   - Keep implementation details in separate files

**Implementation Plan**:

1. **TDD Approach** (following existing patterns):
   - Write tests for files under 350 LOC (pass)
   - Write tests for files with 351-500 LOC (warning)
   - Write tests for files with >500 LOC (fail)
   - Test should create temporary files with known line counts

2. **Implementation**:
   - Extend `check_modularity()` function to count file lines
   - Track file LOC alongside existing metrics
   - Add file-level results to modularity check output
   - Provide actionable error messages with refactoring suggestions

3. **Integration**:
   - Add to existing modularity check flow
   - Update summary to include file LOC statistics
   - Ensure check runs on all `.rs` files in `src/`

4. **Documentation**:
   - Update README.md with file LOC check description
   - Update --help text with file size limits
   - Add refactoring examples to AI agent instructions
   - Document in learnings.md if patterns emerge

5. **Testing**:
   - Test with files of various sizes
   - Test with actual monolithic files
   - Verify refactoring suggestions are helpful
   - Ensure no false positives on legitimately large files

**Expected Output Example**:
```
✗ FAIL | File LOC [my-crate]
       File main.rs has 687 lines (max 500)
       Consider: Extract modules, separate structs/impls, or use traits

⚠ WARN | File LOC [my-crate]
       File utils.rs has 423 lines (warning at >350, max 500)
       Consider: Extract related functionality to separate files
```

**Future Enhancements**:
- Exclude test modules from LOC counts (optional)
- Configurable limits per-project via `.sw-checklist.toml`
- Automatic refactoring suggestions based on code analysis
- Integration with IDEs for real-time feedback

### Phase 6: Additional Future Features

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
- File LOC validation
- Updated documentation
- Performance improvements
- Bug fixes from user feedback

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
