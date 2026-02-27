//! Progressive enrichment stages and profile completeness tracking.

use super::Config;

// ---------------------------------------------------------------------------
// Enrichment stages
// ---------------------------------------------------------------------------

/// An enrichment stage that groups related configuration fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnrichmentStage {
    /// Brand voice, reply style, content style — shapes every LLM output.
    Voice,
    /// Opinions, experiences, content pillars — makes content authentic.
    Persona,
    /// Target accounts, competitor keywords — focuses discovery.
    Targeting,
}

impl EnrichmentStage {
    /// Human-readable label for display.
    pub fn label(self) -> &'static str {
        match self {
            Self::Voice => "Voice",
            Self::Persona => "Persona",
            Self::Targeting => "Targeting",
        }
    }

    /// Short description of what this stage unlocks.
    pub fn description(self) -> &'static str {
        match self {
            Self::Voice => "shapes every LLM-generated reply and tweet",
            Self::Persona => "makes content authentic with opinions and experiences",
            Self::Targeting => "focuses discovery on specific accounts and competitors",
        }
    }

    /// All stages in recommended order.
    pub fn all() -> &'static [EnrichmentStage] {
        &[Self::Voice, Self::Persona, Self::Targeting]
    }
}

impl std::fmt::Display for EnrichmentStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ---------------------------------------------------------------------------
// Profile completeness
// ---------------------------------------------------------------------------

/// Snapshot of profile completeness across all enrichment stages.
pub struct ProfileCompleteness {
    /// Each stage paired with its completion status.
    pub stages: Vec<(EnrichmentStage, bool)>,
}

impl ProfileCompleteness {
    /// Number of completed stages.
    pub fn completed_count(&self) -> usize {
        self.stages.iter().filter(|(_, done)| *done).count()
    }

    /// Total number of stages.
    pub fn total_count(&self) -> usize {
        self.stages.len()
    }

    /// Whether all enrichment stages are complete.
    pub fn is_fully_enriched(&self) -> bool {
        self.stages.iter().all(|(_, done)| *done)
    }

    /// The next incomplete stage, if any.
    pub fn next_incomplete(&self) -> Option<EnrichmentStage> {
        self.stages
            .iter()
            .find(|(_, done)| !*done)
            .map(|(stage, _)| *stage)
    }

    /// One-line summary like "Voice OK  Persona --  Targeting OK".
    pub fn one_line_summary(&self) -> String {
        self.stages
            .iter()
            .map(|(stage, done)| {
                let status = if *done { "OK" } else { "--" };
                format!("{} {}", stage.label(), status)
            })
            .collect::<Vec<_>>()
            .join("  ")
    }
}

// ---------------------------------------------------------------------------
// Config impl
// ---------------------------------------------------------------------------

impl Config {
    /// Compute profile completeness across all enrichment stages.
    pub fn profile_completeness(&self) -> ProfileCompleteness {
        let opt_non_empty =
            |opt: &Option<String>| opt.as_ref().is_some_and(|v| !v.trim().is_empty());

        let voice = opt_non_empty(&self.business.brand_voice)
            || opt_non_empty(&self.business.reply_style)
            || opt_non_empty(&self.business.content_style);

        let persona = !self.business.persona_opinions.is_empty()
            || !self.business.persona_experiences.is_empty()
            || !self.business.content_pillars.is_empty();

        let targeting =
            !self.targets.accounts.is_empty() || !self.business.competitor_keywords.is_empty();

        ProfileCompleteness {
            stages: vec![
                (EnrichmentStage::Voice, voice),
                (EnrichmentStage::Persona, persona),
                (EnrichmentStage::Targeting, targeting),
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_config() -> Config {
        Config::default()
    }

    #[test]
    fn all_empty_has_zero_completeness() {
        let config = empty_config();
        let pc = config.profile_completeness();
        assert_eq!(pc.completed_count(), 0);
        assert_eq!(pc.total_count(), 3);
        assert!(!pc.is_fully_enriched());
    }

    #[test]
    fn voice_stage_complete_when_brand_voice_set() {
        let mut config = empty_config();
        config.business.brand_voice = Some("witty and concise".to_string());
        let pc = config.profile_completeness();
        assert_eq!(pc.completed_count(), 1);
        assert!(pc.stages[0].1); // Voice = true
    }

    #[test]
    fn voice_stage_complete_when_reply_style_set() {
        let mut config = empty_config();
        config.business.reply_style = Some("helpful".to_string());
        let pc = config.profile_completeness();
        assert!(pc.stages[0].1);
    }

    #[test]
    fn voice_stage_complete_when_content_style_set() {
        let mut config = empty_config();
        config.business.content_style = Some("educational".to_string());
        let pc = config.profile_completeness();
        assert!(pc.stages[0].1);
    }

    #[test]
    fn persona_stage_complete_when_opinions_set() {
        let mut config = empty_config();
        config.business.persona_opinions = vec!["types are good".to_string()];
        let pc = config.profile_completeness();
        assert!(pc.stages[1].1); // Persona = true
    }

    #[test]
    fn targeting_stage_complete_when_accounts_set() {
        let mut config = empty_config();
        config.targets.accounts = vec!["elonmusk".to_string()];
        let pc = config.profile_completeness();
        assert!(pc.stages[2].1); // Targeting = true
    }

    #[test]
    fn targeting_stage_complete_when_competitor_keywords_set() {
        let mut config = empty_config();
        config.business.competitor_keywords = vec!["rival_tool".to_string()];
        let pc = config.profile_completeness();
        assert!(pc.stages[2].1);
    }

    #[test]
    fn empty_string_voice_not_counted() {
        let mut config = empty_config();
        config.business.brand_voice = Some("".to_string());
        let pc = config.profile_completeness();
        assert!(!pc.stages[0].1);
    }

    #[test]
    fn whitespace_only_voice_not_counted() {
        let mut config = empty_config();
        config.business.brand_voice = Some("   ".to_string());
        let pc = config.profile_completeness();
        assert!(!pc.stages[0].1);
    }

    #[test]
    fn fully_enriched_config() {
        let mut config = empty_config();
        config.business.brand_voice = Some("witty".to_string());
        config.business.persona_opinions = vec!["opinion".to_string()];
        config.targets.accounts = vec!["target".to_string()];
        let pc = config.profile_completeness();
        assert_eq!(pc.completed_count(), 3);
        assert!(pc.is_fully_enriched());
        assert!(pc.next_incomplete().is_none());
    }

    #[test]
    fn next_incomplete_returns_first_missing() {
        let mut config = empty_config();
        config.business.brand_voice = Some("witty".to_string());
        // Persona is missing, targeting is missing
        let pc = config.profile_completeness();
        assert_eq!(pc.next_incomplete(), Some(EnrichmentStage::Persona));
    }

    #[test]
    fn one_line_summary_format() {
        let mut config = empty_config();
        config.business.brand_voice = Some("witty".to_string());
        config.targets.accounts = vec!["target".to_string()];
        let pc = config.profile_completeness();
        assert_eq!(pc.one_line_summary(), "Voice OK  Persona --  Targeting OK");
    }

    #[test]
    fn enrichment_stage_all_returns_three() {
        assert_eq!(EnrichmentStage::all().len(), 3);
    }

    #[test]
    fn enrichment_stage_display() {
        assert_eq!(format!("{}", EnrichmentStage::Voice), "Voice");
        assert_eq!(format!("{}", EnrichmentStage::Persona), "Persona");
        assert_eq!(format!("{}", EnrichmentStage::Targeting), "Targeting");
    }
}
