# AI Commit Message Generator

A CLI tool that uses AI to generate meaningful commit messages based on your staged Git changes.

## Features

- **AI-Powered:** Automatically generates detailed and context-aware commit messages by analyzing your staged git diffs.
- **Customizable:** Configure system prompts, API base URLs, and models to tailor the commit message style to your project's needs.
- **Interactive Execution:** Review, modify, and execute generated commit commands with ease.
- **Configuration Management:** Easily set, update, or list configuration options such as API tokens and default prompts.

## Installation

To build and install the tool, ensure you have [Rust](https://www.rust-lang.org/tools/install) installed, then follow these steps:

```bash
git clone https://github.com/yourusername/aic.git
cd aic
cargo build --release
```

For global installation (optional):

```bash
cargo install --path .
```

## Usage

### Basic Usage

Generate a commit message using the current staged changes:

```bash
aic
```

If no subcommand is provided, the CLI defaults to generating a commit message.

### Command Options

#### Generate Command

Generate a commit message based on your staged changes. Customize the prompt, API base, and model if desired:

```bash
aic generate --prompt "Write commit messages in conventional commit format" --api-base "https://api.openai.com" --model "gpt-4-turbo"
```

To automatically execute the commit command after generating the message, use the `--execute` (or `-e`) flag:

```bash
aic generate --execute
```

#### Configuration Commands

Manage configuration settings for your project.

- **Get a configuration value:**

  ```bash
  aic config get <key>
  ```

  Example:

  ```bash
  aic config get api_token
  ```

- **Set a configuration value:**

  ```bash
  aic config set <key> <value>
  ```

  Example:

  ```bash
  aic config set model gpt-4-turbo
  ```

- **Setup multiple configurations at once:**

  ```bash
  aic config setup --api-token <TOKEN> --api-base-url <URL> --model <MODEL> --default-prompt "Your default prompt"
  ```

- **List all configuration values:**

  ```bash
  aic config list
  ```

## Workflow

1. **Stage Your Changes:**
   Use `git add` to stage changes in your repository.

2. **Generate a Commit Message:**
   Run `aic` (or `aic generate`) to have the tool analyze your staged changes and produce a commit message using AI.

3. **Review and Execute Command:**
   The generated commit command is displayed. You can either:
   - Confirm execution (using the `--execute` flag or during interactive prompt).
   - Modify the commit message using the provided editor (configured via the `EDITOR` environment variable).

## Editing Commit Message

If you choose to modify the generated commit message, the tool opens your preferred text editor (or falls back to `vim`, `vi`, or `nano`) to let you make changes before committing.

## Configuration File

The tool stores its configuration in a file that includes settings such as:

- **api_token:** API token for authentication with your AI provider.
- **api_base_url:** Custom base URL for the OpenAI-compatible API.
- **model:** The model to generate commit messages.
- **default_prompt:** A default prompt guiding the style of generated commit messages.

The configuration file location is displayed when running `aic config list`.

## Contributing

Contributions are greatly appreciated! If you have ideas for improvements or bug fixes, please fork the repository and create a pull request with your changes.

## License

This project is licensed under the MIT License.
