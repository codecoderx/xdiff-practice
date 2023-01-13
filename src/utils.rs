use std::fmt;
use anyhow::Result;
use std::fmt::Write;
use console::{style, Style};
use similar::{ChangeTag, TextDiff};
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

struct Line(Option<usize>);

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}

pub fn diff_text(text1: String, text2: String)-> Result<String> {
    let mut output = String::new();
    let diff = TextDiff::from_lines(&text1, &text2);
    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            writeln!(&mut output,"{:-^1$}", "-", 80)?;
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, s) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().red()),
                    ChangeTag::Insert => ("+", Style::new().green()),
                    ChangeTag::Equal => (" ", Style::new().dim()),
                };
                write!(&mut output,
                    "{}{} |{}",
                    style(Line(change.old_index())).dim(),
                    style(Line(change.new_index())).dim(),
                    s.apply_to(sign).bold(),
                )?;
                for (emphasized, value) in change.iter_strings_lossy() {
                    if emphasized {
                        write!(&mut output, "{}", s.apply_to(value).underlined().on_black())?;
                    } else {
                        write!(&mut output,"{}", s.apply_to(value))?;
                    }
                }
                if change.missing_newline() {
                    write!(&mut output, "")?;
                }
            }
        }
    }

    Ok(output.to_string())
}


pub fn highlight_text(text: &str ,extension: &str) -> Result<String> {
    let mut output = String::new();
    // Load these once at the start of your program
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ps.find_syntax_by_extension(extension).unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    for line in LinesWithEndings::from(text) {
        let ranges: Vec<(syntect::highlighting::Style, &str)> = h.highlight_line(line, &ps)?;
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        write!(&mut output, "{}", escaped)?;
    }

    Ok(output.to_string())
}