# AIC: AI Commit Message Generator

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

1.Configure your API settings (choose one):

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

2.Verify your configuration:
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

3.Test your API connection:
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

4. Stage your changes and generate a commit:
```bash
git add .
aic
```

Output:
```
╭─────────────────────────────────────╮
│     AI Commit Message Generator     │
╰─────────────────────────────────────╯
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
# Test API connection and configuration
aic ping

# Generate commit message
aic

# Generate and execute commit automatically
aic -e

# Use custom prompt
aic --prompt "Write commits in conventional commit format"

# Use specific model
aic --model gpt-4-turbo

# Use different API provider
aic --api-base "https://api.deepseek.com"
```

### Configuration

```bash
# Quick setup
aic config setup --api-token <TOKEN> --model gpt-4-turbo

# View settings
aic config list

# Update settings
aic config set model gpt-4-turbo
aic config set default_prompt "Write detailed commit messages"
```

## Configuration Options

- `api_token`: Your API authentication token
- `api_base_url`: API endpoint (default: OpenAI)
- `model`: AI model to use (default: gpt-3.5-turbo)
- `default_prompt`: Default system prompt

## Environment Variables

- `EDITOR`: Preferred editor for modifying commit messages
  - Falls back to: vim → vi → nano

## Troubleshooting

1. **No Changes Detected**
   - Ensure changes are staged with `git add`
   - Check if you're in a git repository

2. **API Errors**
   - Verify your API token is set correctly
   - Check API endpoint accessibility
   - Confirm you have sufficient API credits

3. **Editor Issues**
   - Set your preferred editor: `export EDITOR=code`
   - Ensure the editor is installed and accessible

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
