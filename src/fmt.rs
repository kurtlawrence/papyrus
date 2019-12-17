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

/// Format a code snippet.
///
/// Removes newlines from formatted code.
///
/// # Example
/// ```rust
/// let src = "fn a_b(  s: & str) -> String {   String::new(  )  }";
/// let fmtd = papyrus::fmt::format(src).unwrap();
/// assert_eq!(&fmtd, "fn a_b(s: &str) -> String { String::new() }");
/// ```
pub fn format(code_snippet: &str) -> Result<String, FormatError> {
    use std::{io::Write, process::*};

    let (success, outputbuf) = {
        let mut child = Command::new("rustfmt")
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

        let trimmed = s.trim();
        let end = trimmed.len().saturating_sub(1);

        // the output of rustfmt can change...
        // at the moment it is
        // fn __fmt_wrapper() {
        // 0..................^ 20 chars long
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
        assert_eq!(ans, Err(&FormatError::RustfmtFailure));
    }
}
