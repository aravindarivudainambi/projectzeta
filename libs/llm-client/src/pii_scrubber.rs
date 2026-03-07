use regex::Regex;

/// Scrubs or masks sensitive input before it is sent to an external model provider.
///
/// Applies deterministic regex passes that replace:
/// - Email addresses (RFC-5321-ish local@domain pattern)
/// - US phone numbers (common 10-digit formats with optional country code)
/// - 9-digit SSN patterns (with or without dashes)
///
/// All matches are replaced with the literal string `[REDACTED]`.
pub fn scrub_pii(input: &str) -> String {
    // Order matters: run SSN before email so that patterns embedded next to `@` are caught.
    let ssn_re = Regex::new(r"\b\d{3}-\d{2}-\d{4}\b|\b\d{9}\b").unwrap();
    let email_re = Regex::new(r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}").unwrap();
    let phone_re = Regex::new(
        r"(\+?1[\s\-.]?)?\(?\d{3}\)?[\s\-.]?\d{3}[\s\-.]?\d{4}\b",
    )
    .unwrap();

    let out = ssn_re.replace_all(input, "[REDACTED]");
    let out = email_re.replace_all(&out, "[REDACTED]");
    let out = phone_re.replace_all(&out, "[REDACTED]");

    out.into_owned()
}

#[cfg(test)]
mod tests {
    use super::scrub_pii;

    #[test]
    fn redacts_email_and_ssn() {
        let input = "Send to alice@example.com, SSN is 123-45-6789, phone 555-867-5309";
        let output = scrub_pii(input);

        assert!(
            !output.contains("alice@example.com"),
            "Email should be redacted"
        );
        assert!(
            !output.contains("123-45-6789"),
            "SSN should be redacted"
        );
        assert!(
            !output.contains("555-867-5309"),
            "Phone should be redacted"
        );

        assert_eq!(
            output.matches("[REDACTED]").count(),
            3,
            "Should contain three [REDACTED] tokens"
        );
    }

    #[test]
    fn redacts_nine_digit_ssn_without_dashes() {
        let input = "SSN 123456789 here";
        let output = scrub_pii(input);
        assert!(!output.contains("123456789"));
        assert!(output.contains("[REDACTED]"));
    }

    #[test]
    fn leaves_clean_input_unchanged() {
        let input = "Hello world, no PII here.";
        assert_eq!(scrub_pii(input), input);
    }
}
