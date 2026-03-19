//! QA module tests.

#[cfg(test)]
mod tests {
    use super::super::helpers::{detect_language, extract_domain, extract_urls};
    use super::super::types::QaReport;

    #[test]
    fn detect_language_english() {
        let result = detect_language("The quick brown fox");
        assert!(result.is_some());
        let lang = result.unwrap();
        assert_eq!(lang.code, "en");
    }

    #[test]
    fn detect_language_spanish() {
        let result = detect_language("Hola gracias el mundo");
        assert!(result.is_some());
        let lang = result.unwrap();
        assert_eq!(lang.code, "es");
    }

    #[test]
    fn extract_urls_finds_https() {
        let urls = extract_urls("Check https://example.com for info");
        assert_eq!(urls.len(), 1);
        assert!(urls[0].contains("example.com"));
    }

    #[test]
    fn extract_domain_basic() {
        let domain = extract_domain("https://example.com/path");
        assert_eq!(domain, Some("example.com".to_string()));
    }

    #[test]
    fn qa_report_default() {
        let report = QaReport::default();
        assert!(report.hard_flags.is_empty());
        assert!(report.soft_flags.is_empty());
        assert!(!report.requires_override);
    }
}
