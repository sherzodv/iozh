use crate::parser as p;
use crate::lang::scala2::*;
use crate::lang::scala2::utils::*;
use crate::lang::scala2::gen_circe::*;

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

impl ProjectContext {
    pub fn push_nspace(&self, nspace: &p::Nspace) -> NspaceContext {
        let nspace_name = &fs_sanitize(&nspace.name);
        let folder = self.target_folder.join(nspace_name);
        NspaceContext {
            project: self.clone(),
            folder,
            path: vec![ nspace_name.to_string() ],
        }
    }
}

impl NspaceContext {
    pub fn push_nspace(&self, nspace: &p::Nspace) -> NspaceContext {
        let nspace_name = fs_sanitize(&nspace.name);
        let mut nspace = self.path.clone();
        nspace.push(nspace_name.to_string());
        let folder = self.folder.join(nspace_name);
        NspaceContext {
            project: self.project.clone(),
            folder,
            path: nspace,
        }
    }
    pub fn push_struct(&self, s: &p::Structure) -> Result<StructContext, IozhError> {
        let base_name = sanitize(&s.name.name);
        let full_type_name = s.name.gen()?.to_string();
        let type_args = gen_type_args(&s.name.args)?;
        Ok(StructContext {
            nspace: self.clone(),
            base_name,
            full_type_name,
            type_args,
        })
    }
    pub fn push_choice(&self, c: &p::Choice) -> Result<ChoiceContext, IozhError> {
        let base_name = sanitize(&c.name.name);
        let full_type_name = c.name.gen()?.to_string();
        let tag_opt = c.get_most_common_tag_key();
        Ok(ChoiceContext {
            nspace: self.clone(),
            base_name,
            full_type_name,
            most_common_tag_key: tag_opt,
        })
    }
    pub fn push_service(&self, s: &p::Service) -> Result<ServiceContext, IozhError> {
        let base_name = sanitize(&s.name.name);
        let full_type_name = s.name.gen()?.to_string();
        Ok(ServiceContext {
            nspace: self.clone(),
            base_name,
            full_type_name,
        })
    }
    pub fn push_http_service(&self, s: &p::HttpService) -> Result<HttpServiceContext, IozhError> {
        let base_name = sanitize(&s.name.name);
        let full_type_name = s.name.gen()?.to_string();
        Ok(HttpServiceContext {
            nspace: self.clone(),
            base_name,
            full_type_name,
        })
    }
}

impl ChoiceContext {
    pub fn push_struct(&self, s: &p::Structure) -> Result<StructContext, IozhError> {
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

impl ServiceContext {
    pub fn push_method(&self, m: &p::Method) -> MethodContext {
        let method_name = fs_sanitize(&m.name.name);
        MethodContext {
            service: self.clone(),
            name: method_name,
        }
    }
}

fn gen_type_args(type_args: &Vec<p::TypeTag>) -> Result<Vec<String>, IozhError> {
    Ok(type_args.mapg(|x| x.gen())?.map_content())
}

impl Gen for p::Literal {
    fn gen(&self) -> Result<Vec<GenResult>, IozhError> {
        match self {
            p::Literal::String{ pos: _, value} => GenResult::single(format!("\"{}\"", value)),
            p::Literal::Int{ pos: _, value } => GenResult::single(format!("{}", value)),
            p::Literal::Nil => GenResult::single("None".to_string()),
        }
    }
}

impl InChoice for p::Literal {
    fn gen_in_choice(&self, _parent: &ChoiceContext) -> Result<Vec<GenResult>, IozhError> {
        match self {
            p::Literal::String{ pos: _, value} => GenResult::single(format!("def getValue = {}", value)),
            p::Literal::Int{ pos: _, value } => GenResult::single(format!("def getValue = {}", value)),
            p::Literal::Nil => GenResult::single("def getValue = ???".to_string()),
        }
    }
}

impl Gen for p::TypeTag {
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

impl Gen for p::TypePath {
    fn gen(&self) -> Result<Vec<GenResult>, IozhError> {
        let path = self.path.mapg(|x| x.gen())?.map_content().join("\n");
        GenResult::single(format!("{}", path))
    }
}

impl InChoice for p::ChoiceItem {
    fn gen_in_choice(&self, parent: &ChoiceContext) -> Result<Vec<GenResult>, IozhError> {
        match self {
            p::ChoiceItem::Structure(v) => v.gen_in_choice(&parent),
            p::ChoiceItem::TypeTag { doc: _, choice } => {
                let choice_content = choice.gen()?.map_content().join("\n");
                GenResult::single(format!("case object {} extends {}", choice_content, parent.base_name))
            }
            p::ChoiceItem::Value { doc: _, name, value } => {
                let name_content = name.gen()?.map_content().join("\n");
                let value_content = value.gen_in_choice(&parent)?.map_content().join("\n");
                GenResult::single(format!("case object {} extends {} {{\n{}\n}}", name_content, parent.base_name, value_content))
            }
            p::ChoiceItem::Wrap { doc: _, name, field, target } => {
                let nn = &name.name;
                let targetn = target.gen()?.to_string();
                let content = format!("case class {nn}({field}: {targetn}) extends {}", parent.base_name);
                Ok(vec![GenResult {
                    file: None,
                    content: content,
                    imports: imports_for(&targetn),
                    package: vec![],
                    block: None,
                }])
            }
            p::ChoiceItem::Nil => GenResult::single("".to_string()),
        }
    }
}

impl InNspace for p::Choice {
    fn gen_in_nspace(&self, parent: &NspaceContext) -> Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_choice(self)?;
        let result = self.choices
            .filter_gen(|x| match x {
                p::ChoiceItem::Structure(_) => true,
                p::ChoiceItem::Value { doc: _, name: _, value: _ } => true,
                p::ChoiceItem::TypeTag { doc: _, choice: _ } => true,
                p::ChoiceItem::Wrap { doc: _, name: _, field: _, target: _ } => true,
                p::ChoiceItem::Nil => false,
            }, |x| x.gen_in_choice(&scope))?;
        let imports = result.map_imports();
        let items = result.map_content().join("\n");
        let header = format!("sealed trait {}", scope.full_type_name);
        let body = format!("object {} {{ {} }}", scope.base_name, items);
        let file_name = gen_filename(&scope.base_name);
        let file_path = parent.folder.join(file_name);
        let mut circe_codecs = self.codec_in_nspace(parent)?;
        circe_codecs.iter_mut().for_each(|m| m.imports.append(&mut imports.clone()));
        circe_codecs.push(
            GenResult {
                file: Some(file_path),
                content: format!("{}\n{}", header, body),
                imports,
                package: scope.nspace.path,
                block: None,
            }
        );
        Ok(circe_codecs)
    }
}

impl InStruct for p::Field {
    fn gen_in_struct(&self, _parent: &StructContext) -> Result<Vec<GenResult>, IozhError> {
        let tp = self.type_path.gen()?.to_string();
        Ok(vec![
            GenResult {
                file: None,
                content: format!("{}: {}", self.name, tp),
                imports: imports_for(&tp),
                package: vec![],
                block: None,
            }
        ])
    }
}

impl InMethod for p::Field {
    fn gen_in_method(&self, _parent: &MethodContext) -> Result<Vec<GenResult>, IozhError> {
        let tp = self.type_path.gen()?.to_string();
        GenResult::single(format!("{}: {}", self.name, tp))
    }
}

impl InStruct for p::StructItem {
    fn gen_in_struct(&self, parent: &StructContext) -> Result<Vec<GenResult>, IozhError> {
        match self {
            p::StructItem::Field(v) => v.gen_in_struct(parent),
            p::StructItem::Tag(_v) => GenResult::empty()
        }
    }
}

impl InChoice for p::Structure {
    fn gen_in_choice(&self, parent: &ChoiceContext) -> Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_struct(self)?;
        let result = self.fields
            .filter_gen(|x| match x {
                p::StructItem::Field(_) => true,
                _ => false,
            }, |x| x.gen_in_struct(&scope))?;
        let imports = result.map_imports();
        let fields = result.map_content().join(",");
        Ok(vec![
            GenResult {
                file: None,
                content: format!("case class {}({fields}) extends {}", scope.full_type_name, parent.full_type_name),
                imports,
                package: scope.nspace.path,
                block: None,
            }
        ])
    }
}

impl InNspace for p::Structure {
    fn gen_in_nspace(&self, parent: &NspaceContext) -> Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_struct(self)?;
        let result = self.fields
            .filter_gen(|x|
                match x {
                    p::StructItem::Field(_) => true,
                    _ => false,
                },
                |x| x.gen_in_struct(&scope)
            )?;
        let imports = result.map_imports();
        let fields = result.map_content().join(",");
        let file_name = gen_filename(&scope.base_name);
        let file_path = parent.folder.join(file_name);
        let mut circe_codecs = self.codec_in_nspace(parent)?;
        circe_codecs.iter_mut().for_each(|m| m.imports.append(&mut imports.clone()));
        circe_codecs.push(
            GenResult {
                file: Some(file_path),
                content: format!("case class {}({fields})", scope.full_type_name),
                imports,
                package: scope.nspace.path,
                block: None,
            }
        );
        Ok(circe_codecs)
    }
}

impl InService for p::Method {
    fn gen_in_service(&self, parent: &ServiceContext) -> Result<Vec<GenResult>, IozhError> {
        let name = self.name.gen()?.to_string();
        let scope = parent.push_method(self);
        let args = self.args.mapg(|x| x.gen_in_method(&scope))?.map_content().join("\n");
        let ret = self.result.gen()?.to_string();
        GenResult::single(format!("def {}({}): {}", name, args, ret))
    }
}

impl InNspace for p::Service {
    fn gen_in_nspace(&self, parent: &NspaceContext) -> Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_service(self)?;
        let methods_results = self.methods.mapg(|x| x.gen_in_service(&scope))?;
        let imports = methods_results.map_imports();
        let methods = methods_results.map_content().join("\n");
        let content = format!("trait {} {{\n{}\n}}", scope.full_type_name, methods);
        let file_name = gen_filename(&scope.base_name);
        let file_path = parent.folder.join(file_name);
        Ok(vec![
            GenResult {
                file: Some(file_path),
                content: content,
                imports,
                package: scope.nspace.path,
                block: None,
            }
        ])
    }
}

impl InHttpService for p::HttpRoute {
    fn gen_in_http_service(&self, _parent: &HttpServiceContext) -> Result<Vec<GenResult>, IozhError> {
        Ok(vec![
        ])
    }
}

impl InNspace for p::HttpService {
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

impl InNspace for p::NspaceItem {
    fn gen_in_nspace(&self, parent: &NspaceContext) -> Result<Vec<GenResult>, IozhError> {
        match self {
            p::NspaceItem::Structure(v) => v.gen_in_nspace(parent),
            p::NspaceItem::Choice(v) => v.gen_in_nspace(parent),
            p::NspaceItem::Service(v) => v.gen_in_nspace(parent),
            p::NspaceItem::HttpService(v) => v.gen_in_nspace(parent),
            p::NspaceItem::Nspace(v) => v.gen_in_nspace(parent),
        }
    }
}

impl InNspace for p::Nspace {
    fn gen_in_nspace(&self, parent: &NspaceContext) -> Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_nspace(self);
        self.items.mapg(|x| x.gen_in_nspace(&scope))
    }
}

impl InProject for p::Nspace {
    fn gen_in_project(&self, parent: &ProjectContext) -> Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_nspace(&self);
        self.items.mapg(|x| x.gen_in_nspace(&scope))
    }
}

impl p::Project {
    pub fn generate(&self, target_folder: &std::path::Path) -> Result<(), IozhError> {
        let scope = ProjectContext {
            target_folder: target_folder.to_path_buf(),
        };
        let mut items = self.nspaces.mapg(|x| x.gen_in_project(&scope))?;
        let mut circe_items = circe_pack(&scope)?;
        items.append(&mut circe_items);
        write_fs_tree(items, &scope)
    }
}