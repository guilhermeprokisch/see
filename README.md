/# smd (Simple Markdown Viewer)

<img width="1482" alt="image" src="https://github.com/user-attachments/assets/9ead893a-e3b2-4b0f-9945-e751ff67d3ef">
<img width="1482" alt="image" src="https://github.com/user-attachments/assets/8411f297-f13f-47b6-99f4-b4579531edcb">

**DISCLAIMER: This project is currently in alpha stage. It may contain bugs, incomplete features, or undergo significant changes. Use with caution and please report any issues you encounter.**

smd is a minimalistic Markdown renderer for the terminal with syntax highlighting, emoji support, and image rendering. It provides a visually appealing way to view Markdown content directly in your console.

## Motivation

The primary goal of smd is to create CLI documentation in Markdown that can be rendered both in the terminal and viewed in a web browser. This dual-format approach aims to:

1. Provide a unified documentation format accessible across different environments
2. Enable quick viewing end editing of cli documentations from anywhere

As the project evolved, support for more complex Markdown features was added. This expansion opens up possibilities for integration with other documentation tools and workflows, potentially enhancing its utility in diverse development ecosystems.

## Features

- Rich text rendering in the terminal
- Syntax highlighting for code blocks
- Emoji support :smile:
- Image rendering (when possible)
- Clickable links (in supported terminals)
- Table formatting
- Task list rendering
- Nested list support
- Blockquote styling
- And more adding soon!

## Installation

There are several ways to install smd:

### 1. Using Cargo (Recommended)

You can install smd directly from crates.io using Cargo:

```bash
cargo install smd
```

This will download, compile, and install the latest version of smd. Make sure your Rust installation is up to date.

### 2. Install prebuilt binaries via shell script

You can quickly install smd using our shell script:

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/guilhermeprokisch/smd/releases/download/v0.2.9/smd-installer.sh | sh
```

**DISCLAIMER: The version number in the URL above (v0.2.9) may not be the latest version. Please check the [releases page](https://github.com/guilhermeprokisch/smd/releases) for the most recent version and update the URL accordingly before running the command.**

### 3. Install prebuilt binaries via Homebrew

If you're using Homebrew, you can install smd with:

```sh
brew install guilhermeprokisch/smd/smd
```

### 4. Building from Source

If you prefer to build from source or want to contribute to the project:

1. Ensure you have Rust and Cargo installed. If not, get them from [https://rustup.rs/](https://rustup.rs/).

2. Clone the repository:

   ```bash
   git clone https://github.com/guilhermeprokisch/smd.git
   cd smd
   ```

3. Build and install the project using Cargo:

   ```bash
   cargo install --path .
   ```

This will compile the project and install the `smd` binary in your Cargo bin directory, which should be in your PATH.

## Usage

There are two main ways to use smd:

### 1. Rendering a Markdown file

To render a Markdown file, simply pass the path to the file as an argument:

```bash
smd path/to/your/markdown_file.md
```

### 2. Rendering Markdown from piped input

smd can also read Markdown content from standard input, allowing you to pipe content directly into it:

```bash
echo "# Hello, *world*" | smd
```

This feature is particularly useful for integrating smd with other commands or for quickly rendering Markdown snippets. For example:

```bash
cat README.md | smd  # Render a file's content
curl -sL https://raw.githubusercontent.com/guilhermeprokisch/smd/master/README.md | smd  # Render a remote Markdown file
```

#### Integration with CLI Tools

smd can be easily integrated with CLI tools to replace traditional man pages with rich Markdown documentation. Here's an example of how you can use smd with a custom CLI tool's --help flag:

```bash
#!/bin/bash

# Name: mycli
# Description: Example CLI tool using smd for documentation

if [[ "$1" == "--help" ]]; then
    # Use smd to render the Markdown help file
    smd ~/.mycli/help.md
else
    # Regular CLI functionality
    echo "Running mycli with arguments: $@"
fi
```

In this example, create a Markdown file at `~/.mycli/help.md` with your CLI documentation. When users run `mycli --help`, they'll see a beautifully rendered version of your Markdown documentation instead of a plain text man page.

<img width="1482" alt="image" src="https://github.com/user-attachments/assets/9b97388e-64d6-4d48-b1a8-614a209f32ee">

This approach allows you to maintain a single source of documentation that's readable in raw form, rendered nicely in the terminal, and viewable in web browsers.

#### Viewing smd's Own Documentation

smd uses itself to display its own documentation. You can view smd's documentation directly in your terminal by running:

```bash
smd --help
```

This command will render smd's main documentation file `/docs`, giving you a practical example of smd in action and providing detailed information about its usage and features.

## Configuration

smd supports user-defined configuration files. You can customize various aspects of the rendering process by creating a `config.toml` file in the following location:

- On Linux and macOS: `~/.config/smd/config.toml`
- On Windows: `C:\Users\<USERNAME>\AppData\Roaming\smd\config.toml`

You can generate a default configuration file by running:

```bash
smd --generate-config
```

Here's an example of what you can configure:

```toml
code_highlight_theme = "Solarized (dark)"
max_image_width = 40
max_image_height = 13
render_images = true
render_links = true
render_table_borders = false
```

- `code_highlight_theme`: Theme for code syntax highlighting (default: "Solarized (dark)")
- `max_image_width` and `max_image_height`: Maximum dimensions for rendered images
- `render_images`: If false, images will not be rendered
- `render_links`: If false, links will not be clickable
- `render_table_borders`: If true, tables will be rendered with ASCII borders (default: false)

### Available Code Highlight Themes

The `code_highlight_theme` option can be set to any of the following values:

- "base16-ocean.dark"
- "base16-eighties.dark"
- "base16-mocha.dark"
- "base16-ocean.light"
- "InspiredGitHub"
- "Solarized (dark)"
- "Solarized (light)"

These are the default themes provided by the syntect library. Choose the theme that best suits your terminal's color scheme and personal preferences.

Note: The actual appearance of these themes may vary slightly depending on your terminal's color settings.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. As this project is in alpha, your input and contributions can significantly shape its development.

## Known Issues

As this is an alpha version, you may encounter bugs or incomplete features. Some known limitations include:

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## TODO:

1. Extended Markdown Support

2. Improve syntax highlighting

3. Theming and Customization: Develop user-customizable color schemes and rendering options
