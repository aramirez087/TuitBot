//! LLM pricing lookup for cost estimation.
//!
//! Provides per-token pricing for known models and computes estimated costs.
//! Prices are in USD per million tokens; Ollama / unknown models default to $0.

/// Per-token pricing for a model.
#[derive(Debug, Clone, Copy)]
pub struct ModelPricing {
    /// USD per million input tokens.
    pub input_per_million: f64,
    /// USD per million output tokens.
    pub output_per_million: f64,
}

impl ModelPricing {
    /// Compute the estimated cost for the given token counts.
    pub fn compute_cost(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        let input_cost = (input_tokens as f64 / 1_000_000.0) * self.input_per_million;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * self.output_per_million;
        input_cost + output_cost
    }
}

/// Look up pricing for a provider + model combination.
///
/// Falls back to zero-cost for Ollama and unknown models.
pub fn lookup(provider: &str, model: &str) -> ModelPricing {
    match provider {
        "openai" => lookup_openai(model),
        "anthropic" => lookup_anthropic(model),
        "gemini" | "google" => lookup_gemini(model),
        "deepseek" => lookup_deepseek(model),
        // Ollama and unknowns are free (local inference).
        _ => ModelPricing {
            input_per_million: 0.0,
            output_per_million: 0.0,
        },
    }
}

fn lookup_openai(model: &str) -> ModelPricing {
    if model.starts_with("gpt-4o-mini") {
        ModelPricing {
            input_per_million: 0.15,
            output_per_million: 0.60,
        }
    } else if model.starts_with("gpt-4o") {
        ModelPricing {
            input_per_million: 2.50,
            output_per_million: 10.0,
        }
    } else if model.starts_with("gpt-4-turbo") {
        ModelPricing {
            input_per_million: 10.0,
            output_per_million: 30.0,
        }
    } else if model.starts_with("gpt-3.5") {
        ModelPricing {
            input_per_million: 0.50,
            output_per_million: 1.50,
        }
    } else {
        // Unknown OpenAI model — use gpt-4o-mini as a reasonable default.
        ModelPricing {
            input_per_million: 0.15,
            output_per_million: 0.60,
        }
    }
}

fn lookup_anthropic(model: &str) -> ModelPricing {
    if model.contains("opus") {
        ModelPricing {
            input_per_million: 15.0,
            output_per_million: 75.0,
        }
    } else if model.contains("sonnet") {
        ModelPricing {
            input_per_million: 3.0,
            output_per_million: 15.0,
        }
    } else if model.contains("haiku") {
        ModelPricing {
            input_per_million: 1.0,
            output_per_million: 5.0,
        }
    } else {
        // Unknown Anthropic model — use sonnet pricing.
        ModelPricing {
            input_per_million: 3.0,
            output_per_million: 15.0,
        }
    }
}

fn lookup_gemini(model: &str) -> ModelPricing {
    if model.contains("2.5-pro") {
        ModelPricing {
            input_per_million: 1.25,
            output_per_million: 10.0,
        }
    } else if model.contains("2.5-flash") {
        ModelPricing {
            input_per_million: 0.15,
            output_per_million: 0.60,
        }
    } else if model.contains("2.0-flash") {
        ModelPricing {
            input_per_million: 0.10,
            output_per_million: 0.40,
        }
    } else if model.contains("1.5-pro") {
        ModelPricing {
            input_per_million: 1.25,
            output_per_million: 5.0,
        }
    } else if model.contains("1.5-flash") {
        ModelPricing {
            input_per_million: 0.075,
            output_per_million: 0.30,
        }
    } else {
        // Unknown Gemini model — use 2.0-flash pricing.
        ModelPricing {
            input_per_million: 0.10,
            output_per_million: 0.40,
        }
    }
}

fn lookup_deepseek(model: &str) -> ModelPricing {
    if model.contains("chat") || model.contains("v3") {
        ModelPricing {
            input_per_million: 0.27,
            output_per_million: 1.10,
        }
    } else if model.contains("reasoner") || model.contains("r1") {
        ModelPricing {
            input_per_million: 0.55,
            output_per_million: 2.19,
        }
    } else {
        // Unknown DeepSeek model — use chat pricing.
        ModelPricing {
            input_per_million: 0.27,
            output_per_million: 1.10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_gpt4o_pricing() {
        let p = lookup("openai", "gpt-4o");
        assert!((p.input_per_million - 2.5).abs() < f64::EPSILON);
        assert!((p.output_per_million - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn openai_gpt4o_mini_pricing() {
        let p = lookup("openai", "gpt-4o-mini");
        assert!((p.input_per_million - 0.15).abs() < f64::EPSILON);
    }

    #[test]
    fn anthropic_sonnet_pricing() {
        let p = lookup("anthropic", "claude-sonnet-4-5-20250514");
        assert!((p.input_per_million - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ollama_is_free() {
        let p = lookup("ollama", "llama3.1");
        assert!((p.input_per_million).abs() < f64::EPSILON);
        assert!((p.output_per_million).abs() < f64::EPSILON);
    }

    #[test]
    fn compute_cost_basic() {
        let p = ModelPricing {
            input_per_million: 3.0,
            output_per_million: 15.0,
        };
        // 1000 input + 500 output
        let cost = p.compute_cost(1000, 500);
        let expected = (1000.0 / 1_000_000.0) * 3.0 + (500.0 / 1_000_000.0) * 15.0;
        assert!((cost - expected).abs() < 1e-10);
    }

    #[test]
    fn gemini_2_5_pro_pricing() {
        let p = lookup("gemini", "gemini-2.5-pro");
        assert!((p.input_per_million - 1.25).abs() < f64::EPSILON);
        assert!((p.output_per_million - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn gemini_2_5_flash_pricing() {
        let p = lookup("gemini", "gemini-2.5-flash");
        assert!((p.input_per_million - 0.15).abs() < f64::EPSILON);
    }

    #[test]
    fn gemini_google_alias() {
        let p = lookup("google", "gemini-2.0-flash");
        assert!((p.input_per_million - 0.10).abs() < f64::EPSILON);
    }

    #[test]
    fn deepseek_chat_pricing() {
        let p = lookup("deepseek", "deepseek-chat");
        assert!((p.input_per_million - 0.27).abs() < f64::EPSILON);
        assert!((p.output_per_million - 1.10).abs() < f64::EPSILON);
    }

    #[test]
    fn deepseek_reasoner_pricing() {
        let p = lookup("deepseek", "deepseek-reasoner");
        assert!((p.input_per_million - 0.55).abs() < f64::EPSILON);
        assert!((p.output_per_million - 2.19).abs() < f64::EPSILON);
    }

    #[test]
    fn deepseek_r1_pricing() {
        let p = lookup("deepseek", "deepseek-r1");
        assert!((p.input_per_million - 0.55).abs() < f64::EPSILON);
    }

    #[test]
    fn unknown_provider_is_free() {
        let p = lookup("custom", "my-model");
        assert!((p.input_per_million).abs() < f64::EPSILON);
    }
}
