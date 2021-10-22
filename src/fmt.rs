/// Code snippet formatting error.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum FormatError {
    /// An `io::Error` occurred.
    ///
    /// This usually occurs when the stdio redirection fails.
    Io,
    /// Converting `stdout` to `str` failed.
    StrConvertFailed,
    /// `rustfmt` failed in formatting.
    RustfmtFailure,
}

/// Format a code snippet using an external `rustfmt` call.
///
/// # Example
/// ```rust
/// let src = "fn a_b(  s: & str) -> String {   String::new(  )  }";
/// let fmtd = papyrus::fmt::format(src).unwrap();
/// assert_eq!(&fmtd, r#"fn a_b(s: &str) -> String {
///     String::new()
/// }"#);
/// ```
pub fn format(code_snippet: &str) -> Result<String, FormatError> {
    use std::{io::Write, process::*};

    let (success, outputbuf) = {
        let mut child = Command::new("rustfmt")
            .args(&["--config", "newline_style=Unix"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| FormatError::RustfmtFailure)?;

        let stdin = child.stdin.as_mut().expect("stdin has been set");
        write!(stdin, "fn __fmt_wrapper() {{ {} }}", code_snippet)
            .map_err(|_| FormatError::RustfmtFailure)?;

        let output = child
            .wait_with_output()
            .map_err(|_| FormatError::RustfmtFailure)?;

        (output.status.success(), output.stdout)
    };

    if success && !outputbuf.is_empty() {
        let s = std::str::from_utf8(&outputbuf).map_err(|_| FormatError::StrConvertFailed)?;

        //Protect against a .rustfmt.toml with "hard_tabs = true" in it. 
        let s = s.replace("\t", "    ");
        
        let trimmed = s.trim();
        let end = trimmed.len().saturating_sub(2); // \n}

        // the output of rustfmt can change...
        // at the moment it is
        // fn __fmt_wrapper() {\n
        // 0....................^ 21 chars long
        Ok(reduce_indent(&trimmed[21..end]))
    } else {
        Err(FormatError::RustfmtFailure)
    }
}

fn reduce_indent(s: &str) -> String {
    #[derive(Copy, Clone, PartialEq, Eq)]
    enum LitType {
        Literal,
        Str,
        None,
    }
    use LitType::*;
    const LITERALS: [(&str, &str); 6] = [
        (r##"r#""##, r##""#"##),
        (r###"r##""###, r###""#"###),
        (r####"r###""####, r####""###"####),
        (r#####"r####""#####, r#####""####"#####),
        (r######"r#####""######, r######""#####"######),
        (r#######"r######""#######, r#######""######"#######),
    ];
    let mut literal = None;
    let mut idx = 0;
    let mut reduced = String::with_capacity(s.len());
    for line in s.lines() {
        match literal {
            Str => {
                reduced.push_str(line);
                if odd_quotations(line) {
                    literal = None;
                }
            }
            Literal => {
                reduced.push_str(line);
                if line.contains(LITERALS[idx].1) {
                    literal = None;
                }
            }
            None => {
                reduced.push_str(&line[4..]);
                for (i, l) in LITERALS.iter().enumerate() {
                    if line.contains(l.0) && !line.contains(l.1) {
                        idx = i;
                        literal = Literal;
                        break;
                    }
                }
                if literal == None && odd_quotations(line) {
                    literal = Str;
                }
            }
        }
        reduced.push('\n');
    }
    reduced.pop();
    reduced
}

/// Counts quotations and returns if odd or not, indicating if there is unmatched string.
/// Ignores escaped quotes. (so string contains sequence `\"`).
fn odd_quotations(s: &str) -> bool {
    let mut odd = false;
    let mut escaped = false;
    for ch in s.chars() {
        match ch {
            '\\' => escaped = true,
            '\"' if !escaped => odd = !odd,
            _ if escaped => escaped = false,
            _ => (),
        }
    }
    odd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_expr() {
        let snippet = "a+b";
        let s = format(snippet);
        let ans = s.as_ref().map(|x| x.as_str());
        assert_eq!(ans, Ok("a + b"));
    }

    #[test]
    fn test_format_stmt() {
        let snippet = "println! ( \"  \", aaaa ) ;";
        let s = format(snippet);
        let ans = s.as_ref().map(|x| x.as_str());
        assert_eq!(ans, Ok("println!(\"  \", aaaa);"));
    }

    #[test]
    fn test_format_func() {
        let snippet = "fn fmt(){ let a = 1  ; a + b  } ";
        let s = format(snippet);
        let ans = s.as_ref().map(|x| x.as_str());
        assert_eq!(
            ans,
            Ok(r#"fn fmt() {
    let a = 1;
    a + b
}"#)
        );
    }

    #[test]
    fn test_format_err() {
        let snippet = "fn fmt(){ let a = 1  ; a + b   ";
        let s = format(snippet);
        let ans = s.as_ref().map(|x| x.as_str());
        assert_eq!(ans, Err(&FormatError::RustfmtFailure));
    }

    #[test]
    fn test_odd_quotations() {
        assert_eq!(odd_quotations(""), false);
        assert_eq!(odd_quotations("no quoates"), false);
        assert_eq!(odd_quotations(r#""this is a matched string""#), false);
        assert_eq!(odd_quotations(r#""This has \"String\" string""#), false);
        assert_eq!(odd_quotations(r#""This is missing closing quote"#), true);
        assert_eq!(odd_quotations(r#""This has \"escaped\" and missing"#), true);
        assert_eq!(odd_quotations(r#""one "two "three"#), true);
        assert_eq!(odd_quotations(r#""one", "two""#), false);
    }

    #[test]
    fn test_multiline_literal_str() {
        let s = r##"   let s =  r#"Hello
    World
    What
        Up"#;  "##;
        let fmtd = format(s);
        let ans = fmtd.as_ref().map(|x| x.as_str());
        assert_eq!(
            ans,
            Ok(r##"let s = r#"Hello
    World
    What
        Up"#;"##)
        );

        let s = r#""Hello
World
    This
        Indent""#;
        let fmtd = format(s);
        let ans = fmtd.as_ref().map(|x| x.as_str());
        assert_eq!(ans, Ok(s));
    }
}
