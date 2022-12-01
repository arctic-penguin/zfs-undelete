use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

#[derive(Debug)]
pub(super) struct ConfigParser {
    key_value_pairs: HashMap<String, String>,
}

impl ConfigParser {
    pub fn get_value_into(&self, key: &str, field: &mut String) {
        if let Some(value) = self.key_value_pairs.get(key) {
            *field = value.to_owned();
        }
    }

    pub fn get_values_into(&self, key: &str, field: &mut Vec<String>) {
        if let Some(value) = self.key_value_pairs.get(key) {
            field.clear();
            for item in value.split_whitespace() {
                field.push(item.trim().to_owned());
            }
        }
    }
}

impl TryFrom<PathBuf> for ConfigParser {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path).context("reading config file")?;

        let mut key_value_pairs = HashMap::new();

        for line in content.lines() {
            let before_comment = line.split('#').next().unwrap().trim();
            if before_comment.is_empty() {
                continue;
            }

            if before_comment.contains('=') {
                extract_key_value_pair(before_comment, &mut key_value_pairs)?;
            }
        }

        Ok(Self { key_value_pairs })
    }
}

/// From a line in the config, extract the items like `<key>=<value>`.
fn extract_key_value_pair(line: &str, pairs: &mut HashMap<String, String>) -> Result<()> {
    let (key, value) = line
        .split_once('=')
        .with_context(|| format!("separating key and value in '{line}'"))?;
    let key = key.trim();
    let value = value.trim();
    if pairs.insert(key.to_owned(), value.to_owned()).is_some() {
        bail!("duplicate key in config: {key}")
    }
    Ok(())
}
