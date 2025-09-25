# QM Rust Agent Installation Guide

Easy installation of the Quine-McCluskey Boolean minimization agent for Claude Code.

## Quick Install

### Unix/Linux/macOS
```bash
./install.sh           # Install locally (current project only)
./install.sh --global  # Install globally (all projects)
```

### Windows
```cmd
install.bat           # Install locally (current project only)
install.bat --global  # Install globally (all projects)
```

## Installation Options

### Local Installation (Recommended for Testing)
Installs the agent for the current project only:
- Agent config: `./.claude/agents/qm-agent.md`
- Available only in current directory

```bash
# Unix/Linux/macOS
./install.sh --local

# Windows
install.bat --local
```

### Global Installation (Recommended for Regular Use)
Installs the agent globally for all Claude Code projects:
- Agent config: `~/.claude/agents/qm-agent.md`
- Available in all projects

```bash
# Unix/Linux/macOS
./install.sh --global

# Windows
install.bat --global
```

## Prerequisites

The installer will check for these requirements:

- **Rust & Cargo**: Install from [rustup.rs](https://rustup.rs/)
- **Git**: For cloning and version control
- **Claude Code**: The installer configures agents for Claude Code

## Manual Installation

If you prefer to install manually:

1. **Build the project**:
   ```bash
   cargo build --release
   ```

2. **Create agent directory**:
   ```bash
   # For global installation
   mkdir -p ~/.claude/agents

   # For local installation
   mkdir -p .claude/agents
   ```

3. **Copy agent configuration**:
   ```bash
   # For global installation
   cp .claude/agents/qm-agent.md ~/.claude/agents/

   # For local installation (already in place)
   # Agent config is already at .claude/agents/qm-agent.md
   ```

## Verification

After installation, verify the agent is working:

1. **Check agent is registered**:
   The agent should be available when Claude Code detects Boolean minimization tasks.

2. **Test basic functionality**:
   ```bash
   cargo run -- examples
   cargo run -- minimize -i "f(A,B) = Σ(1,3)"
   ```

3. **Test with Claude Code**:
   Ask Claude: *"Minimize the Boolean function f(A,B,C) = Σ(1,3,7)"*

## Usage

Once installed, the agent activates automatically when you ask Claude Code for:
- Boolean function minimization
- Karnaugh map simplification
- Digital logic optimization
- Quine-McCluskey algorithm tasks

### Example Requests
- "Minimize f(A,B,C) = Σ(1,3,7)"
- "Simplify this Karnaugh map"
- "Find prime implicants for minterms 1,3,7"
- "Optimize this Boolean expression"

## Troubleshooting

### Common Issues

**"Rust/Cargo not found"**
- Install Rust from [rustup.rs](https://rustup.rs/)
- Restart your terminal after installation

**"Agent not found"**
- Ensure you're in the project directory
- Check that `.claude/agents/qm-agent.md` exists
- Try running the installer again

**"Build failed"**
- Check Rust installation: `cargo --version`
- Ensure all dependencies are available
- Try cleaning and rebuilding: `cargo clean && cargo build --release`

**"Permission denied" (Unix/Linux)**
- Make the installer executable: `chmod +x install.sh`
- Run with appropriate permissions

### Getting Help

1. Check the [README.md](README.md) for usage documentation
2. Review [CLAUDE.md](CLAUDE.md) for technical details
3. Run `cargo run -- examples` for usage examples
4. Create an issue on the project repository

## Uninstalling

To remove the agent:

### Local Installation
```bash
rm -rf .claude/agents
```

### Global Installation
```bash
rm -rf ~/.claude/agents/qm-agent.md
```

## Advanced Configuration

The agent configuration is stored in the markdown file:
- Local: `./.claude/agents/qm-agent.md`
- Global: `~/.claude/agents/qm-agent.md`

You can customize:
- Agent description and trigger conditions
- Available tools and permissions
- System prompt and behavior

Edit the file to modify the agent's behavior according to your needs.