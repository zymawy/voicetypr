#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::HashMap;

    #[test]
    fn test_ai_error_display() {
        let err = AIError::ApiError("Test error".to_string());
        assert_eq!(err.to_string(), "API error: Test error");

        let err = AIError::ValidationError("Invalid input".to_string());
        assert_eq!(err.to_string(), "Validation error: Invalid input");

        let err = AIError::RateLimitExceeded;
        assert_eq!(err.to_string(), "Rate limit exceeded");
    }

    #[test]
    fn test_ai_enhancement_request_validation() {
        // Empty text
        let request = AIEnhancementRequest {
            text: "".to_string(),
            context: None,
            options: None,
            language: None,
        };
        assert!(request.validate().is_err());

        // Whitespace only
        let request = AIEnhancementRequest {
            text: "   \n\t  ".to_string(),
            context: None,
            options: None,
            language: None,
        };
        assert!(request.validate().is_err());

        // Valid text
        let request = AIEnhancementRequest {
            text: "Hello, world!".to_string(),
            context: None,
            options: None,
            language: None,
        };
        assert!(request.validate().is_ok());

        // Text at max length
        let request = AIEnhancementRequest {
            text: "a".repeat(MAX_TEXT_LENGTH),
            context: None,
            options: None,
            language: None,
        };
        assert!(request.validate().is_ok());

        // Text exceeding max length
        let request = AIEnhancementRequest {
            text: "a".repeat(MAX_TEXT_LENGTH + 1),
            context: None,
            options: None,
            language: None,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_ai_provider_config_serialization() {
        let config = AIProviderConfig {
            provider: "openai".to_string(),
            model: "gpt-5-nano".to_string(),
            api_key: "secret_key".to_string(),
            enabled: true,
            options: HashMap::new(),
        };

        // API key should not be serialized
        let serialized = serde_json::to_string(&config).unwrap();
        assert!(!serialized.contains("secret_key"));
        assert!(serialized.contains("openai"));
        assert!(serialized.contains("gpt-5-nano"));
    }

    #[test]
    fn test_ai_provider_factory_validation() {
        let config = AIProviderConfig {
            provider: "invalid_provider".to_string(),
            model: "test".to_string(),
            api_key: "test".to_string(),
            enabled: true,
            options: HashMap::new(),
        };

        let result = AIProviderFactory::create(&config);
        assert!(result.is_err());

        if let Err(err) = result {
            match err {
                AIError::ProviderNotFound(provider) => {
                    assert_eq!(provider, "invalid_provider");
                }
                _ => panic!("Expected ProviderNotFound error"),
            }
        }
    }

    #[test]
    fn test_enhancement_prompt_generation() {
        use crate::ai::prompts::{build_enhancement_prompt, EnhancementOptions};

        // Test with default options (English)
        let options = EnhancementOptions::default();
        let prompt = build_enhancement_prompt("hello world", None, &options, None);

        assert!(prompt.contains("hello world"));
        assert!(prompt.contains("post-processor that converts raw voice transcripts"));
        assert!(prompt.contains("written English")); // Default language

        // Test with context
        let prompt_with_context =
            build_enhancement_prompt("hello world", Some("Casual conversation"), &options, None);

        assert!(prompt_with_context.contains("Context: Casual conversation"));

        // Test with Spanish language
        let prompt_spanish = build_enhancement_prompt("hola mundo", None, &options, Some("es"));
        assert!(prompt_spanish.contains("written Spanish"));

        // Test with French language
        let prompt_french = build_enhancement_prompt("bonjour monde", None, &options, Some("fr"));
        assert!(prompt_french.contains("written French"));
    }

    #[test]
    fn test_enhancement_presets() {
        use crate::ai::prompts::{build_enhancement_prompt, EnhancementOptions, EnhancementPreset};

        let text = "um hello world";

        // Test Default preset
        let default_options = EnhancementOptions::default();
        let default_prompt = build_enhancement_prompt(text, None, &default_options, None);
        assert!(default_prompt.contains("post-processor that converts raw voice transcripts"));

        // Test Prompts preset
        let mut prompts_options = EnhancementOptions::default();
        prompts_options.preset = EnhancementPreset::Prompts;
        let prompts_prompt = build_enhancement_prompt(text, None, &prompts_options, None);
        assert!(prompts_prompt.contains("transform the cleaned text into a concise AI prompt"));

        // Test Email preset
        let mut email_options = EnhancementOptions::default();
        email_options.preset = EnhancementPreset::Email;
        let email_prompt = build_enhancement_prompt(text, None, &email_options, None);
        assert!(email_prompt.contains("format the cleaned text as an email"));

        // Test Commit preset
        let mut commit_options = EnhancementOptions::default();
        commit_options.preset = EnhancementPreset::Commit;
        let commit_prompt = build_enhancement_prompt(text, None, &commit_options, None);
        assert!(commit_prompt.contains("convert the cleaned text to a Conventional Commit"));

        // Test Notes preset
        let mut notes_options = EnhancementOptions::default();
        notes_options.preset = EnhancementPreset::Notes;
        let notes_prompt = build_enhancement_prompt(text, None, &notes_options, None);
        assert!(notes_prompt.contains("restructure the cleaned text as organized notes"));

        // Test Technical preset
        let mut tech_options = EnhancementOptions::default();
        tech_options.preset = EnhancementPreset::Technical;
        let tech_prompt = build_enhancement_prompt(text, None, &tech_options, None);
        assert!(tech_prompt.contains("rewrite the cleaned text in professional technical style"));

        // Test Chat preset
        let mut chat_options = EnhancementOptions::default();
        chat_options.preset = EnhancementPreset::Chat;
        let chat_prompt = build_enhancement_prompt(text, None, &chat_options, None);
        assert!(chat_prompt.contains("rewrite the cleaned text as a casual chat message"));

        // Test Tweet preset
        let mut tweet_options = EnhancementOptions::default();
        tweet_options.preset = EnhancementPreset::Tweet;
        let tweet_prompt = build_enhancement_prompt(text, None, &tweet_options, None);
        assert!(tweet_prompt.contains("condense the cleaned text into a concise social media post"));
    }

    #[test]
    fn test_self_correction_rules_in_all_presets() {
        use crate::ai::prompts::{build_enhancement_prompt, EnhancementOptions, EnhancementPreset};

        let test_text = "send it to john... to mary";

        // Test that ALL presets include self-correction rules
        let presets = vec![
            EnhancementPreset::Default,
            EnhancementPreset::Prompts,
            EnhancementPreset::Email,
            EnhancementPreset::Commit,
            EnhancementPreset::Notes,
            EnhancementPreset::Technical,
            EnhancementPreset::Chat,
            EnhancementPreset::Tweet,
        ];

        for preset in presets {
            let mut options = EnhancementOptions::default();
            options.preset = preset.clone();
            let prompt = build_enhancement_prompt(test_text, None, &options, None);

            // All prompts should include self-correction rules
            assert!(
                prompt.contains("Self-Corrections"),
                "Preset {:?} should include self-correction rules",
                preset
            );
        }
    }

    #[test]
    fn test_layered_architecture() {
        use crate::ai::prompts::{build_enhancement_prompt, EnhancementOptions, EnhancementPreset};

        let test_text = "test";

        // Test that all presets include base processing
        let presets = vec![
            EnhancementPreset::Default,
            EnhancementPreset::Prompts,
            EnhancementPreset::Email,
            EnhancementPreset::Commit,
            EnhancementPreset::Notes,
            EnhancementPreset::Technical,
            EnhancementPreset::Chat,
            EnhancementPreset::Tweet,
        ];

        for preset in presets {
            let mut options = EnhancementOptions::default();
            options.preset = preset.clone();
            let prompt = build_enhancement_prompt(test_text, None, &options, None);

            // All should include self-correction rules
            assert!(
                prompt.contains("Self-Corrections"),
                "Preset {:?} should include self-correction rules",
                preset
            );

            // All should include base processing
            assert!(
                prompt.contains("post-processor"),
                "Preset {:?} should include base processing",
                preset
            );

            // Non-default presets should have transformation instruction
            if !matches!(preset, EnhancementPreset::Default) {
                assert!(
                    prompt.contains("Now"),
                    "Preset {:?} should have transformation",
                    preset
                );
            }
        }
    }

    #[test]
    fn test_default_prompt_comprehensive_features() {
        use crate::ai::prompts::{build_enhancement_prompt, EnhancementOptions};

        let test_text = "test transcription";
        let options = EnhancementOptions::default();
        let prompt = build_enhancement_prompt(test_text, None, &options, None);

        // Test that Default prompt includes all comprehensive features

        // 1. Self-correction handling
        assert!(
            prompt.contains("Self-Corrections"),
            "Should handle self-corrections"
        );
        assert!(
            prompt.contains("last-intent wins"),
            "Should use last-intent policy"
        );

        // 2. Error correction
        assert!(
            prompt.contains("grammar, punctuation, capitalization"),
            "Should handle grammar and spelling"
        );

        // 3. Intent extraction
        assert!(
            prompt.contains("Intent Extraction"),
            "Should include intent extraction"
        );

        // 4. Structural formatting
        assert!(
            prompt.contains("Structural Formatting"),
            "Should include structural formatting"
        );

        // 5. Technical terms
        assert!(
            prompt.contains("Normalize obvious names/brands/terms"),
            "Should normalize technical terms"
        );
    }

    #[test]
    fn test_custom_instructions_appended() {
        use crate::ai::prompts::{build_enhancement_prompt, EnhancementOptions};

        let mut options = EnhancementOptions::default();
        options.custom_instructions = Some("Always use formal tone".to_string());

        let prompt = build_enhancement_prompt("hello world", None, &options, None);
        assert!(prompt.contains("Additional instructions: Always use formal tone"));
    }

    #[test]
    fn test_custom_instructions_empty_not_appended() {
        use crate::ai::prompts::{build_enhancement_prompt, EnhancementOptions};

        let mut options = EnhancementOptions::default();
        options.custom_instructions = Some("   ".to_string());

        let prompt = build_enhancement_prompt("hello world", None, &options, None);
        assert!(!prompt.contains("Additional instructions"));
    }

    #[test]
    fn test_custom_vocabulary_appended() {
        use crate::ai::prompts::{build_enhancement_prompt, EnhancementOptions};

        let mut options = EnhancementOptions::default();
        options.custom_vocabulary = vec!["VoiceTypr".to_string(), "Tauri".to_string()];

        let prompt = build_enhancement_prompt("hello world", None, &options, None);
        assert!(prompt.contains("Always use these exact terms/spellings: VoiceTypr, Tauri"));
    }

    #[test]
    fn test_custom_vocabulary_empty_not_appended() {
        use crate::ai::prompts::{build_enhancement_prompt, EnhancementOptions};

        let options = EnhancementOptions::default();
        let prompt = build_enhancement_prompt("hello world", None, &options, None);
        assert!(!prompt.contains("Always use these exact terms"));
    }

    #[test]
    fn test_vocabulary_before_instructions() {
        use crate::ai::prompts::{build_enhancement_prompt, EnhancementOptions};

        let mut options = EnhancementOptions::default();
        options.custom_vocabulary = vec!["VoiceTypr".to_string()];
        options.custom_instructions = Some("Use formal tone".to_string());

        let prompt = build_enhancement_prompt("hello world", None, &options, None);
        let vocab_pos = prompt.find("Always use these exact terms").unwrap();
        let instr_pos = prompt.find("Additional instructions").unwrap();
        assert!(vocab_pos < instr_pos, "Vocabulary should appear before instructions");
    }

    #[test]
    fn test_enhancement_options_serde_backward_compat() {
        // Old stored JSON without custom fields should deserialize fine
        let old_json = r#"{"preset":"Default"}"#;
        let options: EnhancementOptions = serde_json::from_str(old_json).unwrap();
        assert!(options.custom_instructions.is_none());
        assert!(options.custom_vocabulary.is_empty());
    }

    #[test]
    fn test_ai_model_serialization() {
        let model = AIModel {
            id: "test-model".to_string(),
            name: "Test Model".to_string(),
            description: Some("A test model".to_string()),
        };

        let json = serde_json::to_string(&model).unwrap();
        assert!(json.contains("test-model"));
        assert!(json.contains("Test Model"));
        assert!(json.contains("A test model"));
    }

    #[test]
    fn test_language_name_mapping() {
        use crate::ai::prompts::get_language_name;

        // Test common languages
        assert_eq!(get_language_name("en"), "English");
        assert_eq!(get_language_name("es"), "Spanish");
        assert_eq!(get_language_name("fr"), "French");
        assert_eq!(get_language_name("de"), "German");
        assert_eq!(get_language_name("ja"), "Japanese");
        assert_eq!(get_language_name("zh"), "Chinese");
        assert_eq!(get_language_name("ar"), "Arabic");
        assert_eq!(get_language_name("hi"), "Hindi");
        assert_eq!(get_language_name("pt"), "Portuguese");
        assert_eq!(get_language_name("ru"), "Russian");

        // Test case insensitivity
        assert_eq!(get_language_name("EN"), "English");
        assert_eq!(get_language_name("Es"), "Spanish");

        // Test fallback for unknown language codes
        assert_eq!(get_language_name("xyz"), "English");
        assert_eq!(get_language_name(""), "English");
    }

    #[test]
    fn test_language_aware_prompts() {
        use crate::ai::prompts::{build_enhancement_prompt, EnhancementOptions};

        let options = EnhancementOptions::default();
        let text = "test text";

        // English (default)
        let prompt_en = build_enhancement_prompt(text, None, &options, Some("en"));
        assert!(prompt_en.contains("written English"));

        // Spanish
        let prompt_es = build_enhancement_prompt(text, None, &options, Some("es"));
        assert!(prompt_es.contains("written Spanish"));

        // Japanese
        let prompt_ja = build_enhancement_prompt(text, None, &options, Some("ja"));
        assert!(prompt_ja.contains("written Japanese"));

        // None defaults to English
        let prompt_none = build_enhancement_prompt(text, None, &options, None);
        assert!(prompt_none.contains("written English"));
    }

    // =========================================================================
    // Rephrase prompt tests
    // =========================================================================

    #[test]
    fn test_rephrase_prompt_contains_input_text() {
        use crate::ai::prompts::{build_rephrase_prompt, RephraseStyle};

        let text = "Hello world, this is a test sentence.";
        let prompt = build_rephrase_prompt(text, &RephraseStyle::Professional, None, None);
        assert!(prompt.contains(text));
    }

    #[test]
    fn test_rephrase_prompt_professional_style() {
        use crate::ai::prompts::{build_rephrase_prompt, RephraseStyle};

        let prompt =
            build_rephrase_prompt("some text", &RephraseStyle::Professional, None, None);
        assert!(prompt.contains("professional"));
        assert!(prompt.contains("English")); // default language
    }

    #[test]
    fn test_rephrase_prompt_concise_style() {
        use crate::ai::prompts::{build_rephrase_prompt, RephraseStyle};

        let prompt = build_rephrase_prompt("some text", &RephraseStyle::Concise, None, None);
        assert!(prompt.contains("concise"));
    }

    #[test]
    fn test_rephrase_prompt_friendly_style() {
        use crate::ai::prompts::{build_rephrase_prompt, RephraseStyle};

        let prompt = build_rephrase_prompt("some text", &RephraseStyle::Friendly, None, None);
        assert!(prompt.contains("friendly") || prompt.contains("warm"));
    }

    #[test]
    fn test_rephrase_prompt_fix_grammar_style() {
        use crate::ai::prompts::{build_rephrase_prompt, RephraseStyle};

        let prompt = build_rephrase_prompt("some text", &RephraseStyle::FixGrammar, None, None);
        assert!(prompt.contains("grammar"));
    }

    #[test]
    fn test_rephrase_prompt_elaborate_style() {
        use crate::ai::prompts::{build_rephrase_prompt, RephraseStyle};

        let prompt = build_rephrase_prompt("some text", &RephraseStyle::Elaborate, None, None);
        assert!(prompt.contains("Expand") || prompt.contains("elaborate"));
    }

    #[test]
    fn test_rephrase_prompt_language_aware() {
        use crate::ai::prompts::{build_rephrase_prompt, RephraseStyle};

        let prompt_es =
            build_rephrase_prompt("hola mundo", &RephraseStyle::Professional, None, Some("es"));
        assert!(prompt_es.contains("Spanish"));

        let prompt_fr =
            build_rephrase_prompt("bonjour", &RephraseStyle::Professional, None, Some("fr"));
        assert!(prompt_fr.contains("French"));

        // None falls back to English
        let prompt_default =
            build_rephrase_prompt("hello", &RephraseStyle::Professional, None, None);
        assert!(prompt_default.contains("English"));
    }

    #[test]
    fn test_rephrase_prompt_custom_instructions_appended() {
        use crate::ai::prompts::{build_rephrase_prompt, RephraseStyle};

        let prompt = build_rephrase_prompt(
            "some text",
            &RephraseStyle::Professional,
            Some("Always use bullet points"),
            None,
        );
        assert!(prompt.contains("Additional instructions: Always use bullet points"));
    }

    #[test]
    fn test_rephrase_prompt_empty_custom_instructions_not_appended() {
        use crate::ai::prompts::{build_rephrase_prompt, RephraseStyle};

        let prompt = build_rephrase_prompt(
            "some text",
            &RephraseStyle::Professional,
            Some("   "),
            None,
        );
        assert!(!prompt.contains("Additional instructions"));
    }

    #[test]
    fn test_rephrase_prompt_does_not_contain_voice_instructions() {
        use crate::ai::prompts::{build_rephrase_prompt, RephraseStyle};

        // Rephrase prompts should NOT include voice-transcript-specific language
        let prompt =
            build_rephrase_prompt("some text", &RephraseStyle::Professional, None, None);
        assert!(!prompt.contains("voice transcript"));
        assert!(!prompt.contains("fillers"));
        assert!(!prompt.contains("Self-Corrections"));
    }

    #[test]
    fn test_rephrase_style_from_str() {
        use crate::ai::prompts::RephraseStyle;
        use std::str::FromStr;

        assert_eq!(
            RephraseStyle::from_str("Professional").unwrap(),
            RephraseStyle::Professional
        );
        assert_eq!(
            RephraseStyle::from_str("Concise").unwrap(),
            RephraseStyle::Concise
        );
        assert_eq!(
            RephraseStyle::from_str("Friendly").unwrap(),
            RephraseStyle::Friendly
        );
        assert_eq!(
            RephraseStyle::from_str("FixGrammar").unwrap(),
            RephraseStyle::FixGrammar
        );
        assert_eq!(
            RephraseStyle::from_str("Elaborate").unwrap(),
            RephraseStyle::Elaborate
        );
        assert!(RephraseStyle::from_str("InvalidStyle").is_err());
    }

    #[test]
    fn test_rephrase_style_display() {
        use crate::ai::prompts::RephraseStyle;

        assert_eq!(RephraseStyle::Professional.to_string(), "Professional");
        assert_eq!(RephraseStyle::Concise.to_string(), "Concise");
        assert_eq!(RephraseStyle::Friendly.to_string(), "Friendly");
        assert_eq!(RephraseStyle::FixGrammar.to_string(), "FixGrammar");
        assert_eq!(RephraseStyle::Elaborate.to_string(), "Elaborate");
    }
}
