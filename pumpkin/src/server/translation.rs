#![expect(dead_code)]

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use pumpkin_config::ADVANCED_CONFIG;
use pumpkin_core::text::{style::Style, TextComponent, TextContent};
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TranslationError {
    #[error("File does not exist")]
    NoFileFound,
    #[error("File cannot be opened.")]
    InvalidFile,
    #[error("Failed to read file. Error: {0}")]
    FileRead(std::io::Error),
    #[error("Invalid JSON encountered. Use only objects in the translation file.")]
    JsonParse,
}

pub fn translate(message: &'_ str) -> Result<TextComponent<'_>, TranslationError> {
    let config = &ADVANCED_CONFIG.translation;
    if !config.enabled {
        let path = "assets/lang/en_us/en_us.json";
        let translations = get_translations(path, message)?;

        return Ok(translations);
    }

    if config.client_translations {
        return Ok(TextComponent {
            content: TextContent::Translate {
                translate: std::borrow::Cow::Borrowed(message),
                with: vec![],
            },

            style: Style::default(),
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
    let translation_file = File::open(path.into()).map_err(|_| TranslationError::InvalidFile)?;
    let reader = BufReader::new(translation_file);

    let json_results = read_translation_file(reader, message)?;
    let parsed_results = parse_json_object(json_results, message);
    let text_components = TextComponent {
        content: TextContent::Text {
            text: std::borrow::Cow::Owned(parsed_results),
        },
        style: Style::default(),
    };

    Ok(text_components)
}
///Read a huge object line by line and tricking `serde_json` into thinking they are individual objects
fn read_translation_file(
    mut reader: impl BufRead,
    message: &str,
) -> Result<Vec<Value>, TranslationError> {
    let mut buf = String::new();
    let mut results = Vec::new();

    loop {
        let bytes_read = reader
            .read_line(&mut buf)
            .map_err(TranslationError::FileRead)?;

        if bytes_read == 0 {
            break;
        }

        if buf == "{" || buf == "}" {
            continue;
        }

        if buf.contains(message) {
            let mut buf = buf.trim().replace(',', "");
            buf.insert(0, '{');
            buf.push('}');
            let v: Value = serde_json::from_str(&buf).map_err(|_| TranslationError::JsonParse)?;

            results.push(v);
        }

        buf.clear();
    }

    Ok(results)
}

fn parse_json_object(vec: Vec<Value>, message: &str) -> String {
    for value in vec {
        //breaking up a serde_json object, looks really ugly not gonna lie
        if let Value::Object(map) = value {
            if let Some(text) = map.keys().next() {
                if text.trim() == message {
                    if let Some(Value::String(translation)) = map.values().next() {
                        return translation.to_owned();
                    }
                }
            }
        }
    }

    String::default()
}

#[cfg(test)]
mod test {
    use super::{parse_json_object, read_translation_file};

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
            parse_json_object(
                read_translation_file(reader, "advancement.advancementNotFound").unwrap(),
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
            parse_json_object(
                read_translation_file(reader, "advancement.advancementNotFound").unwrap(),
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
            parse_json_object(
                read_translation_file(reader, "advancement.advancementNotFound").unwrap(),
                "advancement.advancementNotFound"
            )
        );
    }
}
