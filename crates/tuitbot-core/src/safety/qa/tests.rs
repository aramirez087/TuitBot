//! QA module tests.

#[cfg(test)]
mod tests {
    use super::super::types::QaReport;

    #[test]
    fn qa_report_default() {
        let report = QaReport::default();
        assert!(report.hard_flags.is_empty());
        assert!(report.soft_flags.is_empty());
        assert!(!report.requires_override);
    }
}
