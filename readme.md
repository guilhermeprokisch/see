# smd (Simple Markdown Viewer)

**DISCLAIMER: This project is currently in alpha stage. It may contain bugs, incomplete features, or undergo significant changes. Use with caution and please report any issues you encounter.**

smd is a command-line tool that renders Markdown files with rich formatting in your terminal. It provides a visually appealing way to view Markdown content directly in your console, complete with syntax highlighting, emojis, and image support.

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

## Installation

To install smd, you need to have Rust and Cargo installed on your system. If you don't have them, you can install them from [https://rustup.rs/](https://rustup.rs/).

Once you have Rust set up, clone this repository and build the project:

```
git clone https://github.com/yourusername/smd.git
cd smd
cargo build --release
```

The compiled binary will be available in `target/release/smd`.

## Usage

To use smd, simply pass the path to a Markdown file as an argument:

```
./smd path/to/your/markdown_file.md
```

## Dependencies

smd relies on several Rust crates to provide its functionality:

- `markdown`: For parsing Markdown
- `syntect`: For syntax highlighting
- `termcolor`: For terminal color output
- `viuer`: For image rendering in the terminal
- `emojis`: For emoji support
- `reqwest`: For downloading images from URLs
- `serde_json`: For JSON parsing
- `sha2`: For generating file hashes
- And more (see Cargo.toml for a full list)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. As this project is in alpha, your input and contributions can significantly shape its development.

## Known Issues

As this is an alpha version, you may encounter bugs or incomplete features. Some known limitations include:

[List any known issues or limitations here]

## License

[Insert your chosen license here]

## Acknowledgements

This project was inspired by the need for a simple, yet feature-rich Markdown viewer for the terminal.
