use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;
use itertools::Itertools;
use iozh_parse::ast;
use iozh_parse::error::IozhError;
use crate::utils::*;
use crate::context::*;
use crate::gen_circe::*;

#[derive(Debug)]
pub struct GenResult {
    pub unit: Option<String>,
    pub content: String,
    pub imports: Vec<String>,
    pub package: Vec<String>,
    pub block: Option<String>,
}

impl GenResult {
    pub fn empty() -> Result<Vec<GenResult>, IozhError> {
        Ok(vec![])
    }
    pub fn single(content: String) -> Result<Vec<GenResult>, IozhError> {
        Ok(vec![
            GenResult {
                unit: None,
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
                pos: ast::Pos { line: 0, col: 0},
                msg: format!("Failed to write file: {}", e),
            })
    }
    fn ln(& mut self) -> std::result::Result<(), IozhError> {
        self.write_all("\n".as_bytes()).map_err(|e| IozhError {
            pos: ast::Pos { line: 0, col: 0},
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
        let mut key: String = item.unit.clone().unwrap_or_else(|| "".to_string());
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

pub fn write_fs_tree(items: Vec<GenResult>, target_folder: &Path) -> std::result::Result<(), IozhError> {
    let grouped_items = group(items);
    for item in grouped_items {
        let rel_package_path = item.package
            .iter()
            .fold(PathBuf::new(), |mut acc, package| {
                acc.push(fs_sanitize(package));
                acc
            });
        let abs_package_path = target_folder.join(rel_package_path);
        fs::create_dir_all(&abs_package_path).to_iozh()?;
        if let Some(unit) = &item.unit {
            let file_name = gen_filename(unit);
            let file_path = abs_package_path.as_path().join(file_name);
            let mut file = fs::File::create(abs_package_path.join(file_path)).to_iozh()?;
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
pub trait GenItems<A> {
    fn mapg<G>(&self, g: G) -> Result<Vec<GenResult>, IozhError>
    where
        G: Fn(&A) -> Result<Vec<GenResult>, IozhError>;

    fn filter_gen<F, G>(&self, f: F, g: G) -> Result<Vec<GenResult>, IozhError>
    where
        F: Fn(&A) -> bool,
        G: Fn(&A) -> Result<Vec<GenResult>, IozhError>;
}

impl <A> GenItems<A> for Vec<A> {
    fn mapg<G>(&self, g: G) -> Result<Vec<GenResult>, IozhError>
    where
        G: Fn(&A) -> Result<Vec<GenResult>, IozhError>
    {
        Ok(self
            .iter()
            .map(|x| g(x))
            .collect::<Result<Vec<Vec<GenResult>>, IozhError>>()
            .map(|vec| vec.into_iter().flatten())?
            .collect::<Vec<_>>())
    }

    fn filter_gen<F, G>(&self, f: F, g: G) -> Result<Vec<GenResult>, IozhError>
    where
        F: Fn(&A) -> bool,
        G: Fn(&A) -> Result<Vec<GenResult>, IozhError>
    {
        Ok(self
            .iter()
            .filter(|x| f(x))
            .map(|x| g(x))
            .collect::<Result<Vec<Vec<GenResult>>, IozhError>>()
            .map(|vec| vec.into_iter().flatten())?
            .collect::<Vec<_>>())
    }
}

pub trait GenResults {
    fn map_content(&self) -> Vec<String>;
    fn map_imports(&self) -> Vec<String>;
}

impl GenResults for Vec<GenResult> {
    fn map_content(&self) -> Vec<String> {
        self.iter().map(|x| x.content.clone()).collect::<Vec<_>>()
    }
    fn map_imports(&self) -> Vec<String> {
        self.iter().map(|x| x.imports.clone()).flatten().collect::<Vec<_>>()
    }
}

impl <'a> ProjectContext<'a> {
    pub fn push_nspace(&self, nspace: &ast::Nspace) -> NspaceContext {
        let nspace_name = &nspace.name;
        NspaceContext {
            project: self,
            path: vec![ nspace_name.to_string() ],
        }
    }
}

impl <'a> NspaceContext<'a> {
    pub fn push_nspace(&self, nspace: &ast::Nspace) -> NspaceContext {
        let nspace_name = &nspace.name;
        let mut nspace = self.path.clone();
        nspace.push(nspace_name.to_string());
        NspaceContext {
            project: self.project.clone(),
            path: nspace,
        }
    }
    pub fn push_struct(&self, s: &ast::Structure) -> Result<StructContext, IozhError> {
        let base_name = sanitize(&s.name.name);
        let full_type_name = s.name.gen()?.to_string();
        let type_args = gen_type_args(&s.name.args)?;
        Ok(StructContext {
            nspace: self,
            base_name,
            full_type_name,
            type_args,
        })
    }
    pub fn push_choice(&'a self, c: &'a ast::Choice) -> Result<ChoiceContext, IozhError> {
        let base_name = sanitize(&c.name.name);
        let full_type_name = c.name.gen()?.to_string();
        let tag_opt = c.get_most_common_tag_key(&self.project.p);
        Ok(ChoiceContext {
            nspace: self,
            p: c,
            base_name,
            full_type_name,
            most_common_tag_key: tag_opt,
        })
    }
    pub fn push_service(&self, s: &ast::Service) -> Result<ServiceContext, IozhError> {
        let base_name = sanitize(&s.name.name);
        let full_type_name = s.name.gen()?.to_string();
        Ok(ServiceContext {
            nspace: self,
            base_name,
            full_type_name,
        })
    }
    pub fn push_http_service(&self, s: &ast::HttpService) -> Result<HttpServiceContext, IozhError> {
        let base_name = sanitize(&s.name.name);
        let full_type_name = s.name.gen()?.to_string();
        Ok(HttpServiceContext {
            nspace: self,
            base_name,
            full_type_name,
        })
    }
}

impl <'a> ChoiceContext<'a> {
    pub fn push_struct(&self, s: &ast::Structure) -> Result<StructContext, IozhError> {
        let base_name = sanitize(&s.name.name);
        let full_type_name = s.name.gen()?.to_string();
        let type_args = gen_type_args(&s.name.args)?;
        Ok(StructContext {
            nspace: self.nspace.clone(),
            base_name,
            full_type_name,
            type_args,
        })
    }
}

impl <'a> ServiceContext<'a> {
    pub fn push_method(&self, m: &ast::Method) -> MethodContext {
        let method_name = m.name.name.clone();
        MethodContext {
            name: method_name,
        }
    }
}

fn gen_type_args(type_args: &Vec<ast::TypePath>) -> Result<Vec<String>, IozhError> {
    Ok(type_args.mapg(|x| x.gen())?.map_content())
}

impl Gen for ast::Literal {
    fn gen(&self) -> Result<Vec<GenResult>, IozhError> {
        match self {
            ast::Literal::String{ pos: _, value} => GenResult::single(format!("\"{}\"", value)),
            ast::Literal::Int{ pos: _, value } => GenResult::single(format!("{}", value)),
            ast::Literal::Nil => GenResult::single("None".to_string()),
        }
    }
}

impl InChoice for ast::Literal {
    fn gen_in_choice(&self, _parent: &ChoiceContext) -> Result<Vec<GenResult>, IozhError> {
        match self {
            ast::Literal::String{ pos: _, value} => GenResult::single(format!("def getValue = {}", value)),
            ast::Literal::Int{ pos: _, value } => GenResult::single(format!("def getValue = {}", value)),
            ast::Literal::Nil => GenResult::single("def getValue = ???".to_string()),
        }
    }
}

impl Gen for ast::TypeTag {
    fn gen(&self) -> Result<Vec<GenResult>, IozhError> {
        let args = gen_type_args(&self.args)?.join(",");
        let name = map_type(sanitize(&self.name).as_str()).to_string();
        if args.len() == 0 {
            GenResult::single(format!("{}", name))
        } else {
            GenResult::single(format!("{}[{}]", name, args))
        }
    }
}

impl Gen for ast::TypePath {
    fn gen(&self) -> Result<Vec<GenResult>, IozhError> {
        let path = self.path.mapg(|x| x.gen())?.map_content().join(".");
        GenResult::single(format!("{}", path))
    }
}

impl InChoice for ast::ChoiceItem {
    fn gen_in_choice(&self, parent: &ChoiceContext) -> Result<Vec<GenResult>, IozhError> {
        match self {
            ast::ChoiceItem::Structure(idx) => {
                let s = parent.nspace.project.p.get_structure(*idx)?;
                s.gen_in_choice(&parent)
            }
            ast::ChoiceItem::TypeTag { doc: _, choice } => {
                let choice_content = choice.gen()?.map_content().join("\n");
                GenResult::single(format!("case object {} extends {}", choice_content, parent.base_name))
            }
            ast::ChoiceItem::Value { doc: _, name, value } => {
                let name_content = name.gen()?.map_content().join("\n");
                let value_content = value.gen_in_choice(&parent)?.map_content().join("\n");
                GenResult::single(format!("case object {} extends {} {{\n{}\n}}", name_content, parent.base_name, value_content))
            }
            ast::ChoiceItem::Wrap { doc: _, name, field, target } => {
                let nn = &name.name;
                let targetn = target.gen()?.to_string();
                let content = format!("case class {nn}({field}: {targetn}) extends {}", parent.base_name);
                Ok(vec![GenResult {
                    unit: None,
                    content: content,
                    imports: imports_for(&targetn),
                    package: vec![],
                    block: None,
                }])
            }
            ast::ChoiceItem::Nil => GenResult::single("".to_string()),
        }
    }
}

impl InNspace for ast::Choice {
    fn gen_in_nspace(&self, parent: &NspaceContext) -> Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_choice(self)?;
        let result = self.choices
            .filter_gen(|x| match x {
                ast::ChoiceItem::Structure(_) => true,
                ast::ChoiceItem::Value { doc: _, name: _, value: _ } => true,
                ast::ChoiceItem::TypeTag { doc: _, choice: _ } => true,
                ast::ChoiceItem::Wrap { doc: _, name: _, field: _, target: _ } => true,
                ast::ChoiceItem::Nil => false,
            }, |x| x.gen_in_choice(&scope))?;
        let imports = result.map_imports();
        let items = result.map_content().join("\n");
        let fields = self.fields
            .mapg(|x| x.gen_in_choice(&scope))?
            .map_content()
            .iter()
            .map(|f| format!("def {f}"))
            .collect::<Vec<_>>()
            .join("\n");
        let header = if fields.len() > 0 {
            format!("sealed trait {} {{ {fields} }}", scope.full_type_name)
        } else {
            format!("sealed trait {}", scope.full_type_name)
        };
        let body = format!("object {} {{ {} }}", scope.base_name, items);
        let unit = Some(scope.base_name.clone());
        let mut circe_codecs = self.codec_in_nspace(parent)?;
        circe_codecs.iter_mut().for_each(|m| m.imports.append(&mut imports.clone()));
        circe_codecs.push(
            GenResult {
                unit,
                content: format!("{}\n{}", header, body),
                imports,
                package: scope.nspace.path.clone(),
                block: None,
            }
        );
        Ok(circe_codecs)
    }
}

impl InChoice for ast::Field {
    fn gen_in_choice(&self, _parent: &ChoiceContext) -> Result<Vec<GenResult>, IozhError> {
        let tp = self.type_path.gen()?.to_string();
        Ok(vec![
            GenResult {
                unit: None,
                content: format!("{}: {}", self.name, tp),
                imports: imports_for(&tp),
                package: vec![],
                block: None,
            }
        ])
    }
}

impl InStruct for ast::Field {
    fn gen_in_struct(&self, _parent: &StructContext) -> Result<Vec<GenResult>, IozhError> {
        let tp = self.type_path.gen()?.to_string();
        Ok(vec![
            GenResult {
                unit: None,
                content: format!("{}: {}", sanitize(&self.name), tp),
                imports: imports_for(&tp),
                package: vec![],
                block: None,
            }
        ])
    }
}

impl InMethod for ast::Field {
    fn gen_in_method(&self, _parent: &MethodContext) -> Result<Vec<GenResult>, IozhError> {
        let tp = self.type_path.gen()?.to_string();
        GenResult::single(format!("{}: {}", self.name, tp))
    }
}

impl InStruct for ast::StructItem {
    fn gen_in_struct(&self, parent: &StructContext) -> Result<Vec<GenResult>, IozhError> {
        match self {
            ast::StructItem::Field(v) => v.gen_in_struct(parent),
            ast::StructItem::Tag(_v) => GenResult::empty()
        }
    }
}

impl InChoice for ast::Structure {
    fn gen_in_choice(&self, parent: &ChoiceContext) -> Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_struct(self)?;
        let mut result = self.fields
            .filter_gen(|x| match x {
                ast::StructItem::Field(_) => true,
                _ => false,
            }, |x| x.gen_in_struct(&scope))?;
        let imports = result.map_imports();
        let mut inherited_fields = parent.p.fields.mapg(|x| x.gen_in_struct(&scope))?;
        result.append(&mut inherited_fields);
        let fields = result.map_content().join(",");
        let content = if fields.len() > 0 {
            format!("case class {}({fields}) extends {}", scope.full_type_name, parent.full_type_name)
        } else {
            format!("case object {} extends {}", scope.full_type_name, parent.full_type_name)
        };
        Ok(vec![
            GenResult {
                unit: None,
                content,
                imports,
                package: scope.nspace.path.clone(),
                block: None,
            }
        ])
    }
}

impl InNspace for ast::Structure {
    fn gen_in_nspace(&self, parent: &NspaceContext) -> Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_struct(self)?;
        let result = self.fields
            .filter_gen(|x|
                match x {
                    ast::StructItem::Field(_) => true,
                    _ => false,
                },
                |x| x.gen_in_struct(&scope)
            )?;
        let imports = result.map_imports();
        let fields = result.map_content().join(",");
        let unit = Some(scope.base_name.clone());
        let mut circe_codecs = self.codec_in_nspace(parent)?;
        circe_codecs.iter_mut().for_each(|m| m.imports.append(&mut imports.clone()));
        circe_codecs.push(
            GenResult {
                unit,
                content: format!("case class {}({fields})", scope.full_type_name),
                imports,
                package: scope.nspace.path.clone(),
                block: None,
            }
        );
        Ok(circe_codecs)
    }
}

impl InService for ast::Method {
    fn gen_in_service(&self, parent: &ServiceContext) -> Result<Vec<GenResult>, IozhError> {
        let name = self.name.gen()?.to_string();
        let scope = parent.push_method(self);
        let args = self.args.mapg(|x| x.gen_in_method(&scope))?.map_content().join("\n");
        let ret = self.result.gen()?.to_string();
        GenResult::single(format!("def {}({}): {}", name, args, ret))
    }
}

impl InNspace for ast::Service {
    fn gen_in_nspace(&self, parent: &NspaceContext) -> Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_service(self)?;
        let methods_results = self.methods.mapg(|x| x.gen_in_service(&scope))?;
        let imports = methods_results.map_imports();
        let methods = methods_results.map_content().join("\n");
        let content = format!("trait {} {{\n{}\n}}", scope.full_type_name, methods);
        let unit = Some(scope.base_name.clone());
        Ok(vec![
            GenResult {
                unit,
                content: content,
                imports,
                package: scope.nspace.path.clone(),
                block: None,
            }
        ])
    }
}

impl InHttpService for ast::HttpRoute {
    fn gen_in_http_service(&self, _parent: &HttpServiceContext) -> Result<Vec<GenResult>, IozhError> {
        Ok(vec![
        ])
    }
}

impl InNspace for ast::HttpService {
    fn gen_in_nspace(&self, parent: &NspaceContext) -> Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_http_service(self)?;
        let methods = self.routes
            .mapg(|x| x.gen_in_http_service(&scope))?
            .map_content()
            .join("\n");
        let header = format!("trait {}", scope.full_type_name);
        let body = format!("object {} {{ {} }}", scope.base_name, methods);
        GenResult::single(format!("{}\n{}", header, body))
    }
}

impl InNspace for ast::NspaceItem {
    fn gen_in_nspace(&self, parent: &NspaceContext) -> Result<Vec<GenResult>, IozhError> {
        match self {
            ast::NspaceItem::Structure(idx) => {
                let s = parent.project.p.get_structure(*idx)?;
                s.gen_in_nspace(parent)
            }
            ast::NspaceItem::Choice(idx) => {
                let c = parent.project.p.get_choice(*idx)?;
                c.gen_in_nspace(parent)
            }
            ast::NspaceItem::Service(v) => v.gen_in_nspace(parent),
            ast::NspaceItem::HttpService(v) => v.gen_in_nspace(parent),
            ast::NspaceItem::Nspace(v) => v.gen_in_nspace(parent),
        }
    }
}

impl InNspace for ast::Nspace {
    fn gen_in_nspace(&self, parent: &NspaceContext) -> Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_nspace(self);
        self.items.mapg(|x| x.gen_in_nspace(&scope))
    }
}

impl InProject for ast::Nspace {
    fn gen_in_project(&self, parent: &ProjectContext) -> Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_nspace(&self);
        self.items.mapg(|x| x.gen_in_nspace(&scope))
    }
}

pub fn generate(project: ast::Project, target_folder: &std::path::Path) -> Result<(), IozhError> {
    let scope = ProjectContext { p: &project };
    let mut items = project.nspaces.mapg(|x| x.gen_in_project(&scope))?;
    let mut circe_items = circe_pack(&scope)?;
    items.append(&mut circe_items);
    write_fs_tree(items, target_folder)
}