# smd (Simple Markdown Viewer)

<img width="1482" alt="image" src="https://github.com/user-attachments/assets/9ead893a-e3b2-4b0f-9945-e751ff67d3ef">
<img width="1482" alt="image" src="https://github.com/user-attachments/assets/8411f297-f13f-47b6-99f4-b4579531edcb">

**DISCLAIMER: This project is currently in alpha stage. It may contain bugs, incomplete features, or undergo significant changes. Use with caution and please report any issues you encounter.**

smd is a minimalistic Markdown renderer for the terminal with syntax highlighting, emoji support, and image rendering. It provides a visually appealing way to view Markdown content directly in your console.

## Motivation

The primary goal of smd is to create CLI documentation in Markdown that can be rendered both in the terminal and viewed in a web browser. This dual-format approach aims to:

1. Provide a unified documentation format accessible across different environments
2. Enable quick viewing of rich documentation directly in the terminal

As the project evolved, support for more complex Markdown features was added. This expansion opens up possibilities for integration with other documentation tools and workflows, potentially enhancing its utility in diverse development ecosystems.

## Features

- Rich text rendering in the terminal
- Syntax highlighting for code blocks
- Emoji support
- Image rendering (when possible)
- Clickable links (in supported terminals)
- Table formatting
- Task list rendering
- Nested list support
- Blockquote styling

## Usage

To use smd, simply pass the path to a Markdown file as an argument:

```bash
smd path/to/your/markdown_file.md
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

## Installation

There are two ways to install smd:

### 1. Using Cargo (Recommended)

The easiest way to install smd is directly from crates.io using Cargo:

```bash
cargo install smd
```

This will download, compile, and install the latest version of smd. Make sure your Rust installation is up to date.

### 2. Building from Source

If you prefer to build from source or want to contribute to the project:

1. Ensure you have Rust and Cargo installed. If not, get them from [https://rustup.rs/](https://rustup.rs/).

2. Clone the repository:

   ```bash
   git clone https://github.com/guilhermeprokisch/smd.git
   cd smd
   ```

3. Build the project:

   ```bash
   cargo build --release
   ```

4. The compiled binary will be available in `target/release/smd`.

5. Optionally, add the binary to your PATH or move it to a directory in your PATH:
   ```bash
   sudo mv target/release/smd /usr/local/bin/
   ```

Now you can use `smd` from anywhere in your terminal.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. As this project is in alpha, your input and contributions can significantly shape its development.

## Known Issues

As this is an alpha version, you may encounter bugs or incomplete features. Some known limitations include:

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Goals

1. Extended Markdown Support: Create plugin system for custom extensions

2. Theming and Customization: Develop user-customizable color schemes

3. Enhanced Image Rendering: Implement ASCII art fallback for text-only terminals

4. Self-Documentation: Create smd's own documentation using smd
