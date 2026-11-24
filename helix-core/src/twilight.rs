use std::ops::Range;

use crate::{syntax::Syntax, RopeSlice};

/// Returns the half-open range of lines that should stay undimmed around `cursor`.
pub fn context_lines(
    text: RopeSlice,
    syntax: Option<&Syntax>,
    cursor: usize,
    expand: &[String],
) -> Range<usize> {
    syntax
        .and_then(|syntax| outermost_expand_node_lines(text, syntax, cursor, expand))
        .unwrap_or_else(|| paragraph_lines(text, text.char_to_line(cursor)))
}

fn outermost_expand_node_lines(
    text: RopeSlice,
    syntax: &Syntax,
    cursor: usize,
    expand: &[String],
) -> Option<Range<usize>> {
    let byte = text.char_to_byte(cursor) as u32;
    let mut node = syntax.descendant_for_byte_range(byte, byte);
    let mut outermost = None;
    while let Some(current) = node {
        if expand.iter().any(|kind| kind == current.kind()) {
            outermost = Some(current.byte_range());
        }
        node = current.parent();
    }

    let bytes = outermost?;
    let last_byte = bytes.end.saturating_sub(1).max(bytes.start) as usize;
    Some(text.byte_to_line(bytes.start as usize)..text.byte_to_line(last_byte) + 1)
}

fn paragraph_lines(text: RopeSlice, line: usize) -> Range<usize> {
    let is_blank = |line: usize| text.line(line).chars().all(char::is_whitespace);
    if is_blank(line) {
        return line..line + 1;
    }

    let mut start = line;
    while start > 0 && !is_blank(start - 1) {
        start -= 1;
    }
    let mut end = line + 1;
    while end < text.len_lines() && !is_blank(end) {
        end += 1;
    }
    start..end
}

#[cfg(test)]
mod test {
    use once_cell::sync::Lazy;

    use super::*;
    use crate::{syntax::Loader, Rope};

    static LOADER: Lazy<Loader> = Lazy::new(crate::config::default_lang_loader);

    const GO_SOURCE: &str = "package main

import \"fmt\"

func outer() {
\tx := 1
\tif x > 0 {
\t\tfmt.Println(x)
\t}
}

var a = 1
var b = 2

func other() {}
";

    #[test]
    fn test_context_lines() {
        let go_expand = vec![
            "function_declaration".to_string(),
            "if_statement".to_string(),
        ];

        struct Case {
            name: &'static str,
            use_syntax: bool,
            expand: Vec<String>,
            cursor_line: usize,
            cursor_col: usize,
            want: Range<usize>,
        }

        let cases = [
            Case {
                name: "treesitter - cursor in function body expands to whole function",
                use_syntax: true,
                expand: go_expand.clone(),
                cursor_line: 5,
                cursor_col: 1,
                want: 4..10,
            },
            Case {
                name: "treesitter - nested if expands to outermost function",
                use_syntax: true,
                expand: go_expand.clone(),
                cursor_line: 7,
                cursor_col: 2,
                want: 4..10,
            },
            Case {
                name: "treesitter - single line function",
                use_syntax: true,
                expand: go_expand.clone(),
                cursor_line: 14,
                cursor_col: 5,
                want: 14..15,
            },
            Case {
                name: "fallback - outside expand nodes uses paragraph",
                use_syntax: true,
                expand: go_expand.clone(),
                cursor_line: 11,
                cursor_col: 0,
                want: 11..13,
            },
            Case {
                name: "fallback - empty expand list uses paragraph",
                use_syntax: true,
                expand: Vec::new(),
                cursor_line: 7,
                cursor_col: 2,
                want: 4..10,
            },
            Case {
                name: "fallback - no syntax uses paragraph",
                use_syntax: false,
                expand: go_expand.clone(),
                cursor_line: 12,
                cursor_col: 0,
                want: 11..13,
            },
            Case {
                name: "fallback - blank line only keeps itself",
                use_syntax: true,
                expand: go_expand.clone(),
                cursor_line: 10,
                cursor_col: 0,
                want: 10..11,
            },
        ];

        let source = Rope::from_str(GO_SOURCE);
        let text = source.slice(..);
        let language = LOADER.language_for_name("go").unwrap();
        let syntax = Syntax::new(text, language, &LOADER).unwrap();

        for case in cases {
            let syntax = case.use_syntax.then_some(&syntax);
            let cursor = text.line_to_char(case.cursor_line) + case.cursor_col;

            let got = context_lines(text, syntax, cursor, &case.expand);

            assert_eq!(got, case.want, "{}", case.name);
        }
    }
}
