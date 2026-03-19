#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn qa_evaluator_new() {
        let config = crate::config::Config::default();
        let evaluator = QaEvaluator::new(&config);
        let report = evaluator.evaluate("test", "output", &[]);
        assert!(!report.requires_override);
        assert_eq!(report.hard_flags.len(), 0);
        assert_eq!(report.soft_flags.len(), 0);
    }

    #[test]
    fn qa_report_default_score() {
        let config = crate::config::Config::default();
        let evaluator = QaEvaluator::new(&config);
        let report = evaluator.evaluate("test", "output", &[]);
        assert_eq!(report.score.overall, 100.0);
    }
}
