# smd: simple Markdown and Code renderer

## Usage

```bash
smd [OPTIONS] [FILE]
```

If FILE is not provided, smd reads from standard input.

## Options

|                     |                                             |
| ------------------- | ------------------------------------------- |
| `--debug`           | Enable debug mode for verbose output        |
| `--help`            | Display this help information               |
| `--version`         | Display version information                 |
| `--generate-config` | Generate a default configuration file       |
| `--line-numbers`    | Show line numbers when rendering code files |

## Configuration

smd uses a configuration file located at:

- Linux/macOS: `~/.config/smd/config.toml`
- Windows: `%APPDATA%\smd\config.toml`

You can generate a default configuration file using the `--generate-config` option.

The configuration file allows you to customize various aspects of the rendering, including:

- Enabling/disabling image rendering
- Setting maximum image dimensions
- Choosing the code highlighting theme
- Enabling/disabling clickable links
- Showing line numbers for code files (can also be set with `--line-numbers` option)

## Examples

Render a Markdown file:

```bash
smd path/to/your/markdown_file.md
```

Render a code file with line numbers:

```bash
smd --line-numbers path/to/your/code_file.py
```

Render from standard input:

```bash
echo "# Hello, world" | smd
```

Generate a default configuration file:

```bash
smd --generate-config
```
