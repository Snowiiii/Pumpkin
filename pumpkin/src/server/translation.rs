#![expect(dead_code)]

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

use pumpkin_config::TranslationConfig;
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TranslationError {
    #[error("Translations not turned on")]
    NotEnabled,
    #[error("File cannot be opened.")]
    InvalidFile,
    #[error("Failed to read file. Error: {0}")]
    FileRead(std::io::Error),
    #[error("Invalid JSON encountered. Use only objects in the translation file.")]
    JsonParse,
}

pub fn translate(
    config: TranslationConfig,
    message: &str,
) -> Result<HashMap<String, String>, TranslationError> {
    if !config.enabled {
        return Err(TranslationError::NotEnabled);
    }

    if config.client_translations {
        return Ok(HashMap::from([(message.to_owned(), message.to_owned())]));
    }

    if let Some(path) = config.translation_file_path {
        let file = File::open(path).map_err(|_| TranslationError::InvalidFile)?;
        let mut bufreader = BufReader::new(file);

        let translations = fetch_translations(&mut bufreader, message)?;
        let results = make_hashmap(translations);

        Ok(results)
    } else {
        Err(TranslationError::InvalidFile)
    }
}

///Read a huge object line by line and tricking `serde_json` into thinking they are individual objects
fn fetch_translations(
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

fn make_hashmap(vec: Vec<Value>) -> HashMap<String, String> {
    let mut hashmap: HashMap<String, String> = HashMap::new();

    for value in vec {
        if let Value::Object(map) = value {
            if let Some(text) = map.keys().next() {
                if let Some(Value::String(translation)) = map.values().next() {
                    hashmap.insert(text.to_owned(), translation.to_owned());
                }
            }
        }
    }

    hashmap
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::{fetch_translations, make_hashmap};

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

        let intended_result = HashMap::from([
            (
                "commands.advancement.advancementNotFound".to_owned(),
                "「%s」という名前の進捗は見つかりませんでした".to_owned(),
            ),
            (
                "advancement.advancementNotFound".to_owned(),
                "\u{4e0d}\u{660e}\u{306a}\u{9032}\u{6357}\u{3067}\u{3059}\u{ff1a}%s".to_owned(),
            ),
        ]);

        assert_eq!(
            intended_result,
            make_hashmap(fetch_translations(reader, "advancement.advancementNotFound").unwrap())
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

        let intended_result = HashMap::from([(
            "advancement.advancementNotFound".to_owned(),
            "Progresso sconosciuto: %s".to_owned(),
        )]);

        assert_eq!(
            intended_result,
            make_hashmap(fetch_translations(reader, "advancement.advancementNotFound").unwrap())
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

        let intended_result = HashMap::new();

        assert_eq!(
            intended_result,
            make_hashmap(fetch_translations(reader, "advancement.advancementNotFound").unwrap())
        );
    }
}
