use crate::lang::scala2::IozhError;
use crate::lang::scala2::GenResult;
use crate::lang::scala2::ProjectContext;

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::collections::HashMap;
use itertools::Itertools;
use crate::parser as p;


pub trait ResultVec {
    fn join(&self, sep: &str) -> String;
    fn to_string(&self) -> String;
}

pub trait ResultExt<T> {
    fn to_iozh(self) -> Result<T, IozhError>;
}


impl <T> ResultExt<T> for std::io::Result<T> {
    fn to_iozh(self) -> Result<T, IozhError> {
        self.map_err(|e| IozhError {
            pos: p::Pos { line: 0, col: 0},
            msg: format!("Failed to write file or dir: {}", e),
        })
    }
}

impl ResultVec for Vec<GenResult> {
    fn join(&self, sep: &str) -> String {
        self.iter().map(|x| x.content.clone()).collect::<Vec<_>>().join(sep)
    }
    fn to_string(&self) -> String {
        self.join("")
    }
}

impl GenResult {
    pub fn empty() -> Result<Vec<GenResult>, IozhError> {
        Ok(vec![])
    }
    pub fn single(content: String) -> Result<Vec<GenResult>, IozhError> {
        Ok(vec![
            GenResult {
                file: None,
                content: content,
                imports: vec![],
                package: vec![],
                block: None,
            }
        ])
    }
}

pub trait FileWriter {
    fn put(& mut self, content: &str) -> std::result::Result<(), IozhError>;
    fn putln(& mut self, content: &str) -> std::result::Result<(), IozhError>;
    fn putlnln(& mut self, content: &str) -> std::result::Result<(), IozhError>;
    fn ln(& mut self) -> std::result::Result<(), IozhError>;
}

impl FileWriter for std::fs::File {
    fn put(& mut self, content: &str) -> std::result::Result<(), IozhError> {
        self.write_all(content.as_bytes())
            .map_err(|e| IozhError {
                pos: p::Pos { line: 0, col: 0},
                msg: format!("Failed to write file: {}", e),
            })
    }
    fn ln(& mut self) -> std::result::Result<(), IozhError> {
        self.write_all("\n".as_bytes()).map_err(|e| IozhError {
            pos: p::Pos { line: 0, col: 0},
            msg: format!("Failed to write file: {}", e),
        })
    }
    fn putln(& mut self, content: &str) -> std::result::Result<(), IozhError> {
        self.put(&content)?;
        self.ln()
    }
    fn putlnln(& mut self, content: &str) -> std::result::Result<(), IozhError> {
        self.put(&content)?;
        self.ln()?;
        self.ln()
    }
}

pub fn group(items: Vec<GenResult>) -> Vec<GenResult> {
    let mut m = HashMap::<String, GenResult>::new();
    for mut item in items {
        let mut key: String = item.file.iter().map(|x| x.to_string_lossy().to_string()).join("");
        key.push_str(&item.block.iter().map(|x| x.to_string()).join(""));
        if let Some(existing) = m.get_mut(&key) {
            existing.content.push_str("\n");
            existing.content.push_str(&item.content);
            existing.imports.append(&mut item.imports);
        } else {
            m.insert(key, item);
        }
    }
    m.into_iter()
        .map(|(_, mut v)| {
            let imports = v.imports.clone().into_iter().sorted().unique();
            v.imports.clear();
            v.imports.extend(imports);
            v
        })
        .collect::<Vec<_>>()
}

pub fn write_fs_tree(items: Vec<GenResult>, project: &ProjectContext) -> std::result::Result<(), IozhError> {
    let grouped_items = group(items);
    for item in grouped_items {
        let rel_package_path = item.package
            .iter()
            .fold(PathBuf::new(), |mut acc, string| {
                acc.push(string);
                acc
            });
        let abs_package_path = project.target_folder.join(rel_package_path);
        fs::create_dir_all(abs_package_path).to_iozh()?;
        if let Some(file) = &item.file {
            let mut file = fs::File::create(file).to_iozh()?;
            file.putlnln(&format!("package {}", item.package.join(".")))?;
            if !item.imports.is_empty() {
                for import in &item.imports {
                    file.putln(&format!("import {}", import))?;
                }
                file.ln()?;
            }
            if let Some(block) = &item.block {
                file.putln(&format!("{block} {{"))?;
                file.putln(&item.content)?;
                file.putln(&format!("}}"))?;
            } else {
                file.put(&item.content)?;
            }
        }
    }
    Ok(())
}

pub fn imports_for(type_name: &str) -> Vec<String> {
    match type_name {
        "Instant" => vec!["java.time.Instant".to_string()],
        "Duration" => vec!["scala.concurrent.duration.Duration".to_string()],
        "FiniteDuration" => vec!["scala.concurrent.duration.FiniteDuration".to_string()],
        "File" => vec!["java.io.File".to_string()],
        _ => vec![],
    }
}

pub fn map_type(name: &str) -> &str {
    match name {
        "I32" => "Int",
        "I64" => "Long",
        "F32" => "Float",
        "F64" => "Double",
        "Bool" => "Boolean",
        "DateTime" => "Instant",
        "Duration" => "Duration",
        x => x,
    }
}

pub fn fs_sanitize(name: &str) -> String {
  let invalid_chars = "\\/:?\"<>|*";
  let safe_sym = '_';
  name.chars()
      .map(|c| if invalid_chars.contains(c) { safe_sym } else { c })
      .collect()
}

pub fn gen_filename(name: &str) -> String {
    format!("{}.scala", fs_sanitize(name))
}

pub fn sanitize(name: &str) -> String {
    let keywords = [
        "abstract",
        "case",
        "catch",
        "class",
        "def",
        "do",
        "else",
        "extends",
        "false",
        "final",
        "finally",
        "for",
        "forSome",
        "if",
        "implicit",
        "import",
        "lazy",
        "match",
        "new",
        "null",
        "object",
        "override",
        "package",
        "private",
        "protected",
        "return",
        "sealed",
        "super",
        "this",
        "throw",
        "trait",
        "try",
        "true",
        "type",
        "val",
        "var",
        "while",
        "with",
        "yield",
    ];

    let mut chars = name.chars();
    let is_valid_start = |c: char| c.is_alphabetic() || c == '_';
    let is_valid_char = |c: char| c.is_alphanumeric() || c == '_';

    if let Some(c) = chars.next() {
        if !is_valid_start(c) {
            return format!("`{}`", name);
        }

        for c in chars {
            if !is_valid_char(c) {
                return format!("`{}`", name);
            }
        }

        if keywords.contains(&name) {
            return format!("`{}`", name);
        }

        name.to_string()
    } else {
        format!("`{}`", name)
    }
}