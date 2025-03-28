# AI Commit Message Generator (aic)

A CLI tool that uses AI to generate meaningful commit messages by analyzing your staged Git changes.

## Features

- **AI-Powered:** Automatically generates detailed and context-aware commit messages
- **Interactive Mode:** Review and edit generated messages before committing
- **Multiple AI Providers:** Works with OpenAI and compatible APIs
- **Customizable:** Configure prompts, models, and API endpoints

## Quick Start

1. Install the tool:

```bash
cargo install --git https://github.com/yourusername/aic.git
```

2. Configure your API token:

```bash
aic config set api_token your_openai_token
```

3. Stage your changes and generate a commit:

```bash
git add .
aic
```

## Common Use Cases

### Basic Commit Generation

```bash
# Stage changes and generate commit
git add .
aic

# Generate and automatically execute commit
aic -e
```

### Customizing Generation

```bash
# Use a custom prompt
aic --prompt "Write commits in conventional commit format"

# Use a specific model
aic --model gpt-4-turbo

# Use a different API provider
aic --api-base "https://api.deepseek.com"
```

### Configuration Management

```bash
# Quick setup
aic config setup --api-token <TOKEN> --model gpt-4-turbo

# View current settings
aic config list

# Update individual settings
aic config set model gpt-4-turbo
aic config set default_prompt "Write detailed commit messages"
```

## Command Reference

### Main Commands

- `aic`: Generate commit message (default)
- `aic generate`: Same as above, with more options
- `aic config`: Manage configuration

### Common Options

- `-e, --execute`: Execute commit automatically
- `-p, --prompt`: Custom system prompt
- `--model`: Specify AI model
- `--api-base`: Custom API endpoint

### Configuration Keys

- `api_token`: Your API authentication token
- `api_base_url`: API endpoint (default: OpenAI)
- `model`: AI model to use (default: gpt-3.5-turbo)
- `default_prompt`: Default system prompt

## Environment Variables

- `EDITOR`: Preferred editor for modifying commit messages
  - Falls back to: vim → vi → nano

## Tips & Tricks

1. **Quick Commits**: Use `aic -e` to skip the confirmation prompt

2. **Custom Prompts**: Set project-specific prompts:
   ```bash
   aic config set default_prompt "Write commits focusing on security implications"
   ```

3. **Alternative Providers**: Use with other OpenAI-compatible APIs:
   ```bash
   aic config set api_base_url "https://your-api-endpoint"
   ```

## Troubleshooting

### Common Issues

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

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
