use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use serde_json::Value;

#[derive(Debug)]
pub enum TranslationError {
    InvalidFile,
    FileRead,
    JsonParse,
}

pub fn translate(
    path: impl Into<PathBuf>,
    message: &str,
) -> Result<HashMap<String, String>, TranslationError> {
    let file = File::open(path.into()).map_err(|_| TranslationError::InvalidFile)?;
    let translations = fetch_translations(&file, message)?;
    let results = make_hashmap(translations);

    Ok(results)
}

//Read a huge object line by line and tricking serde_json into thinking they are individual objects
fn fetch_translations(file: &File, message: &str) -> Result<Vec<Value>, TranslationError> {
    let mut bufreader = BufReader::new(file);
    let mut buf = String::new();
    let mut results = Vec::new();

    loop {
        let bytes_read = bufreader
            .read_line(&mut buf)
            .map_err(|_| TranslationError::FileRead)?;

        if bytes_read == 0 {
            break;
        }

        if buf == "{" || buf == "}" {
            continue;
        } else if buf.contains(message) {
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
            let text = map.keys().next().unwrap_or(&String::default()).to_owned();
            if let Value::String(translation) = map
                .values()
                .next()
                .unwrap_or(&Value::String(String::default()))
            {
                hashmap.insert(text, translation.to_owned());
            }
        }
    }

    hashmap
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::translate;

    #[test]
    fn test_lang_it_it() {
        let path = "C:/Users/ADMIN/AppData/Roaming/.minecraft/assets/objects/67/6793f17e4febc8801da3877f2a058c1ef77c338a";
        let intended_result = HashMap::from([(
            "advancement.advancementNotFound".to_owned(),
            "Progresso sconosciuto: %s".to_owned(),
        )]);

        assert_eq!(
            intended_result,
            translate(path, "advancement.advancementNotFound").unwrap()
        );
    }

    #[test]
    fn test_lang_ja_jp() {
        let path = "C:/Users/ADMIN/AppData/Roaming/.minecraft/assets/objects/09/09dcd0a0f7313920d433da26b26d9bb451f24594";
        let intended_result = HashMap::from([
            (
                "advancement.advancementNotFound".to_owned(),
                "\u{4e0d}\u{660e}\u{306a}\u{9032}\u{6357}\u{3067}\u{3059}\u{ff1a}%s".to_owned(),
            ),
            (
                "commands.advancement.advancementNotFound".to_owned(),
                "「%s」という名前の進捗は見つかりませんでした".to_owned(),
            ),
        ]);

        assert_eq!(
            intended_result,
            translate(path, "advancement.advancementNotFound").unwrap()
        );
    }
}
