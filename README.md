# ksec - Kubernetes Secrets at your fingertips

A command-line tool for easy access to Kubernetes secrets with intelligent shell completions.

## Features

- Quick access to Kubernetes secrets and their keys
- Support for multiple kubeconfig files and contexts
- Smart shell completions for Bash and Zsh
- Dynamic completion of:
    - Kubernetes contexts
    - Namespaces
    - Secret names
    - Secret keys

## Installation

### From Source

```bash
git clone <repository-url>
cd ksec
cargo build --release
sudo cp target/release/ksec /usr/local/bin/
```

## Usage

### Basic Usage

```bash
# Get a secret value (first key if multiple keys exist)
ksec my-secret

# Get a specific key from a secret
ksec my-secret my-key

# Use specific context and namespace
ksec -c my-context -n my-namespace my-secret my-key

# Use custom kubeconfig file
ksec --kubeconfig ~/.kube/other-config my-secret
```

## Shell Completions

ksec supports intelligent shell completions for both Bash and Zsh that can autocomplete:

- Kubernetes contexts from your kubeconfig
- Namespaces from your cluster
- Secret names from the selected namespace
- Keys from the selected secret

### Bash Completion

#### Install completion script:

```bash
# Generate and install the completion script
ksec --completion=bash dummy > /etc/bash_completion.d/ksec

# Or for user-specific installation:
ksec --completion=bash dummy > ~/.local/share/bash-completion/completions/ksec
```

#### Load in current session:

```bash
source <(ksec --completion=bash dummy)
```

### Zsh Completion

#### Install completion script:

```bash
# For system-wide installation:
ksec --completion=zsh dummy > /usr/local/share/zsh/site-functions/_ksec

# For user-specific installation (oh-my-zsh):
ksec --completion=zsh dummy > ~/.oh-my-zsh/completions/_ksec

# For user-specific installation (standard):
mkdir -p ~/.zfunc
ksec --completion=zsh dummy > ~/.zfunc/_ksec
echo 'fpath+=~/.zfunc' >> ~/.zshrc
```

#### Load in current session:

```bash
source <(ksec --completion=zsh dummy)
```

### Completion Examples

Once installed, you can use tab completion:

```bash
# Complete contexts
ksec -c <TAB>

# Complete namespaces
ksec -n <TAB>

# Complete secret names
ksec <TAB>

# Complete secret keys
ksec my-secret <TAB>
```

## Options

- `--kubeconfig <PATH>`: Path to kubeconfig file (default: `$KUBECONFIG` or `~/.kube/config`)
- `-c, --context <CONTEXT>`: Kubernetes context to use
- `-n, --namespace <NAMESPACE>`: Kubernetes namespace to use
- `--completion <SHELL>`: Generate completion script for bash or zsh
- `-h, --help`: Print help information
- `-V, --version`: Print version information

## Examples

```bash
# Get database password from production context
ksec -c prod-cluster -n database db-secret password

# List all keys in a secret (when no key specified, shows first key)
ksec app-config

# Use completion to navigate
ksec -c <TAB>           # Shows available contexts
ksec -n <TAB>           # Shows available namespaces  
ksec <TAB>              # Shows available secrets
ksec my-secret <TAB>    # Shows available keys in secret
```

## Planned Features

The following features are planned for future releases:

- Shell completions for powershell
- Shell completions for fish
- Shell completions for JSON navigation
- Support for editing secrets in-place
- Shell completions for YAML navigation
- Integration with external secret management systems
- Secret templates and generation
- Bulk operations on secrets
- Secret validation and schema checking
