use serde::{Deserialize, Serialize};

// Base prompt template with {language} placeholder
const BASE_PROMPT_TEMPLATE: &str = r#"You are an expert post-processor that converts raw voice transcripts into polished written {language}.

## Self-Corrections & Intent
Resolve self-corrections using last-intent wins: delete retracted parts, keep only the final intended phrasing.
- Prefer the last explicit directive ("we will", "let's", "I'll").
- For conflicting recipients/places/dates/numbers, keep the last stated value.
- Remove "or/maybe" alternatives that precede a final choice.
- If still uncertain, output the safest minimal intent without inventing details.

## Intent Extraction
Focus on what the speaker MEANT, not what they literally said:
- Reconstruct incomplete thoughts into complete, clear sentences.
- When the speaker circles around a point, distill to the core message.
- Preserve the speaker's actual opinions, decisions, and facts — never add your own.

## Verbal Emphasis
Convert spoken emphasis patterns to natural written emphasis:
- Repeated words ("really really important") → strong written emphasis (e.g., "very important" or "critically important").
- Spoken stress markers ("literally", "absolutely", "seriously") → keep only when they add genuine meaning, remove when used as filler.

## Structural Formatting
Intelligently format based on content shape:
- Multiple distinct points or items → use bullet points or a numbered list.
- A single flowing thought → use clean paragraph(s).
- Mixed content → combine paragraphs with lists where natural.
- Short responses (one or two sentences) → keep as plain text, no lists.

## Topic Transitions
When the speaker jumps between topics:
- Add smooth, natural transitions between distinct subjects.
- If topics are unrelated, separate with paragraph breaks.
- Never merge unrelated topics into one sentence.

## Tone & Style
- Preserve the speaker's register (casual, formal, technical) — don't force formality.
- Remove fillers (um, uh, like, you know), false starts, and verbal tics.
- Fix grammar, punctuation, capitalization, and spacing.
- Normalize obvious names/brands/terms when unambiguous; if uncertain, keep generic.
- Format numbers/dates/times naturally. Handle dictation commands only when explicitly said (e.g., "period", "new line").

Output only the polished text."#;

/// Convert ISO 639-1 language code to full language name
pub fn get_language_name(code: &str) -> &'static str {
    match code.to_lowercase().as_str() {
        "en" => "English",
        "es" => "Spanish",
        "fr" => "French",
        "de" => "German",
        "it" => "Italian",
        "pt" => "Portuguese",
        "nl" => "Dutch",
        "pl" => "Polish",
        "ru" => "Russian",
        "ja" => "Japanese",
        "ko" => "Korean",
        "zh" => "Chinese",
        "ar" => "Arabic",
        "hi" => "Hindi",
        "tr" => "Turkish",
        "vi" => "Vietnamese",
        "th" => "Thai",
        "id" => "Indonesian",
        "ms" => "Malay",
        "sv" => "Swedish",
        "da" => "Danish",
        "no" => "Norwegian",
        "fi" => "Finnish",
        "cs" => "Czech",
        "sk" => "Slovak",
        "uk" => "Ukrainian",
        "el" => "Greek",
        "he" => "Hebrew",
        "ro" => "Romanian",
        "hu" => "Hungarian",
        "bg" => "Bulgarian",
        "hr" => "Croatian",
        "sr" => "Serbian",
        "sl" => "Slovenian",
        "lt" => "Lithuanian",
        "lv" => "Latvian",
        "et" => "Estonian",
        "bn" => "Bengali",
        "ta" => "Tamil",
        "te" => "Telugu",
        "mr" => "Marathi",
        "gu" => "Gujarati",
        "kn" => "Kannada",
        "ml" => "Malayalam",
        "pa" => "Punjabi",
        "ur" => "Urdu",
        "fa" => "Persian",
        "sw" => "Swahili",
        "af" => "Afrikaans",
        "ca" => "Catalan",
        "eu" => "Basque",
        "gl" => "Galician",
        "cy" => "Welsh",
        "is" => "Icelandic",
        "mt" => "Maltese",
        "sq" => "Albanian",
        "mk" => "Macedonian",
        "be" => "Belarusian",
        "ka" => "Georgian",
        "hy" => "Armenian",
        "az" => "Azerbaijani",
        "kk" => "Kazakh",
        "uz" => "Uzbek",
        "tl" => "Tagalog",
        "ne" => "Nepali",
        "si" => "Sinhala",
        "km" => "Khmer",
        "lo" => "Lao",
        "my" => "Burmese",
        "mn" => "Mongolian",
        _ => "English", // Default fallback
    }
}

/// Build the base prompt with the specified language
fn build_base_prompt(language: Option<&str>) -> String {
    let lang_name = language.map(get_language_name).unwrap_or("English");
    BASE_PROMPT_TEMPLATE.replace("{language}", lang_name)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnhancementPreset {
    Default,
    Prompts,
    Email,
    Commit,
    Notes,
    Technical,
    Chat,
    Tweet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancementOptions {
    pub preset: EnhancementPreset,
    #[serde(default)]
    pub custom_instructions: Option<String>,
    #[serde(default)]
    pub custom_vocabulary: Vec<String>,
}

impl Default for EnhancementOptions {
    fn default() -> Self {
        Self {
            preset: EnhancementPreset::Default,
            custom_instructions: None,
            custom_vocabulary: Vec::new(),
        }
    }
}

pub fn build_enhancement_prompt(
    text: &str,
    context: Option<&str>,
    options: &EnhancementOptions,
    language: Option<&str>,
) -> String {
    // Base processing applies to ALL presets, with language-aware output
    let base_prompt = build_base_prompt(language);

    // Add mode-specific transformation if not Default
    let mode_transform = match options.preset {
        EnhancementPreset::Default => "",
        EnhancementPreset::Prompts => PROMPTS_TRANSFORM,
        EnhancementPreset::Email => EMAIL_TRANSFORM,
        EnhancementPreset::Commit => COMMIT_TRANSFORM,
        EnhancementPreset::Notes => NOTES_TRANSFORM,
        EnhancementPreset::Technical => TECHNICAL_TRANSFORM,
        EnhancementPreset::Chat => CHAT_TRANSFORM,
        EnhancementPreset::Tweet => TWEET_TRANSFORM,
    };

    // Build the complete prompt
    let mut prompt = if mode_transform.is_empty() {
        // Default preset: just base processing
        format!("{}\n\nTranscribed text:\n{}", base_prompt, text.trim())
    } else {
        // Other presets: base + transform
        format!(
            "{}\n\n{}\n\nTranscribed text:\n{}",
            base_prompt,
            mode_transform,
            text.trim()
        )
    };

    // Add context if provided
    if let Some(ctx) = context {
        prompt.push_str(&format!("\n\nContext: {}", ctx));
    }

    // Append custom vocabulary if provided
    if !options.custom_vocabulary.is_empty() {
        prompt.push_str(&format!(
            "\n\nAlways use these exact terms/spellings: {}",
            options.custom_vocabulary.join(", ")
        ));
    }

    // Append custom instructions if provided
    if let Some(ref instructions) = options.custom_instructions {
        let trimmed = instructions.trim();
        if !trimmed.is_empty() {
            prompt.push_str(&format!("\n\nAdditional instructions: {}", trimmed));
        }
    }

    prompt
}

// Minimal transformation layer for Prompts preset
const PROMPTS_TRANSFORM: &str = r#"Now transform the cleaned text into a concise AI prompt:
- Classify as Request, Question, or Task.
- Add only essential missing what/how/why.
- Include constraints and success criteria if relevant.
- Specify output format when helpful.
- Preserve all technical details; do not invent any.
Return only the enhanced prompt."#;

// Minimal transformation layer for Email preset
const EMAIL_TRANSFORM: &str = r#"Now format the cleaned text as an email:
- Subject: specific and action-oriented.
- Greeting: Hi/Dear/Hello [Name].
- Body: short paragraphs; lead with the key info or ask.
- If it's a request, include action items and deadlines if present.
- Match tone (formal/casual) to the source.
- Closing: appropriate sign-off; use [Your Name].
Return only the formatted email."#;

// Minimal transformation layer for Commit preset
const COMMIT_TRANSFORM: &str = r#"Now convert the cleaned text to a Conventional Commit:
Format: type(scope): description
Types: feat, fix, docs, style, refactor, perf, test, chore, build, ci
Rules: present tense, no period, ≤72 chars; add ! for breaking changes.
Return only the commit message."#;

// Transformation layer for Notes preset
const NOTES_TRANSFORM: &str = r#"Now restructure the cleaned text as organized notes:
- Use bullet points (•) for key items, observations, and action items.
- Group related points under short, clear headers if multiple topics exist.
- Use numbered lists for sequences, steps, or priorities.
- Keep each point concise — one idea per bullet.
- Preserve all factual details; do not add information.
- If the speech is a single topic, use bullets without headers.
Return only the formatted notes."#;

// Transformation layer for Technical preset
const TECHNICAL_TRANSFORM: &str = r#"Now rewrite the cleaned text in professional technical style:
- Use precise, unambiguous language.
- Formal tone — avoid colloquialisms, slang, and casual phrasing.
- Structure with clear paragraphs; one idea per paragraph.
- Preserve all technical terms, acronyms, and specific details exactly.
- Use active voice where possible.
- If the content describes steps or procedures, use numbered lists.
Return only the rewritten text."#;

// Transformation layer for Chat preset
const CHAT_TRANSFORM: &str = r#"Now rewrite the cleaned text as a casual chat message:
- Keep it friendly, natural, and conversational.
- Use contractions (I'm, don't, can't, etc.).
- Short sentences. Match the energy of the original speech.
- It's fine to start sentences with "So", "Yeah", "Also", etc. when natural.
- No formal structure, headers, or bullet points unless the speaker was listing things.
- Keep it brief — chat messages should feel quick to read.
Return only the chat message."#;

// Transformation layer for Tweet preset
const TWEET_TRANSFORM: &str = r#"Now condense the cleaned text into a concise social media post:
- Maximum impact in minimum words. Be punchy and direct.
- Aim for under 280 characters when possible.
- Capture the core message or opinion — drop supporting details.
- No hashtags unless the speaker explicitly mentioned them.
- No emojis unless the speaker's tone strongly warrants one.
Return only the post text."#;

/// Style options for rephrasing already-written text
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RephraseStyle {
    /// Formal, polished professional writing
    Professional,
    /// Shorter, more direct
    Concise,
    /// Warm, approachable tone
    Friendly,
    /// Just fix grammar/spelling, minimal changes
    FixGrammar,
    /// Expand and add detail
    Elaborate,
}

impl Default for RephraseStyle {
    fn default() -> Self {
        Self::Professional
    }
}

impl std::fmt::Display for RephraseStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RephraseStyle::Professional => write!(f, "Professional"),
            RephraseStyle::Concise => write!(f, "Concise"),
            RephraseStyle::Friendly => write!(f, "Friendly"),
            RephraseStyle::FixGrammar => write!(f, "FixGrammar"),
            RephraseStyle::Elaborate => write!(f, "Elaborate"),
        }
    }
}

impl std::str::FromStr for RephraseStyle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Professional" => Ok(RephraseStyle::Professional),
            "Concise" => Ok(RephraseStyle::Concise),
            "Friendly" => Ok(RephraseStyle::Friendly),
            "FixGrammar" => Ok(RephraseStyle::FixGrammar),
            "Elaborate" => Ok(RephraseStyle::Elaborate),
            _ => Err(format!("Unknown rephrase style: {}", s)),
        }
    }
}

/// Build a prompt specifically for rephrasing already-written text (not voice transcripts).
///
/// Unlike `build_enhancement_prompt`, this function does not include voice-specific
/// instructions (fillers, self-corrections, etc.) since the input is written text.
pub fn build_rephrase_prompt(
    text: &str,
    style: &RephraseStyle,
    custom_instructions: Option<&str>,
    language: Option<&str>,
) -> String {
    let lang_name = language.map(get_language_name).unwrap_or("English");

    let style_instruction = match style {
        RephraseStyle::Professional => {
            "Rewrite the following text in a professional, polished style:\n\
            - Use formal, precise language appropriate for business or professional contexts.\n\
            - Eliminate casual phrasing, slang, and colloquialisms.\n\
            - Ensure clear, confident sentence structure.\n\
            - Maintain the original meaning and intent exactly — do not add or remove information."
        }
        RephraseStyle::Concise => {
            "Rewrite the following text to be more concise and direct:\n\
            - Remove redundant words, filler phrases, and unnecessary qualifiers.\n\
            - Use shorter, more impactful sentences.\n\
            - Preserve all key information — do not omit important details.\n\
            - Maintain the original meaning and intent exactly."
        }
        RephraseStyle::Friendly => {
            "Rewrite the following text with a warm, friendly, approachable tone:\n\
            - Use conversational language that feels personal and engaging.\n\
            - Contractions are welcome (I'm, we're, you'll, etc.).\n\
            - Keep the energy positive and inviting.\n\
            - Maintain the original meaning and intent exactly — do not add or remove information."
        }
        RephraseStyle::FixGrammar => {
            "Fix the grammar, spelling, and punctuation in the following text:\n\
            - Correct all spelling mistakes.\n\
            - Fix grammatical errors (subject-verb agreement, tense consistency, article usage, etc.).\n\
            - Correct punctuation and capitalization.\n\
            - Make MINIMAL changes — preserve the author's voice and style as much as possible.\n\
            - Do not rewrite sentences unless they are grammatically broken."
        }
        RephraseStyle::Elaborate => {
            "Expand and elaborate on the following text:\n\
            - Add relevant detail, context, and explanation to make the content richer.\n\
            - Expand brief points into fuller sentences or paragraphs.\n\
            - Maintain the original meaning and intent — only add information that is logically consistent.\n\
            - Do not introduce unrelated topics or invent facts."
        }
    };

    let mut prompt = format!(
        "You are an expert writing assistant. Your task is to rephrase written text while preserving its meaning.\n\
        Output language: {lang_name}.\n\
        Return ONLY the rephrased text — no explanations, no labels, no quotes.\n\n\
        {style_instruction}\n\n\
        Text to rephrase:\n{text}",
        lang_name = lang_name,
        style_instruction = style_instruction,
        text = text.trim(),
    );

    if let Some(instructions) = custom_instructions {
        let trimmed = instructions.trim();
        if !trimmed.is_empty() {
            prompt.push_str(&format!("\n\nAdditional instructions: {}", trimmed));
        }
    }

    prompt
}
