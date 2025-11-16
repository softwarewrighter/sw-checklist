# sw-checklist

CLI tool for validating Software Wrighter LLC project conformance requirements.

## Overview

`sw-checklist` inspects a project directory and checks for compliance with Software Wrighter LLC standards and best practices. It automatically detects project types and runs appropriate validation checks.

## Features

- **Automatic Project Detection**: Identifies Rust projects and more
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

# Verbose output
sw-checklist -v /path/to/project
```

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
