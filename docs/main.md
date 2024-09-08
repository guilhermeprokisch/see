# smd: simple Markdown and Code renderer

## Usage

```bash
smd [OPTIONS] [FILE]
```

If FILE is not provided, smd reads from standard input.

## Options

|                          |                                                     |
| ------------------------ | --------------------------------------------------- |
| `--debug`                | Enable debug mode for verbose output                |
| `--help`                 | Display this help information                       |
| `--version`              | Display version information                         |
| `--generate-config`      | Generate a default configuration file               |
| `--max-image-width`      | Set maximum width for rendered images               |
| `--max-image-height`     | Set maximum height for rendered images              |
| `--render-images`        | Enable or disable image rendering                   |
| `--render-links`         | Enable or disable clickable links                   |
| `--render-table-borders` | Enable or disable table borders in rendered output  |
| `--show-line-numbers`    | Show or hide line numbers when rendering code files |
| `--config <file>`        | Specify a custom configuration file                 |

## Examples

Render a Markdown file:

```bash
smd path/to/your/markdown_file.md
```

Render a code file without line numbers:

```bash
smd --show-line-numbers=false path/to/your/code_file.py
```

Render from standard input:

```bash
echo "# Hello, world" | smd
```

Generate a default configuration file:

```bash
smd --generate-config
```

Use a custom configuration file:

```bash
smd --config /path/to/custom/config.toml path/to/your/markdown_file.md
```

Render with maximum image dimensions:

```bash
smd --max-image-width=60 --max-image-height=20 path/to/your/markdown_file.md
```

Disable image rendering:

```bash
smd --render-images=false path/to/your/markdown_file.md
```

Enable table borders:

```bash
smd --render-table-borders=true path/to/your/markdown_file.md
```
