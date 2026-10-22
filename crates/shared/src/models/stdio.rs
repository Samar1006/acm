//! Judging for problems that read standard input and write standard output.

/// The largest absolute or relative error accepted for a decimal answer.
pub const TOLERANCE: f64 = 1e-6;

/// Whether `actual` is an accepted answer for `expected`, as contest judges
/// compare them: whitespace only separates tokens, and every token must match.
/// A decimal expected token such as `2.075` also accepts any number within
/// [`TOLERANCE`]. Other tokens, integers included, must match exactly.
pub fn outputs_match(expected: &str, actual: &str) -> bool {
    let mut expected = expected.split_whitespace();
    let mut actual = actual.split_whitespace();
    loop {
        match (expected.next(), actual.next()) {
            (None, None) => return true,
            (Some(want), Some(got)) if tokens_match(want, got) => {}
            _ => return false,
        }
    }
}

fn tokens_match(expected: &str, actual: &str) -> bool {
    if expected == actual {
        return true;
    }
    if !is_decimal(expected) {
        return false;
    }
    match (expected.parse::<f64>(), actual.parse::<f64>()) {
        (Ok(want), Ok(got)) if want.is_finite() && got.is_finite() => {
            (want - got).abs() <= TOLERANCE * want.abs().max(1.0)
        }
        _ => false,
    }
}

/// A number written with a fractional part or an exponent, so not an exact integer.
fn is_decimal(token: &str) -> bool {
    token.contains(['.', 'e', 'E']) && token.parse::<f64>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace_only_separates_tokens() {
        assert!(outputs_match("YES\nNO\n", "YES NO"));
        assert!(outputs_match("5 4\n", "  5\t4  \r\n\n"));
        assert!(outputs_match("", "\n"));
        assert!(!outputs_match("YES\nNO", "YES"));
        assert!(!outputs_match("YES", "YES NO"));
        assert!(!outputs_match("YES", "yes"));
    }

    #[test]
    fn decimal_answers_accept_small_errors() {
        assert!(outputs_match("1.6666666666666667", "1.666667"));
        assert!(outputs_match("2.075", "2.0750000001"));
        assert!(outputs_match("0.67", "6.7e-1"));
        assert!(outputs_match("1234567.5", "1234567.9"));
        assert!(!outputs_match("2.075", "2.08"));
        assert!(!outputs_match("0.5", "nan"));
        assert!(!outputs_match("0.5", "abc"));
    }

    #[test]
    fn integer_answers_must_match_exactly() {
        assert!(outputs_match("1000000000", "1000000000"));
        assert!(!outputs_match("1000000000", "1000000000.0"));
        assert!(!outputs_match("-1", "-1.0000001"));
        assert!(!outputs_match("5", "05"));
    }
}
