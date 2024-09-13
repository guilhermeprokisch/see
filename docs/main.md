# see: simple Markdown and Code renderer

## Usage

```bash
see [OPTIONS] [FILE]
```

If FILE is not provided, see reads from standard input.

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
| `--show-filename`        | Show or hide the filename before rendering content  |
| `--config <file>`        | Specify a custom configuration file                 |
| `--use-color`            | Control color output                                |

## Examples

Render a Markdown file:

```bash
see path/to/your/markdown_file.md
```

Render a code file without line numbers:

```bash
see --show-line-numbers=false path/to/your/code_file.py
```

Render from standard input:

```bash
echo "# Hello, world" | see
```

Generate a default configuration file:

```bash
see --generate-config
```

Use a custom configuration file:

```bash
see --config /path/to/custom/config.toml path/to/your/markdown_file.md
```

Render with maximum image dimensions:

```bash
see --max-image-width=60 --max-image-height=20 path/to/your/markdown_file.md
```

Disable image rendering:

```bash
see --render-images=false path/to/your/markdown_file.md
```

Enable table borders:

```bash
see --render-table-borders=true path/to/your/markdown_file.md
```

Disable color output when piping to another command:

```bash
see --use-color=false path/to/your/markdown_file.rs | less
```

Render content without showing the filename:

```bash
see --show-filename=false path/to/your/markdown_file.md
```
