smd: simple Markdown renderer

## Usage

```bash
smd [OPTIONS] [FILE]
```

If FILE is not provided, smd reads from standard input.

## Options

|               |                                      |
| ------------- | ------------------------------------ |
| `--debug`     | Enable debug mode for verbose output |
| `--no-images` | Disable image rendering              |
| `--help`      | Display this help information        |
| `--version`   | Display version information          |

## Examples

Render a Markdown file:
`smd path/to/your/markdown_file.md`

Render from standard input:
`echo "# Hello, world!" | smd`
