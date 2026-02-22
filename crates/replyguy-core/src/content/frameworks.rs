//! Content frameworks for varied, human-sounding output.
//!
//! Provides archetypes for replies, formats for tweets, and structures
//! for threads. Each variant includes prompt fragment guidance so the
//! LLM produces distinctly different content depending on the chosen
//! framework.

use rand::seq::SliceRandom;

// ============================================================================
// Reply archetypes
// ============================================================================

/// How we engage in a reply — shapes the prompt so the LLM varies
/// its approach instead of always producing the same structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplyArchetype {
    /// Agree with the author and extend their point.
    AgreeAndExpand,
    /// Respectfully offer an alternative perspective.
    RespectfulDisagree,
    /// Add a concrete data point, stat, or example.
    AddData,
    /// Ask a thoughtful follow-up question.
    AskQuestion,
    /// Share a brief personal experience related to the topic.
    ShareExperience,
}

impl ReplyArchetype {
    /// Weighted selection — prefer archetypes that start conversations.
    pub fn select(rng: &mut impl rand::Rng) -> Self {
        // Weights: AgreeAndExpand 30, AskQuestion 25, ShareExperience 20,
        //          AddData 15, RespectfulDisagree 10
        let choices: &[(Self, u32)] = &[
            (Self::AgreeAndExpand, 30),
            (Self::AskQuestion, 25),
            (Self::ShareExperience, 20),
            (Self::AddData, 15),
            (Self::RespectfulDisagree, 10),
        ];

        let total: u32 = choices.iter().map(|(_, w)| w).sum();
        let mut roll = rng.gen_range(0..total);
        for (archetype, weight) in choices {
            if roll < *weight {
                return *archetype;
            }
            roll -= weight;
        }
        Self::AgreeAndExpand
    }

    /// Prompt fragment injected into the system prompt.
    pub fn prompt_fragment(self) -> &'static str {
        match self {
            Self::AgreeAndExpand => {
                "Approach: Agree with the author's point and extend it with \
                 an additional insight or implication they didn't mention."
            }
            Self::RespectfulDisagree => {
                "Approach: Respectfully offer an alternative take. Start with \
                 what you agree with, then pivot to where you see it differently. \
                 Keep it constructive — never confrontational."
            }
            Self::AddData => {
                "Approach: Add a concrete data point, stat, example, or case study \
                 that supports or contextualizes the topic. Cite specifics when possible."
            }
            Self::AskQuestion => {
                "Approach: Ask a thoughtful follow-up question that shows you've engaged \
                 deeply with the tweet. The question should invite the author to elaborate."
            }
            Self::ShareExperience => {
                "Approach: Share a brief personal experience or observation related to the \
                 topic. Use 'I' language and keep it genuine and specific."
            }
        }
    }
}

impl std::fmt::Display for ReplyArchetype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AgreeAndExpand => write!(f, "agree_and_expand"),
            Self::RespectfulDisagree => write!(f, "respectful_disagree"),
            Self::AddData => write!(f, "add_data"),
            Self::AskQuestion => write!(f, "ask_question"),
            Self::ShareExperience => write!(f, "share_experience"),
        }
    }
}

// ============================================================================
// Tweet formats
// ============================================================================

/// Structural format for an original tweet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweetFormat {
    /// Numbered list of tips or points.
    List,
    /// "Most people think X. But actually Y."
    ContrarianTake,
    /// "Most people think X, but the reality is..."
    MostPeopleThinkX,
    /// A short story or anecdote.
    Storytelling,
    /// "Before: X. After: Y."
    BeforeAfter,
    /// Pose a question to the audience.
    Question,
    /// A single actionable tip.
    Tip,
}

impl TweetFormat {
    /// All available formats.
    const ALL: &'static [Self] = &[
        Self::List,
        Self::ContrarianTake,
        Self::MostPeopleThinkX,
        Self::Storytelling,
        Self::BeforeAfter,
        Self::Question,
        Self::Tip,
    ];

    /// Pick a random format, avoiding recently used ones.
    pub fn select(recent: &[Self], rng: &mut impl rand::Rng) -> Self {
        let available: Vec<Self> = Self::ALL
            .iter()
            .copied()
            .filter(|f| !recent.contains(f))
            .collect();

        if available.is_empty() {
            *Self::ALL.choose(rng).expect("ALL is non-empty")
        } else {
            *available.choose(rng).expect("available is non-empty")
        }
    }

    /// Prompt fragment injected into the system prompt.
    pub fn prompt_fragment(self) -> &'static str {
        match self {
            Self::List => {
                "Format: Write a numbered list of 3-5 quick tips or insights. \
                 Keep each item to one line."
            }
            Self::ContrarianTake => {
                "Format: Start with a common belief, then challenge it with an \
                 unexpected truth. Structure: 'Everyone says X. But actually, Y.'"
            }
            Self::MostPeopleThinkX => {
                "Format: 'Most people think [common assumption]. The reality: [insight].'"
            }
            Self::Storytelling => {
                "Format: Tell a very brief story or anecdote (2-3 sentences) that \
                 illustrates the topic. End with the lesson."
            }
            Self::BeforeAfter => {
                "Format: Show a transformation. 'Before: [old way]. After: [new way]. \
                 [Brief insight on why the change matters].'"
            }
            Self::Question => {
                "Format: Pose a thought-provoking question to the audience that invites \
                 engagement. Optionally share your own answer in 1-2 sentences."
            }
            Self::Tip => {
                "Format: Share one specific, actionable tip. Be concrete — include the \
                 exact steps or command, not vague advice."
            }
        }
    }
}

impl std::fmt::Display for TweetFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::List => write!(f, "list"),
            Self::ContrarianTake => write!(f, "contrarian_take"),
            Self::MostPeopleThinkX => write!(f, "most_people_think_x"),
            Self::Storytelling => write!(f, "storytelling"),
            Self::BeforeAfter => write!(f, "before_after"),
            Self::Question => write!(f, "question"),
            Self::Tip => write!(f, "tip"),
        }
    }
}

// ============================================================================
// Thread structures
// ============================================================================

/// Structural template for a multi-tweet thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadStructure {
    /// "I went from X to Y. Here's the journey" — transformation arc.
    Transformation,
    /// "My framework for X" — step-by-step process.
    Framework,
    /// "N mistakes I made doing X" — lessons learned.
    Mistakes,
    /// Deep analysis of a topic with supporting evidence.
    Analysis,
}

impl ThreadStructure {
    /// All available structures.
    const ALL: &'static [Self] = &[
        Self::Transformation,
        Self::Framework,
        Self::Mistakes,
        Self::Analysis,
    ];

    /// Pick a random structure.
    pub fn select(rng: &mut impl rand::Rng) -> Self {
        *Self::ALL.choose(rng).expect("ALL is non-empty")
    }

    /// Prompt fragment injected into the system prompt.
    pub fn prompt_fragment(self) -> &'static str {
        match self {
            Self::Transformation => {
                "Structure: Tell a transformation story. Start with the 'before' state, \
                 walk through the key turning points, and end with the 'after' state \
                 and lessons learned."
            }
            Self::Framework => {
                "Structure: Present a step-by-step framework. Tweet 1 hooks with the \
                 problem, subsequent tweets present each step, and the last tweet \
                 summarizes the framework."
            }
            Self::Mistakes => {
                "Structure: Share mistakes and lessons. Tweet 1 hooks with 'N mistakes \
                 I made doing X', each subsequent tweet is one mistake with what you \
                 learned, and the last tweet is the key takeaway."
            }
            Self::Analysis => {
                "Structure: Deep-dive analysis. Tweet 1 states the thesis, subsequent \
                 tweets provide evidence or arguments, and the last tweet draws a conclusion."
            }
        }
    }
}

impl std::fmt::Display for ThreadStructure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transformation => write!(f, "transformation"),
            Self::Framework => write!(f, "framework"),
            Self::Mistakes => write!(f, "mistakes"),
            Self::Analysis => write!(f, "analysis"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reply_archetype_select_returns_valid() {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let _ = ReplyArchetype::select(&mut rng);
        }
    }

    #[test]
    fn reply_archetype_select_distribution() {
        let mut rng = rand::thread_rng();
        let mut counts = [0u32; 5];
        for _ in 0..1000 {
            let archetype = ReplyArchetype::select(&mut rng);
            match archetype {
                ReplyArchetype::AgreeAndExpand => counts[0] += 1,
                ReplyArchetype::RespectfulDisagree => counts[1] += 1,
                ReplyArchetype::AddData => counts[2] += 1,
                ReplyArchetype::AskQuestion => counts[3] += 1,
                ReplyArchetype::ShareExperience => counts[4] += 1,
            }
        }
        // All archetypes should appear at least once in 1000 samples
        for (i, count) in counts.iter().enumerate() {
            assert!(
                *count > 0,
                "archetype index {i} never selected in 1000 samples"
            );
        }
        // AgreeAndExpand should appear more often than RespectfulDisagree
        assert!(
            counts[0] > counts[1],
            "AgreeAndExpand should be more frequent"
        );
    }

    #[test]
    fn reply_archetype_prompt_fragments_non_empty() {
        let archetypes = [
            ReplyArchetype::AgreeAndExpand,
            ReplyArchetype::RespectfulDisagree,
            ReplyArchetype::AddData,
            ReplyArchetype::AskQuestion,
            ReplyArchetype::ShareExperience,
        ];
        for a in archetypes {
            assert!(!a.prompt_fragment().is_empty());
        }
    }

    #[test]
    fn reply_archetype_display() {
        assert_eq!(
            ReplyArchetype::AgreeAndExpand.to_string(),
            "agree_and_expand"
        );
        assert_eq!(ReplyArchetype::AskQuestion.to_string(), "ask_question");
    }

    #[test]
    fn tweet_format_select_avoids_recent() {
        let mut rng = rand::thread_rng();
        let recent = vec![TweetFormat::List, TweetFormat::Tip, TweetFormat::Question];

        for _ in 0..50 {
            let format = TweetFormat::select(&recent, &mut rng);
            assert!(!recent.contains(&format));
        }
    }

    #[test]
    fn tweet_format_select_clears_when_all_recent() {
        let mut rng = rand::thread_rng();
        let recent: Vec<TweetFormat> = TweetFormat::ALL.to_vec();
        // When all are recent, should still pick one
        let format = TweetFormat::select(&recent, &mut rng);
        assert!(TweetFormat::ALL.contains(&format));
    }

    #[test]
    fn tweet_format_prompt_fragments_non_empty() {
        for f in TweetFormat::ALL {
            assert!(!f.prompt_fragment().is_empty());
        }
    }

    #[test]
    fn tweet_format_display() {
        assert_eq!(TweetFormat::List.to_string(), "list");
        assert_eq!(TweetFormat::ContrarianTake.to_string(), "contrarian_take");
        assert_eq!(TweetFormat::BeforeAfter.to_string(), "before_after");
    }

    #[test]
    fn thread_structure_select_returns_valid() {
        let mut rng = rand::thread_rng();
        for _ in 0..50 {
            let structure = ThreadStructure::select(&mut rng);
            assert!(ThreadStructure::ALL.contains(&structure));
        }
    }

    #[test]
    fn thread_structure_prompt_fragments_non_empty() {
        for s in ThreadStructure::ALL {
            assert!(!s.prompt_fragment().is_empty());
        }
    }

    #[test]
    fn thread_structure_display() {
        assert_eq!(
            ThreadStructure::Transformation.to_string(),
            "transformation"
        );
        assert_eq!(ThreadStructure::Framework.to_string(), "framework");
        assert_eq!(ThreadStructure::Mistakes.to_string(), "mistakes");
        assert_eq!(ThreadStructure::Analysis.to_string(), "analysis");
    }
}
