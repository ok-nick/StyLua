use anyhow::{bail, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use stylua_lib::{Config, IndentType, LineEndings};

#[derive(Debug, Deserialize)]
#[serde(remote = "LineEndings")]
enum LineEndingsDef {
    #[serde(rename = "lf")]
    Unix,
    #[serde(rename = "crlf")]
    Windows,
}

#[derive(Debug, Deserialize)]
#[serde(remote = "IndentType")]
enum IndentTypeDef {
    #[serde(rename = "tab")]
    Tabs,
    #[serde(rename = "space")]
    Spaces,
}

#[derive(Debug, Deserialize)]
struct ConfigDef {
    #[serde(rename = "end_of_line", with = "LineEndingsDef", default)]
    line_endings: LineEndings,
    #[serde(rename = "indent_style", with = "IndentTypeDef", default)]
    indent_type: IndentType,
    #[serde(rename = "indent_size", default)]
    indent_width: usize,
}

#[derive(Debug, Deserialize)]
struct Section {
    #[serde(rename = "*", alias = "*.lua")]
    lua: ConfigDef,
}

pub fn from_toml(content: &str) -> Result<Config> {
    match toml::from_str(&content) {
        Ok(config) => Ok(config),
        Err(error) => bail!("error: config file not in correct format: {}", error),
    }
}

pub fn from_ini(content: &str) -> Result<Config> {
    let section: Section = match serde_ini::from_str(content) {
        Ok(section) => section,
        Err(error) => bail!("error: config file not in correct format: {}", error),
    };

    let config = section.lua;
    Ok(Config::new()
        .with_line_endings(config.line_endings)
        .with_indent_type(config.indent_type)
        .with_indent_width(config.indent_width))
}

pub fn read() -> Result<Config> {
    match fs::read_to_string("stylua.toml") {
        Ok(content) => from_toml(&content),
        Err(_) => match fs::read_to_string(".editorconfig") {
            Ok(content) => from_ini(&content),
            Err(_) => Ok(Config::default()),
        },
    }
}

pub fn read_from_path(path: &Path) -> Result<Config> {
    match fs::read_to_string(path) {
        Ok(content) => from_toml(&content),
        Err(error) => bail!("error: couldn't read config file: {}", error),
    }
}
