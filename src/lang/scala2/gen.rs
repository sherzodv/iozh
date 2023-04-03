use crate::parser as p;
use crate::lang::scala2::*;
use crate::lang::scala2::utils::*;

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
    pub fn push_struct(&self, s: &p::Structure) -> std::result::Result<StructContext, IozhError> {
        let base_name = sanitize(&s.name.name);
        let full_type_name = s.name.gen()?.to_string();
        Ok(StructContext {
            nspace: self.clone(),
            base_name,
            full_type_name,
        })
    }
    pub fn push_choice(&self, c: &p::Choice) -> std::result::Result<ChoiceContext, IozhError> {
        let base_name = sanitize(&c.name.name);
        let full_type_name = c.name.gen()?.to_string();
        Ok(ChoiceContext {
            nspace: self.clone(),
            base_name,
            full_type_name,
        })
    }
    pub fn push_service(&self, s: &p::Service) -> std::result::Result<ServiceContext, IozhError> {
        let base_name = sanitize(&s.name.name);
        let full_type_name = s.name.gen()?.to_string();
        Ok(ServiceContext {
            nspace: self.clone(),
            base_name,
            full_type_name,
        })
    }
    pub fn push_http_service(&self, s: &p::HttpService) -> std::result::Result<HttpServiceContext, IozhError> {
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
    pub fn push_struct(&self, s: &p::Structure) -> std::result::Result<StructContext, IozhError> {
        let base_name = sanitize(&s.name.name);
        let full_type_name = s.name.gen()?.to_string();
        Ok(StructContext {
            nspace: self.nspace.clone(),
            base_name,
            full_type_name,
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

impl Gen for p::Literal {
    fn gen(&self) -> std::result::Result<Vec<GenResult>, IozhError> {
        match self {
            p::Literal::String{ pos: _, value} => GenResult::single(format!("\"{}\"", value)),
            p::Literal::Int{ pos: _, value } => GenResult::single(format!("{}", value)),
            p::Literal::Nil => GenResult::single("None".to_string()),
        }
    }
}

impl InChoice for p::Literal {
    fn gen_in_choice(&self, _parent: &ChoiceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        match self {
            p::Literal::String{ pos: _, value} => GenResult::single(format!("def getValue = {}", value)),
            p::Literal::Int{ pos: _, value } => GenResult::single(format!("def getValue = {}", value)),
            p::Literal::Nil => GenResult::single("def getValue = ???".to_string()),
        }
    }
}

impl Gen for p::TypeTag {
    fn gen(&self) -> std::result::Result<Vec<GenResult>, IozhError> {
        let args = self
            .args
            .iter()
            .map(|x| x.gen())
            .collect::<Result<Vec<Vec<GenResult>>, IozhError>>()
            .map(|vec| vec.into_iter().flatten())?
            .map(|m| m.content)
            .collect::<Vec<_>>().join(",");
        let name = map_type(sanitize(&self.name).as_str()).to_string();
        if args.len() == 0 {
            GenResult::single(format!("{}", name))
        } else {
            GenResult::single(format!("{}[{}]", name, args))
        }
    }
}

impl Gen for p::TypePath {
    fn gen(&self) -> std::result::Result<Vec<GenResult>, IozhError> {
        let path = self
            .path
            .iter()
            .map(|x| x.gen())
            .collect::<Result<Vec<Vec<GenResult>>, IozhError>>()
            .map(|vec| vec.into_iter().flatten())?
            .map(|m| m.content)
            .collect::<Vec<_>>().join("\n");
        GenResult::single(format!("{}", path))
    }
}

impl InChoice for p::ChoiceItem {
    fn gen_in_choice(&self, parent: &ChoiceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        match self {
            p::ChoiceItem::Structure(v) => v.gen_in_choice(&parent),
            p::ChoiceItem::TypeTag { doc: _, choice } => {
                let choice_content = choice.gen()?.into_iter().map(|c| c.content).collect::<Vec<_>>().join("\n");
                GenResult::single(format!("case object {} extends {}", choice_content, parent.base_name))
            }
            p::ChoiceItem::Value { doc: _, name, value } => {
                let name_content = name.gen()?.into_iter().map(|c| c.content).collect::<Vec<_>>().join("\n");
                let value_content = value.gen_in_choice(&parent)?.into_iter().map(|c| c.content).collect::<Vec<_>>().join("\n");
                GenResult::single(format!("case object {} extends {} {{\n{}\n}}", name_content, parent.base_name, value_content))
            }
            p::ChoiceItem::Nil => GenResult::single("".to_string()),
        }
    }
}

impl InNspace for p::Choice {
    fn gen_in_nspace(&self, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_choice(self)?;
        let result = self
            .choices
            .iter()
            .filter(|x| match x {
                p::ChoiceItem::Structure(_) => true,
                p::ChoiceItem::Value { doc: _, name: _, value: _ } => true,
                p::ChoiceItem::TypeTag { doc: _, choice: _ } => true,
                p::ChoiceItem::Nil => false,
            })
            .map(|x| x.gen_in_choice(&scope))
            .collect::<Result<Vec<Vec<GenResult>>, IozhError>>()
            .map(|vec| vec.into_iter().flatten())?
            .collect::<Vec<_>>();
        let imports = result
            .iter()
            .map(|m| m.imports.clone())
            .flatten()
            .collect::<Vec<_>>();
        let items = result
            .into_iter()
            .map(|m| m.content)
            .collect::<Vec<_>>().join("\n");
        let header = format!("sealed trait {}", scope.full_type_name);
        let body = format!("object {} {{ {} }}", scope.base_name, items);
        let file_name = gen_filename(&scope.base_name);
        let file_path = parent.folder.join(file_name);
        Ok(vec![
            GenResult {
                file: Some(file_path),
                content: format!("{}\n{}", header, body),
                imports: imports,
                package: scope.nspace.path,
            }
        ])
    }
}

impl InStruct for p::Field {
    fn gen_in_struct(&self, _parent: &StructContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        let tp = self.type_path.gen()?.to_string();
        Ok(vec![
            GenResult {
                file: None,
                content: format!("{}: {}", self.name, tp),
                imports: imports_for(&tp),
                package: vec![],
            }
        ])
    }
}

impl InMethod for p::Field {
    fn gen_in_method(&self, _parent: &MethodContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        let tp = self.type_path.gen()?.to_string();
        GenResult::single(format!("{}: {}", self.name, tp))
    }
}

impl InStruct for p::StructItem {
    fn gen_in_struct(&self, parent: &StructContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        match self {
            p::StructItem::Field(v) => v.gen_in_struct(parent),
            p::StructItem::Tag(_v) => GenResult::empty()
        }
    }
}

impl InChoice for p::Structure {
    fn gen_in_choice(&self, parent: &ChoiceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_struct(self)?;
        let result = self
            .fields
            .iter()
            .filter(|x| match x {
                p::StructItem::Field(_) => true,
                _ => false,
            })
            .map(|x| x.gen_in_struct(&scope))
            .collect::<Result<Vec<Vec<GenResult>>, IozhError>>()
            .map(|vec| vec.into_iter().flatten())?
            .collect::<Vec<_>>();
        let imports = result
            .iter()
            .map(|m| m.imports.clone())
            .flatten()
            .collect::<Vec<_>>();
        let fields = result
            .into_iter()
            .map(|m| m.content)
            .collect::<Vec<_>>().join(",");
        Ok(vec![
            GenResult {
                file: None,
                content: format!("case class {}({fields}) extends {}", scope.full_type_name, parent.full_type_name),
                imports: imports,
                package: scope.nspace.path,
            }
        ])
    }
}

impl InNspace for p::Structure {
    fn gen_in_nspace(&self, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_struct(self)?;
        let result = self
            .fields
            .iter()
            .filter(|x| match x {
                p::StructItem::Field(_) => true,
                _ => false,
            })
            .map(|x| x.gen_in_struct(&scope))
            .collect::<Result<Vec<Vec<GenResult>>, IozhError>>()
            .map(|vec| vec.into_iter().flatten())?
            .collect::<Vec<_>>();
        let imports = result
            .iter()
            .map(|m| m.imports.clone())
            .flatten()
            .collect::<Vec<_>>();
        let fields = result
            .into_iter()
            .map(|m| m.content)
            .collect::<Vec<_>>().join(",");
        let file_name = gen_filename(&scope.base_name);
        let file_path = parent.folder.join(file_name);
        Ok(vec![
            GenResult {
                file: Some(file_path),
                content: format!("case class {}({fields})", scope.full_type_name),
                imports: imports,
                package: scope.nspace.path,
            }
        ])
    }
}

impl InService for p::Method {
    fn gen_in_service(&self, parent: &ServiceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        let name = self.name.gen()?.to_string();
        let scope = parent.push_method(self);
        let args = self
            .args
            .iter()
            .map(|x| x.gen_in_method(&scope))
            .collect::<Result<Vec<Vec<GenResult>>, IozhError>>()
            .map(|vec| vec.into_iter().flatten())?
            .map(|m| m.content)
            .collect::<Vec<_>>().join("\n");
        let ret = self.result.gen()?.to_string();
        GenResult::single(format!("def {}({}): {}", name, args, ret))
    }
}

impl InNspace for p::Service {
    fn gen_in_nspace(&self, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_service(self)?;
        let methods = self
            .methods
            .iter()
            .map(|x| x.gen_in_service(&scope))
            .collect::<Result<Vec<Vec<GenResult>>, IozhError>>()
            .map(|vec| vec.into_iter().flatten())?
            .map(|m| m.content)
            .collect::<Vec<_>>().join("\n");
        let header = format!("trait {}", scope.full_type_name);
        let body = format!("object {} {{ {} }}", scope.base_name, methods);
        GenResult::single(format!("{}\n{}", header, body))
    }
}

impl InHttpService for p::HttpRoute {
    fn gen_in_http_service(&self, _parent: &HttpServiceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        Ok(vec![
        ])
    }
}

impl InNspace for p::HttpService {
    fn gen_in_nspace(&self, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_http_service(self)?;
        let methods = self
            .routes
            .iter()
            .map(|x| x.gen_in_http_service(&scope))
            .collect::<Result<Vec<Vec<GenResult>>, IozhError>>()
            .map(|vec| vec.into_iter().flatten())?
            .map(|m| m.content)
            .collect::<Vec<_>>().join("\n");
        let header = format!("trait {}", scope.full_type_name);
        let body = format!("object {} {{ {} }}", scope.base_name, methods);
        GenResult::single(format!("{}\n{}", header, body))
    }
}

impl InNspace for p::NspaceItem {
    fn gen_in_nspace(&self, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
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
    fn gen_in_nspace(&self, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_nspace(self);
        self
            .items
            .iter()
            .map(|x| x.gen_in_nspace(&scope))
            .collect::<Result<Vec<_>, IozhError>>()
            .map(|vec| vec.into_iter().flatten().collect())
    }
}

impl InProject for p::Nspace {
    fn gen_in_project(&self, parent: &ProjectContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_nspace(&self);
        self
            .items
            .iter()
            .map(|x| x.gen_in_nspace(&scope))
            .collect::<Result<Vec<_>, IozhError>>()
            .map(|vec| vec.into_iter().flatten().collect())
    }
}

impl p::Project {
    pub fn generate(&self, target_folder: &std::path::Path) -> std::result::Result<(), IozhError> {
        let scope = ProjectContext {
            target_folder: target_folder.to_path_buf(),
        };
        let items = self.nspaces
            .iter()
            .map(|ns| ns.gen_in_project(&scope))
            .collect::<Result<Vec<Vec<GenResult>>, IozhError>>()
            .map(|vec| vec.into_iter().flatten())?
            .collect::<Vec<_>>();
        write_fs_tree(items, &scope)
    }
}