# sw-checklist

CLI tool for validating Software Wrighter LLC project conformance requirements.

## Overview

`sw-checklist` inspects a project directory and checks for compliance with Software Wrighter LLC standards and best practices. It automatically detects project types and runs appropriate validation checks.

## Features

- **Automatic Project Detection**: Identifies Rust projects, CLI tools, Web UI crates, and workspaces
- **Workspace-Aware**: Correctly handles multi-component projects with workspace Cargo.toml files
- **Clap CLI Validation**: For Rust projects using clap:
  - Verifies `-h` vs `--help` output (--help should be longer)
  - Checks for AI Coding Agent instructions in `--help`
  - Validates `-V` vs `--version` consistency
  - Ensures version output includes:
    - Copyright notice
    - License information
    - Repository URL
    - Build host
    - Build commit SHA
    - Build timestamp
- **Web UI Validation**: For WASM crates with web-serving indicators (index.html, static/, Trunk.toml):
  - Checks for index.html and favicon.ico
  - Validates footer presence and metadata (copyright, license, repository, build info)
  - Server-side WASM crates without UI indicators skip these checks
- **Modularity Checks**: For all Rust projects:
  - **Function LOC**: Warns if functions exceed 25 lines, fails if over 50 lines
  - **File LOC**: Warns if files exceed 350 lines, fails if over 500 lines
  - **Module Function Count**: Warns if modules have >4 functions, fails if >7
  - **Crate Module Count**: Warns if crates have >4 modules, fails if >7
  - **Project Crate Count**: Warns if projects have >4 crates, fails if >7 (excludes workspace Cargo.toml)
- **Extensible**: Easy to add new checks for different project types

## Installation

### From Source

```bash
git clone https://github.com/softwarewrighter/sw-checklist.git
cd sw-checklist
cargo build --release
```

The binary will be at `target/release/sw-checklist`.

### Using sw-install

If you have `sw-install` from the Software Wrighter toolchain:

```bash
cd /path/to/sw-checklist
cargo build --release
sw-install -p .
```

This installs to `~/.local/softwarewrighter/bin/sw-checklist`.

## Usage

### Basic Usage

```bash
# Check current directory
sw-checklist

# Check specific project
sw-checklist /path/to/project

# Verbose output (shows crate types and checks being run)
sw-checklist -v /path/to/project
```

Verbose mode shows:
- Each Cargo.toml being checked with crate name and type (workspace, CLI, WASM, library)
- Which checks are being run for each crate
- Workspaces are identified and skip CLI/WASM checks

### Help

```bash
# Short help
sw-checklist -h

# Extended help with AI agent instructions
sw-checklist --help
```

### Version

```bash
# Short version
sw-checklist -V

# Full version with build info
sw-checklist --version
```

## Example Output

```
Checking project: /Users/mike/github/softwarewrighter/sw-checklist
Project type: Rust

Check Results:
================================================================================
PASS | Clap Dependency
       Found clap dependency

PASS | Help Length
       --help (847 bytes) is longer than -h (423 bytes)

PASS | AI Agent Instructions
       Found AI Coding Agent section in --help

PASS | Version Consistency
       -V and --version produce identical output

PASS | Version Field: Copyright
       Found Copyright in version output

PASS | Version Field: License
       Found License in version output

Summary: 6 passed, 0 failed
```

## Checks Performed

### Rust Projects with Clap

1. **Dependency Check**: Confirms clap is in Cargo.toml
2. **Binary Exists**: Verifies the project has been built
3. **Help Flags**:
   - `-h` produces short help
   - `--help` produces extended help (must be longer)
   - `--help` includes "AI CODING AGENT INSTRUCTIONS" section
4. **Version Flags**:
   - `-V` and `--version` produce identical output
   - Version output includes:
     - Copyright notice: `Copyright (c)`
     - License: `MIT License`
     - Repository: `https://github.com`
     - Build Host: `Build Host:`
     - Build Commit: `Build Commit:`
     - Build Time: `Build Time:`

### Web UI Projects (WASM with UI indicators)

A crate is considered a Web UI if it has WASM dependencies AND web-serving indicators:
- `index.html` file
- `static/`, `public/`, `dist/`, `assets/`, or `www/` directory
- `Trunk.toml` file

Checks performed:
1. **index.html**: Must exist in crate root
2. **favicon.ico**: Must exist and be referenced in index.html
3. **Footer Metadata**: Source code should contain:
   - Copyright notice
   - License information
   - Repository link
   - Build host, commit, and timestamp

Server-side WASM crates (sandboxes, plugins) without these indicators skip UI checks.

### All Rust Projects (Modularity)

Following the 7±2 rule (Miller's Law) for cognitive limits:

1. **Function Lines of Code (LOC)**:
   - ⚠️ **Warning**: Functions with 26-50 lines
   - ❌ **Fail**: Functions with >50 lines
   - **Rationale**: Functions should do one thing well

2. **File Lines of Code (LOC)**:
   - ⚠️ **Warning**: Files with 351-500 lines
   - ❌ **Fail**: Files with >500 lines
   - **Rationale**: Large files indicate need to split into modules

3. **Module Function Count**:
   - ⚠️ **Warning**: Modules with 5-7 functions
   - ❌ **Fail**: Modules with >7 functions
   - **Rationale**: Modules should have a clear, focused purpose

4. **Crate Module Count**:
   - ⚠️ **Warning**: Crates with 5-7 modules
   - ❌ **Fail**: Crates with >7 modules
   - **Rationale**: Crates should be cohesive units

5. **Project Crate Count**:
   - ⚠️ **Warning**: Projects with 5-7 crates
   - ❌ **Fail**: Projects with >7 crates
   - **Rationale**: Projects should have well-scoped boundaries
   - **Note**: Workspace Cargo.toml files are not counted as crates

## Dogfooding

This tool validates itself! Run it on its own codebase:

```bash
cargo build --release
./target/release/sw-checklist .
```

All checks should pass, demonstrating that sw-checklist follows its own requirements.

## For AI Coding Agents

When using this tool in your workflow:

1. Run `sw-checklist` on the project you're working on
2. Review the check results and identify failures
3. Fix each failing check according to the guidance provided
4. Re-run `sw-checklist` to verify all checks pass
5. Only commit when all checks pass

The tool is designed to provide clear, actionable feedback for both humans and AI agents.

## License

MIT License

Copyright (c) 2025 Michael A Wright

See [LICENSE](LICENSE) for full text.

## Contributing

This tool is part of the Software Wrighter LLC toolchain. For issues or feature requests, please open an issue on GitHub.

## Repository

https://github.com/softwarewrighter/sw-checklist
