/// Code snippet formatting error.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum FormatError {
    /// `rustfmt` command was not found.
    ///
    /// `rustfmt` needs to be installed.
    NoRustfmtCommand,
    /// An `io::Error` occurred.
    Io,
    /// Converting `stdout` to `str` failed.
    StrConvertFailed,
    /// `rustfmt` failed in formatting.
    RustfmtFailure,
}

/// Format a code snippet.
///
/// Removes newlines.
pub fn format(code_snippet: &str) -> Result<String, FormatError> {
    use rustfmt_nightly::*;

    let mut config = Config::default();
    config.set().emit_mode(EmitMode::Stdout);

    let mut buf = Vec::new();

    let success = {
        let code = format!("fn __fmt_wrapper() {{ {} }}", code_snippet);

        let mut session = Session::new(config, Some(&mut buf));

        let _shhout = shh::stdout().map_err(|_| FormatError::Io);
        session.format(Input::Text(code)).is_ok()
    };

    if success && !buf.is_empty() {
        let s = std::str::from_utf8(&buf).map_err(|_| FormatError::StrConvertFailed)?;

        let trimmed = s.trim();

        let end = trimmed.len().saturating_sub(1);

        let inner = &trimmed[20..end];

        let mut cleaned = String::with_capacity(inner.len());

        for line in inner.lines() {
            if !line.is_empty() {
                cleaned.push_str(line.trim());
                cleaned.push(' ');
            }
        }

        cleaned.pop();

        Ok(cleaned)
    } else {
        Err(FormatError::RustfmtFailure)
    }
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
        assert_eq!(ans, Ok("fn fmt() { let a = 1; a + b }"));
    }

    #[test]
    fn test_format_err() {
        let snippet = "fn fmt(){ let a = 1  ; a + b   ";
        let s = format(snippet);
        let ans = s.as_ref().map(|x| x.as_str());
        assert_eq!(ans, Ok(FormatError::RustfmtFailure));
    }
}
