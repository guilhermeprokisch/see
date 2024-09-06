use inkjet::formatter::{Style, Theme};

pub fn create_theme() -> Theme {
    let mut theme = Theme::new(Style {
        primary_color: "#FFFFFF".to_string(),
        secondary_color: "#1E1E1E".to_string(),
        modifiers: Default::default(),
    });

    // Basic styles
    theme.add_style(
        "attribute",
        Style {
            primary_color: "#D7BA7D".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "type",
        Style {
            primary_color: "#4EC9B0".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "type.builtin",
        Style {
            primary_color: "#569CD6".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "type.enum",
        Style {
            primary_color: "#4EC9B0".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "type.enum.variant",
        Style {
            primary_color: "#4FC1FF".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "constructor",
        Style {
            primary_color: "#4EC9B0".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "constant",
        Style {
            primary_color: "#569CD6".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "constant.builtin",
        Style {
            primary_color: "#569CD6".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "constant.builtin.boolean",
        Style {
            primary_color: "#569CD6".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "constant.character",
        Style {
            primary_color: "#CE9178".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "constant.character.escape",
        Style {
            primary_color: "#D7BA7D".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "constant.numeric",
        Style {
            primary_color: "#B5CEA8".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "constant.numeric.integer",
        Style {
            primary_color: "#B5CEA8".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "constant.numeric.float",
        Style {
            primary_color: "#B5CEA8".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "string",
        Style {
            primary_color: "#CE9178".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "string.regexp",
        Style {
            primary_color: "#D16969".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "string.special",
        Style {
            primary_color: "#D7BA7D".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "string.special.path",
        Style {
            primary_color: "#D7BA7D".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "string.special.url",
        Style {
            primary_color: "#4EC9B0".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "string.special.symbol",
        Style {
            primary_color: "#D7BA7D".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "escape",
        Style {
            primary_color: "#D7BA7D".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "comment",
        Style {
            primary_color: "#6A9955".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "comment.line",
        Style {
            primary_color: "#6A9955".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "comment.block",
        Style {
            primary_color: "#6A9955".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "comment.block.documentation",
        Style {
            primary_color: "#6A9955".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "variable",
        Style {
            primary_color: "#9CDCFE".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "variable.builtin",
        Style {
            primary_color: "#9CDCFE".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "variable.parameter",
        Style {
            primary_color: "#9CDCFE".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "variable.other",
        Style {
            primary_color: "#9CDCFE".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "variable.other.member",
        Style {
            primary_color: "#9CDCFE".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "label",
        Style {
            primary_color: "#C8C8C8".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "punctuation",
        Style {
            primary_color: "#D4D4D4".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "punctuation.delimiter",
        Style {
            primary_color: "#D4D4D4".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "punctuation.bracket",
        Style {
            primary_color: "#D4D4D4".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "punctuation.special",
        Style {
            primary_color: "#D4D4D4".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "keyword",
        Style {
            primary_color: "#569CD6".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "keyword.control",
        Style {
            primary_color: "#C586C0".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "keyword.control.conditional",
        Style {
            primary_color: "#C586C0".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "keyword.control.repeat",
        Style {
            primary_color: "#C586C0".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "keyword.control.import",
        Style {
            primary_color: "#C586C0".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "keyword.control.return",
        Style {
            primary_color: "#C586C0".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "keyword.control.exception",
        Style {
            primary_color: "#C586C0".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "keyword.operator",
        Style {
            primary_color: "#D4D4D4".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "keyword.directive",
        Style {
            primary_color: "#C586C0".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "keyword.function",
        Style {
            primary_color: "#569CD6".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "keyword.storage",
        Style {
            primary_color: "#569CD6".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "keyword.storage.type",
        Style {
            primary_color: "#569CD6".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "keyword.storage.modifier",
        Style {
            primary_color: "#569CD6".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "operator",
        Style {
            primary_color: "#D4D4D4".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "function",
        Style {
            primary_color: "#DCDCAA".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "function.builtin",
        Style {
            primary_color: "#DCDCAA".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "function.method",
        Style {
            primary_color: "#DCDCAA".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "function.macro",
        Style {
            primary_color: "#DCDCAA".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "function.special",
        Style {
            primary_color: "#DCDCAA".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "tag",
        Style {
            primary_color: "#569CD6".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "tag.builtin",
        Style {
            primary_color: "#569CD6".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "namespace",
        Style {
            primary_color: "#4EC9B0".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "special",
        Style {
            primary_color: "#D7BA7D".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );

    // Markup styles
    theme.add_style(
        "markup",
        Style {
            primary_color: "#D4D4D4".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.heading",
        Style {
            primary_color: "#569CD6".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.heading.marker",
        Style {
            primary_color: "#569CD6".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.heading.1",
        Style {
            primary_color: "#569CD6".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.heading.2",
        Style {
            primary_color: "#569CD6".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.heading.3",
        Style {
            primary_color: "#569CD6".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.heading.4",
        Style {
            primary_color: "#569CD6".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.heading.5",
        Style {
            primary_color: "#569CD6".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.heading.6",
        Style {
            primary_color: "#569CD6".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.list",
        Style {
            primary_color: "#D4D4D4".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.list.unnumbered",
        Style {
            primary_color: "#D4D4D4".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.list.numbered",
        Style {
            primary_color: "#D4D4D4".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.list.checked",
        Style {
            primary_color: "#D4D4D4".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.list.unchecked",
        Style {
            primary_color: "#D4D4D4".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.bold",
        Style {
            primary_color: "#D4D4D4".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.italic",
        Style {
            primary_color: "#D4D4D4".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.strikethrough",
        Style {
            primary_color: "#D4D4D4".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.link",
        Style {
            primary_color: "#4EC9B0".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.link.url",
        Style {
            primary_color: "#4EC9B0".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.link.label",
        Style {
            primary_color: "#4EC9B0".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.link.text",
        Style {
            primary_color: "#4EC9B0".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.quote",
        Style {
            primary_color: "#D4D4D4".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.raw",
        Style {
            primary_color: "#CE9178".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.raw.inline",
        Style {
            primary_color: "#CE9178".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "markup.raw.block",
        Style {
            primary_color: "#CE9178".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );

    // Diff styles
    theme.add_style(
        "diff",
        Style {
            primary_color: "#D4D4D4".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "diff.plus",
        Style {
            primary_color: "#6A9955".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "diff.minus",
        Style {
            primary_color: "#F44747".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "diff.delta",
        Style {
            primary_color: "#4EC9B0".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );
    theme.add_style(
        "diff.delta.moved",
        Style {
            primary_color: "#4EC9B0".to_string(),
            secondary_color: "".to_string(),
            modifiers: Default::default(),
        },
    );

    theme
}
