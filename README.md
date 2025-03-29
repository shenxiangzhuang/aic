# AIC: AI Commit Message Generator

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Crates.io](https://img.shields.io/crates/v/aic)](https://crates.io/crates/aic)
![docs.rs](https://img.shields.io/docsrs/aic)

A CLI tool that uses AI to generate meaningful commit messages by analyzing your staged Git changes.

## Features

- 🤖 **AI-Powered**: Automatically generates detailed and context-aware commit messages
- ✏️ **Interactive Mode**: Review and edit generated messages before committing
- 🔌 **Multiple AI Providers**: Works with OpenAI and compatible APIs
- ⚙️ **Customizable**: Configure prompts, models, and API endpoints

## Installation

```bash
cargo install --git https://github.com/shenxiangzhuang/aic.git
```

## Quick Start

1. Configure your API settings:
```bash
# For OpenAI
aic config setup --api-token your_openai_token --api-base-url https://api.openai.com --model gpt-3.5-turbo

# For DeepSeek
aic config setup --api-token your_deepseek_token --api-base-url https://api.deepseek.com --model deepseek-chat
```

Output:
```
⚙️  Updating configuration...
✓ Set api_token to: your•••••
✓ Set api_base_url to: https://api.openai.com
✓ Set model to: gpt-3.5-turbo
🎉 Configuration updated successfully!
```

2. Verify your configuration:
```bash
aic config list
```

Output:
```
⚙️  Current Configuration:
┌───────────────┬──────────────────────────────────────┐
│ api_token     │ your•••••                            │
│ api_base_url  │ https://api.openai.com               │
│ model         │ gpt-3.5-turbo                        │
│ default_prompt│ Write detailed commit messages...     │
└───────────────┴──────────────────────────────────────┘

📁 Configuration file location:
   /home/user/.config/aic/config.toml
```

3. Test your API connection:
```bash
aic ping
```

Output:
```
🔍 Testing API connection...
🌐 API Base URL: https://api.openai.com
🤖 Model: gpt-3.5-turbo
✅ API connection successful!
✨ Configuration is working correctly.
```

4. Generate commit messages:
```bash
# Stage changes and generate commit message
aic -a

# Generate and commit automatically
aic -ac

# Generate commit message (with staged changes)
aic
```

Example output:
```
╭─────────────────────────────────────╮
│     AI Commit Message Generator     │
╰─────────────────────────────────────╯
📦 Staging all changes...
🔍 Analyzing staged changes...
🤖 Using model: gpt-3.5-turbo
✨ Generating commit message...
📋 Commit command:
git commit -m "feat: add new feature X"

Execute this commit? [Y/m/n]:
```

## Usage

### Basic Commands

```bash
# Generate commit message (requires staged changes)
aic

# Stage all changes and generate commit message
aic -a

# Generate and commit automatically
aic -c

# Stage all changes and commit automatically
aic -ac

# Test API connection
aic ping
```

> **Note**: The `-a` flag will stage ALL changes in your working directory with `git add .`. The `-c` flag will commit directly without confirmation. Use these flags with caution, especially in repositories with multiple changes.

### Configuration Management

```bash
# Quick setup
aic config setup --api-token <TOKEN> --api-base-url https://api.openai.com --model gpt-4-turbo

# View current settings
aic config list

# Get specific setting
aic config get api_token

# Update setting
aic config set model gpt-4-turbo
aic config set default_prompt "Write detailed commit messages"
```

### Configuration Options

- `api_token`: Your API authentication token
- `api_base_url`: API endpoint (default: OpenAI)
- `model`: AI model to use (default: gpt-3.5-turbo)
- `default_prompt`: Default system prompt for commit message generation

### Environment Variables

- `EDITOR`: Preferred editor for modifying commit messages
  - Falls back to: vim → vi → nano

## Examples

### Basic Usage
```bash
# Stage changes and generate commit message
git add .
aic

# Stage and commit automatically
aic -ac
```

### Configuration Examples
```bash
# Set up OpenAI
aic config setup --api-token sk-... --model gpt-4-turbo

# Set up DeepSeek
aic config setup --api-token ds-... --api-base-url https://api.deepseek.com --model deepseek-chat

# Customize commit message style
aic config set default_prompt "Write commits in conventional commit format"
```

## Troubleshooting

1. **No Changes Detected**
   - Ensure changes are staged with `git add`
   - Check if you're in a git repository

2. **API Errors**
   - Verify your API token is set correctly
   - Check API endpoint accessibility
   - Confirm you have sufficient API credits

3. **Editor Issues**
   - Set your preferred editor: `export EDITOR=vim`
   - Ensure the editor is installed and accessible

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## References

- [The Lost Art of Commit Messages](https://www.seyhan.me/blog/post/lost-art-of-commit-messages)
- [Conventional Commits 1.0.0](https://www.conventionalcommits.org/)
