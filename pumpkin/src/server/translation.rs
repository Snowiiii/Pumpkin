#![expect(dead_code)]

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    sync::LazyLock,
};

use pumpkin_config::ADVANCED_CONFIG;
use pumpkin_core::text::{style::Style, TextComponent, TextContent};
use serde_json::Value;
use thiserror::Error;

static EN_US: LazyLock<&str> = LazyLock::new(|| "../assets/lang/en_us/en_us.json");

#[derive(Error, Debug)]
pub enum TranslationError {
    #[error("File does not exist.")]
    NoFileFound,
    #[error("File cannot be opened.")]
    InvalidFile,
    #[error("Invalid translation format. Ensure that all entries are strings, and none is empty")]
    InvalidFormat,
    #[error("Failed to read file. Error: {0}")]
    FileRead(std::io::Error),
    #[error("Invalid JSON encountered. Use 1 JSON Object in the translation file. Error: {0}")]
    JsonParse(serde_json::Error),
}

pub fn translate(message: &'_ str) -> Result<TextComponent<'_>, TranslationError> {
    let config = &ADVANCED_CONFIG.translation;
    if !config.enabled {
        let translations = get_translations(*EN_US, message)?;

        return Ok(translations);
    }

    if config.client_translations {
        return Ok(TextComponent {
            content: TextContent::Translate {
                translate: std::borrow::Cow::Borrowed(message),
                with: vec![],
            },

            style: Style::default(),
            extra: vec![],
        });
    }

    let Some(path) = &config.translation_file_path else {
        return Err(TranslationError::NoFileFound);
    };
    let translations = get_translations(path, message)?;

    Ok(translations)
}

fn get_translations(
    path: impl Into<PathBuf>,
    message: &str,
) -> Result<TextComponent, TranslationError> {
    let translation_file = File::open(path.into()).map_err(TranslationError::FileRead)?;
    let reader = BufReader::new(translation_file);

    let dict = read_translation_file(reader)?;
    let translation_results = find_translation(&dict, message);
    let text_components = TextComponent {
        content: TextContent::Text {
            text: std::borrow::Cow::Owned(translation_results),
        },
        style: Style::default(),
        extra: vec![],
    };

    Ok(text_components)
}
///Read a huge object line by line and tricking `serde_json` into thinking they are individual objects
fn read_translation_file(
    reader: impl BufRead,
) -> Result<HashMap<String, String>, TranslationError> {
    let mut results = HashMap::new();
    let translation_json: Value =
        serde_json::from_reader(BufReader::new(reader)).map_err(TranslationError::JsonParse)?;

    if let Value::Object(map) = translation_json {
        for (text, value) in map {
            if let Value::String(translation) = value {
                if text.is_empty() {
                    return Err(TranslationError::InvalidFormat);
                }

                results.insert(text, translation);
            } else {
                return Err(TranslationError::InvalidFormat);
            }
        }
    } else {
        return Err(TranslationError::InvalidFormat);
    }

    Ok(results)
}

fn find_translation(dict: &HashMap<String, String>, message: &str) -> String {
    for text in dict.keys() {
        if text == message {
            return dict.get(text).unwrap().to_string(); //unwrap because the dictionary should be populated
        }
    }

    String::new()
}

#[cfg(test)]
mod test {

    use pumpkin_core::text::{style::Style, TextComponent, TextContent};

    use crate::server::translation::{get_translations, EN_US};

    use super::{find_translation, read_translation_file};

    #[test]
    fn test_lang_en_us() {
        let intended_result = TextComponent {
            content: TextContent::Text {
                text: std::borrow::Cow::Owned("Unknown advancement: %s".to_string()),
            },
            style: Style::default(),
            extra: vec![],
        };

        assert_eq!(
            intended_result,
            get_translations(*EN_US, "advancement.advancementNotFound").unwrap()
        );
    }

    #[test]
    fn test_lang_ja_jp() {
        let reader = std::io::Cursor::new(
            r#"
            {
            "advancement.advancementNotFound": "\u4e0d\u660e\u306a\u9032\u6357\u3067\u3059\uff1a%s",
            "advancements.adventure.bullseye.description": "30m\u4ee5\u4e0a\u96e2\u308c\u305f\u5834\u6240\u304b\u3089\u7684\u306e\u4e2d\u5fc3\u3092\u5c04\u629c\u304f",
            "advancements.adventure.bullseye.title": "\u7684\u4e2d",
            "commands.advancement.advancementNotFound": "\u300c%s\u300d\u3068\u3044\u3046\u540d\u524d\u306e\u9032\u6357\u306f\u898b\u3064\u304b\u308a\u307e\u305b\u3093\u3067\u3057\u305f"
        }"#.as_bytes()
        );

        let intended_result =
            "\u{4e0d}\u{660e}\u{306a}\u{9032}\u{6357}\u{3067}\u{3059}\u{ff1a}%s".to_owned();

        assert_eq!(
            intended_result,
            find_translation(
                &read_translation_file(reader).unwrap(),
                "advancement.advancementNotFound"
            )
        );
    }

    #[test]
    fn test_lang_it_it() {
        let reader = std::io::Cursor::new(
            r#"
            {
                "advMode.type": "Tipo",
                "advancement.advancementNotFound": "Progresso sconosciuto: %s",
                "advancements.adventure.adventuring_time.description": "Scopri tutti i biomi",
                "advancements.adventure.adventuring_time.title": "All'avventura!"
        }"#
            .as_bytes(),
        );

        let intended_result = "Progresso sconosciuto: %s".to_owned();

        assert_eq!(
            intended_result,
            find_translation(
                &read_translation_file(reader).unwrap(),
                "advancement.advancementNotFound"
            )
        );
    }

    #[test]
    fn no_match() {
        let reader = std::io::Cursor::new(
            r#"
            {
                "accessibility.onboarding.accessibility.button": "Accessibilità...",
                "accessibility.onboarding.screen.narrator": "Premi Invio per attivare l'assistente vocale",
                "accessibility.onboarding.screen.title": "Ti diamo il benvenuto in Minecraft!\n\nVuoi attivare l'assistente vocale o accedere alle impostazioni di accessibilità?",
                "addServer.add": "Fatto"
        }"#.as_bytes()
        );

        let intended_result = String::default();

        assert_eq!(
            intended_result,
            find_translation(
                &read_translation_file(reader).unwrap(),
                "advancement.advancementNotFound"
            )
        );
    }
}
