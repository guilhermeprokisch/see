# see - See a cute cat.

> cat you see!? Its a fancy cat! :cat:

<img width="1363" alt="image" src="https://github.com/user-attachments/assets/b4c3c3ef-8ba5-4ab5-8a3d-55556ca02536">
<img width="1363" alt="image" src="https://github.com/user-attachments/assets/c5e31ed4-8781-4aca-99a2-3fbe9dfe4add">
<img width="1363" alt="image" src="https://github.com/user-attachments/assets/bd253204-deca-4756-bbba-60272e0a367c">



> [!WARNING]  
> **DISCLAIMER: This project is currently in alpha stage. It may contain bugs, incomplete features, or undergo significant changes. Use with caution and please report any issues you encounter.**

see is a powerful file visualization tool for the terminal, offering advanced code viewing capabilities, Markdown rendering, and more. It provides syntax highlighting, emoji support, and image rendering capabilities, offering a visually appealing way to view various file types directly in your console.

## Features

- State-of-the-art code viewing capabilities with superior syntax highlighting for a wide range of programming languages, powered by tree-sitter
- More accurate, context-aware syntax highlighting
- Minimalistic rich Markdown rendering in the terminal
- Emoji support :smile:
- Image rendering (when possible)
- Clickable links (in supported terminals)
- Table formatting
- Blockquote styling
- And more coming soon!

# Motivation and Context

The primary goal of see is to create a unified tool for viewing both CLI documentation in Markdown and code files, renderable in both the terminal and web browse

As the project evolved from its initial focus on Markdown, support for viewing code files was added, expanding its utility in diverse development ecosystems. Now, see is your go-to tool for seeing everything that a cat can see!

## Markdown Capabilities

While see has expanded its focus beyond just Markdown, it still offers robust Markdown rendering capabilities:

- Rich text formatting (bold, italic, strikethrough)
- Headers and lists
- Code blocks with syntax highlighting
- Tables
- Blockquotes
- Images (when supported by the terminal)
- Clickable links

## Usage

### 1. Viewing Code Files

see serves as a powerful code viewer for the terminal, providing an efficient way to review code directly in your console with advanced syntax highlighting:

```bash
see path/to/your/code_file.py
see --line-numbers path/to/your/code_file.py  # with line numbers
```

<img width="1344" alt="image" src="https://github.com/user-attachments/assets/6bbd4a67-bf30-46a8-b502-fb29ae651b1d">

### 2. Rendering Markdown Files

To render a Markdown file, simply pass the path to the file as an argument:

```bash
see path/to/your/markdown_file.md
```

### 3. Rendering Markdown from Piped Input

see can also read Markdown content from standard input:

```bash
echo "# Hello, *world*" | see
cat README.md | see  # Render a file's content
curl -sL https://raw.githubusercontent.com/guilhermeprokisch/see/master/README.md | see  # Render a remote Markdown file
```

### 4. Viewing see's Own Documentation

You can view see's documentation directly in your terminal by running:

```bash
see --help
```

## Installation

There are several ways to install see:

### 1. Install prebuilt binaries via shell script (Recommended)

The easiest and fastest way to install see is by using our shell script:

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/guilhermeprokisch/see/releases/download/v0.1.0/see-installer.sh | sh
```

**DISCLAIMER: The version number in the URL above (v0.1.0) may not be the latest version. Please check the [releases page](https://github.com/guilhermeprokisch/see/releases) for the most recent version and update the URL accordingly before running the command.**

### 2. Using prebuilt binaries from GitHub releases

If you prefer to manually download and install the binary:

1. Visit the [see releases page](https://github.com/guilhermeprokisch/see/releases) on GitHub.
2. Find the latest release version.
3. Download the appropriate binary for your operating system and architecture.
4. Extract the downloaded file if necessary.
5. Move the `see` binary to a directory in your system's PATH (e.g., `/usr/local/bin` on Unix-like systems).

### 3. Install prebuilt binaries via Homebrew

If you're using Homebrew, you can install see with:

```sh
brew install guilhermeprokisch/see/see
```

### 4. Using Cargo

You can install see directly from crates.io using Cargo:

```bash
cargo install see
```

This will download, compile, and install the latest version of see. Make sure your Rust installation is up to date.

### 5. Building from Source

If you prefer to build from source or want to contribute to the project:

1. Ensure you have Rust and Cargo installed. If not, get them from [https://rustup.rs/](https://rustup.rs/).

2. Clone the repository:

   ```bash
   git clone https://github.com/guilhermeprokisch/see.git
   cd see
   ```

3. Build and install the project using Cargo:

   ```bash
   cargo install --path .
   ```

This will compile the project and install the `see` binary in your Cargo bin directory, which should be in your PATH.

## Usage

There are two main ways to use see:

### 1. Rendering a Markdown file

To render a Markdown file, simply pass the path to the file as an argument:

```bash
see path/to/your/markdown_file.md
```

### 2. Rendering Markdown from piped input

see can also read Markdown content from standard input, allowing you to pipe content directly into it:

```bash
echo "# Hello, *world*" | see
```

This feature is particularly useful for integrating see with other commands or for quickly rendering Markdown snippets. For example:

```bash
cat README.md | see  # Render a file's content
curl -sL https://raw.githubusercontent.com/guilhermeprokisch/see/master/README.md | see  # Render a remote Markdown file
```

#### Integration with CLI Tools

see can be easily integrated with CLI tools to replace traditional man pages with rich Markdown documentation. Here's an example of how you can use see with a custom CLI tool's --help flag:

```bash
#!/bin/bash

# Name: mycli
# Description: Example CLI tool using see for documentation

if [[ "$1" == "--help" ]]; then
    # Use see to render the Markdown help file
    see ~/.mycli/help.md
else
    # Regular CLI functionality
    echo "Running mycli with arguments: $@"
fi
```

In this example, create a Markdown file at `~/.mycli/help.md` with your CLI documentation. When users run `mycli --help`, they'll see a beautifully rendered version of your Markdown documentation instead of a plain text man page.

<img width="1482" alt="image" src="https://github.com/user-attachments/assets/9b97388e-64d6-4d48-b1a8-614a209f32ee">

This approach allows you to maintain a single source of documentation that's readable in raw form, rendered nicely in the terminal, and viewable in web browsers.

#### Viewing see's Own Documentation

see uses itself to display its own documentation. You can view see's documentation directly in your terminal by running:

```bash
see --help
```

This command will render see's main documentation file `/docs`, giving you a practical example of see in action and providing detailed information about its usage and features.

## Configuration

see supports user-defined configuration files. You can customize various aspects of the rendering process by creating a `config.toml` file in the following location:

- On Linux and macOS: `~/.config/see/config.toml`
- On Windows: `C:\Users\<USERNAME>\AppData\Roaming\see\config.toml`

You can generate a default configuration file by running:

```bash
see --generate-config
```

Here's an example of what you can configure:

```toml
max_image_width = 40
max_image_height = 13
render_images = true
render_links = true
render_table_borders = false
show_line_numbers = true
```

- `max_image_width` and `max_image_height`: Maximum dimensions for rendered images
- `render_images`: If false, images will not be rendered
- `render_links`: If false, links will not be clickable
- `render_table_borders`: If true, tables will be rendered with ASCII borders (default: false)
- `show_line_numbers`: If true, line numbers will be shown for code files (can also be set with `--line-numbers` option)

Note: see uses [tree-sitter](https://github.com/tree-sitter/tree-sitter) thanks to [inkjet](https://github.com/Colonial-Dev/inkjet) for syntax highlighting. Currently, only one theme is implemented, but there are plans to make see compatible with Helix editor themes in the future, which will greatly expand customization options.

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
