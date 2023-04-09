use crate::parser as p;
use crate::lang::scala2::*;
use crate::lang::scala2::utils::*;

use stripmargin::StripMargin;

pub fn circe_pack(project: &ProjectContext) -> std::result::Result<Vec<GenResult>, IozhError> {
    let content = r#"
    |implicit lazy val durationEncoder: Encoder[Duration] = (x: Duration) => x.toString.asJson
    |implicit lazy val durationDecoder: Decoder[Duration] = Decoder.decodeString.emap {
    |   str => Try(Duration(str)).toEither.left.map(_.getMessage)
    |}
    "#.strip_margin();
    let mut file = project.target_folder.clone();
    file.push("iozh");
    file.push("circe");
    file.push("package.scala");
    Ok(vec![GenResult {
        file: Some(file),
        content,
        imports: vec![
            "io.circe.Encoder".to_string(),
            "io.circe.Decoder".to_string(),
            "io.circe.syntax._".to_string(),
            "scala.util.Try".to_string(),
            "scala.concurrent.duration.Duration".to_string(),
        ],
        package: vec![
            "iozh".to_string(),
            "circe".to_string(),
        ],
        block: Some("object Implicits".to_string()),
    }])
}

fn literal_decoder(l: &p::Literal) -> &str {
    match l {
        p::Literal::Int{ pos: _, value: _ } => "decodeInt",
        p::Literal::String{ pos: _, value: _ } => "decodeString",
        p::Literal::Nil => "Nil"
    }
}

fn decoder_for_struct_in_nspace(s: &p::Structure, path: &str, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
    let scope = parent.push_struct(s)?;
    let name = if path.len() > 0 {
        path.to_string() + "." + &scope.base_name
    } else {
        scope.base_name.to_string()
    };
    let decoder_name = name.replace(".", "").to_ascii_lowercase();
    let postfix = if s.fields.is_empty() { ".type" } else { "" };
    let fields_decoders = s.fields
        .iter()
        .map(|x| x.decoder_in_struct(&scope))
        .collect::<Result<Vec<Vec<GenResult>>, IozhError>>()
        .map(|vec| vec.into_iter().flatten())?
        .collect::<Vec<_>>();
    let decoder_fields_parse = fields_decoders
        .iter()
        .map(|d| d.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    let decoder_fields_list = s.fields
        .iter()
        .filter(|x| match x {
            p::StructItem::Field(_) => true,
            _ => false,
        })
        .map(|x| match x {
            p::StructItem::Field(f) => {
                let arg = sanitize(&f.name);
                let var = &f.name;
                format!("{arg} = _{var}")
            }
            _ => "".to_string(),
        })
        .collect::<Vec<_>>()
        .join(",");
    let decoder_body = if !s.fields.is_empty() {
        format!(r#"
            |Decoder.instance {{ h =>
            |  for {{
            |    {decoder_fields_parse}
            |  }} yield {{
            |    {name}({decoder_fields_list})
            |  }}
            |}}"#).strip_margin()
    } else {
        format!("(_: HCursor) => Right({name})")
    };
    let type_bounds = scope.type_args.join(": Decoder, ") + ": Decoder";
    let type_args = scope.type_args.join(",");
    let decoder = if type_args.len() > 0 {
        format!("implicit def {decoder_name}Decoder[{type_bounds}]: Decoder[{name}[{type_args}]{postfix}] = {decoder_body}")
    } else {
        format!("implicit lazy val {decoder_name}Decoder: Decoder[{name}{postfix}] = {decoder_body}")
    };
    let fields_imports = fields_decoders.iter().map(|f| f.imports.clone()).flatten().collect::<Vec<_>>();
    Ok(vec![
        GenResult {
            file: None,
            content: decoder,
            imports: fields_imports,
            package: vec![],
            block: None,
        }
    ])
}

fn decoder_for_choice_in_nspace(c: &p::Choice, path: &str, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
    let scope = parent.push_choice(c)?;
    let name = if path.len() > 0 {
        path.to_owned() + "." + &scope.base_name
    } else {
        scope.base_name.to_string()
    };
    let decoder_name = name.replace(".", "").to_ascii_lowercase();
    let postfix = if c.choices.is_empty() { ".type" } else { "" };
    let decoder_items = c.choices
        .iter()
        .filter(|x| match x {
            p::ChoiceItem::Structure(_) => true,
            p::ChoiceItem::TypeTag{ doc: _, choice: _} => true,
            p::ChoiceItem::Value{doc: _, name: _, value: _ } => true,
            p::ChoiceItem::Wrap{doc: _, name: _, field: _, target: _ } => true,
            _ => false,
        })
        .map(|x| match x {
            p::ChoiceItem::Structure(s) => s.name.name.to_ascii_lowercase(),
            p::ChoiceItem::TypeTag{ doc: _, choice } => choice.name.to_ascii_lowercase(),
            p::ChoiceItem::Value{doc: _, name, value: _ } => name.name.to_ascii_lowercase(),
            p::ChoiceItem::Wrap{doc: _, name, field: _, target: _ } => name.name.to_ascii_lowercase(),
            _ => "ERROR_CHOICE_ITEM".to_string(),
        })
        .map(|x| {
            let path = &scope.base_name.to_lowercase();
            if path.len() > 0 {
                format!("{path}{x}Decoder.widen", path = path, x = x)
            } else {
                format!("{x}Decoder.widen", x = x)
            }
        })
        .collect::<Vec<_>>().join(",\n");
    let decoder_body = format!(r#"
        |List[Decoder[{name}]](
        |{decoder_items}
        |).reduceLeft(_ or _)"#).strip_margin();
    let decoder = format!("implicit lazy val {decoder_name}Decoder: Decoder[{name}{postfix}] = {decoder_body}");
    Ok(vec![
        GenResult {
            file: None,
            content: decoder,
            imports: vec![],
            package: vec![],
            block: None,
        }
    ])
}

fn encoder_for_struct_in_nspace(s: &p::Structure, path: &str, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
    let scope = parent.push_struct(s)?;
    let name = if path.len() > 0 {
        path.to_string() + "." + &scope.base_name
    } else {
        scope.base_name.to_string()
    };
    let encoder_name = name.replace(".", "").to_ascii_lowercase();
    let postfix = if s.fields.is_empty() { ".type" } else { "" };
    let fields_encoders = s.fields
        .iter()
        .map(|x| x.encoder_in_struct(&scope))
        .collect::<Result<Vec<Vec<GenResult>>, IozhError>>()
        .map(|vec| vec.into_iter().flatten())?
        .collect::<Vec<_>>();
    let encoder_fields_parse = fields_encoders
        .iter()
        .map(|d| d.content.clone())
        .collect::<Vec<_>>()
        .join(",\n");
    let type_bounds = scope.type_args.join(": Encoder, ") + ": Encoder";
    let type_args = scope.type_args.join(",");
    let type_args_opt = if type_args.len() > 0 {
        format!("[{type_args}]", type_args = type_args)
    } else {
        "".to_string()
    };
    let encoder_body = if !s.fields.is_empty() {
        format!(r#"
            |(x: {name}{type_args_opt}) => {{
            |  Json.fromFields(
            |    List(
            |      {encoder_fields_parse}
            |    ).filter(!_._2.isNull)
            |  )
            |}}"#).strip_margin()
    } else {
        format!("(_: HCursor) => Right({name})")
    };
    let encoder = if type_args.len() > 0 {
        format!("implicit def {encoder_name}encoder[{type_bounds}]: Encoder[{name}[{type_args}]{postfix}] = {encoder_body}")
    } else {
        format!("implicit lazy val {encoder_name}encoder: Encoder[{name}{postfix}] = {encoder_body}")
    };
    let fields_imports = fields_encoders.iter().map(|f| f.imports.clone()).flatten().collect::<Vec<_>>();
    Ok(vec![
        GenResult {
            file: None,
            content: encoder,
            imports: fields_imports,
            package: vec![],
            block: None,
        }
    ])
}

fn encoder_for_choice_in_nspace(c: &p::Choice, path: &str, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
    let scope = parent.push_choice(c)?;
    let name = if path.len() > 0 {
        path.to_owned() + "." + &scope.base_name
    } else {
        scope.base_name.to_string()
    };
    let encoder_name = name.replace(".", "").to_ascii_lowercase();
    let postfix = if c.choices.is_empty() { ".type" } else { "" };
    let encoder_items = c.choices
        .iter()
        .filter(|x| match x {
            p::ChoiceItem::Structure(_) => true,
            p::ChoiceItem::TypeTag{ doc: _, choice: _} => true,
            p::ChoiceItem::Value{doc: _, name: _, value: _ } => true,
            p::ChoiceItem::Wrap{doc: _, name: _, field: _, target: _ } => true,
            _ => false,
        })
        .map(|x| match x {
            p::ChoiceItem::Structure(s) => s.name.name.clone(),
            p::ChoiceItem::TypeTag{ doc: _, choice } => choice.name.clone() + ".type",
            p::ChoiceItem::Value{doc: _, name, value: _ } => name.name.clone() + ".type",
            p::ChoiceItem::Wrap{doc: _, name, field: _, target: _ } => name.name.clone(),
            _ => "ERROR_CHOICE_ITEM".to_string(),
        })
        .map(|x| {
            let path = &scope.base_name;
            if path.len() > 0 {
                format!("case x: {path}.{x} => x.asJson")
            } else {
                format!("case x: {x} => x.asJson")
            }
        })
        .collect::<Vec<_>>().join("\n");
    let encoder_body = format!(r#"{{
        |{encoder_items}
        |}}"#).strip_margin();
    let encoder = format!("implicit lazy val {encoder_name}Encoder: Encoder[{name}{postfix}] = {encoder_body}");
    Ok(vec![
        GenResult {
            file: None,
            content: encoder,
            imports: vec![],
            package: vec![],
            block: None,
        }
    ])
}

fn decoder_imports_for(type_name: String) -> Vec<String> {
    let mut v: Vec<String> = vec![];
    if type_name.starts_with("Duration") {
        v.push("iozh.circe.Implicits.durationDecoder".to_string());
    }
    v
}

fn encoder_imports_for(type_name: String) -> Vec<String> {
    let mut v: Vec<String> = vec![];
    if type_name.starts_with("Duration") {
        v.push("iozh.circe.Implicits.durationEncoder".to_string());
    }
    v
}

impl CirceInStruct for p::Field {
    fn decoder_in_struct(&self, _parent: &StructContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        let tp = self.type_path.gen()?.to_string();
        let fname = &self.name;
        let content = if tp.starts_with("List") {
            format!("_{fname} <- h.getOrElse[{tp}](\"{fname}\")(List.empty)")
        } else {
            format!("_{fname} <- h.get[{tp}](\"{fname}\")")
        };
        let imports = decoder_imports_for(tp);
        Ok(vec![
            GenResult {
                file: None,
                content,
                imports,
                package: vec![],
                block: None,
            }
        ])
    }

    fn encoder_in_struct(&self, _parent: &StructContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        let tp = self.type_path.gen()?.to_string();
        let fname = &self.name;
        let content = format!(r#""{fname}" -> x.{fname}.asJson"#);
        let imports = encoder_imports_for(tp);
        Ok(vec![
            GenResult {
                file: None,
                content,
                imports,
                package: vec![],
                block: None,
            }
        ])
    }
}

impl CirceInStruct for p::StructItem {
    fn decoder_in_struct(&self, parent: &StructContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        match self {
            p::StructItem::Field(v) => v.decoder_in_struct(parent),
            p::StructItem::Tag(_v) => GenResult::empty()
        }
    }

    fn encoder_in_struct(&self, parent: &StructContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        match self {
            p::StructItem::Field(v) => v.encoder_in_struct(parent),
            p::StructItem::Tag(_v) => GenResult::empty()
        }
    }
}

impl CirceInNspace for p::Structure {
    fn decoder_in_nspace(&self, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        decoder_for_struct_in_nspace(&self, "", parent)
    }

    fn encoder_in_nspace(&self, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        encoder_for_struct_in_nspace(&self, "", parent)
    }

    fn codec_in_nspace(&self, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_struct(self)?;
        let decoder_res = self.decoder_in_nspace(&parent)?;
        let encoder_res = self.encoder_in_nspace(&parent)?;
        let decoder = decoder_res.into_iter().map(|x| x.content).collect::<Vec<_>>().join("\n");
        let encoder = encoder_res.into_iter().map(|x| x.content).collect::<Vec<_>>().join("\n");
        let content = format!("{decoder}\n{encoder}\n");
        let file_path = parent.folder.join("package.scala");
        Ok(vec![
            GenResult {
                file: Some(file_path),
                content: content,
                imports: vec![
                    "io.circe.Decoder".to_string(),
                    "io.circe.Encoder".to_string(),
                    "io.circe.HCursor".to_string(),
                    "io.circe.syntax._".to_string(),
                    "io.circe.Json".to_string(),
                ],
                package: scope.nspace.path,
                block: Some("object CirceImplicits".to_string()),
            }
        ])
    }

}

impl CirceInChoice for p::Structure {
    fn decoder_in_choice(&self, parent: &ChoiceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        decoder_for_struct_in_nspace(self, &parent.base_name, &parent.nspace)
    }

    fn encoder_in_choice(&self, parent: &ChoiceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        encoder_for_struct_in_nspace(self, &parent.base_name, &parent.nspace)
    }
}

impl CirceInChoice for p::ChoiceItem {
    fn decoder_in_choice(&self, parent: &ChoiceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        match self {
            p::ChoiceItem::Structure(s) => {
                s.decoder_in_choice(&parent)
            }
            p::ChoiceItem::TypeTag{ doc: _, choice } => {
                let type_name = &choice.name;
                let name = parent.base_name.to_string() + "." + type_name;
                let codec_name = (parent.base_name.to_string() + &type_name).to_ascii_lowercase();
                let decoder = format!(r#"
                    |implicit lazy val {codec_name}Decoder: Decoder[{name}.type] = Decoder.decodeString.emap {{ v =>
                    |  if (v == "{codec_name}") Right({name})
                    |  else Left("Expected {codec_name} but got " + v)
                    |}}
                    |"#).strip_margin();
                Ok(vec![
                    GenResult {
                        file: None,
                        content: decoder,
                        imports: vec![],
                        package: vec![],
                        block: None,
                    }
                ])
            }
            p::ChoiceItem::Value{doc: _, name, value } => {
                let type_name = &name.name;
                let name = parent.base_name.to_string() + "." + type_name;
                let ldecoder = literal_decoder(value);
                let codec_name = (parent.base_name.to_string() + &type_name).to_ascii_lowercase();
                let decoder = format!(r#"
                    |implicit lazy val {codec_name}Decoder: Decoder[{name}.type] = Decoder.{ldecoder}.emap {{ v =>
                    |  if (v == {name}.getValue) Right({name})
                    |  else Left("Expected {name} but got " + v)
                    |}}
                    |"#).strip_margin();
                Ok(vec![
                    GenResult {
                        file: None,
                        content: decoder,
                        imports: vec![],
                        package: vec![],
                        block: None,
                    }
                ])
            }
            p::ChoiceItem::Wrap { doc: _, name, field: _, target } => {
                let type_name = &name.name;
                let name = parent.base_name.to_string() + "." + type_name;
                let codec_name = (parent.base_name.to_string() + &type_name).to_ascii_lowercase();
                let target_name = &target.gen()?.to_string();
                let mut imports: Vec<String> = vec![];
                let decoder_body = if target_name == "File" {
                    imports.push("java.io.File".to_string());
                    format!("Decoder[String].map(s => {name}(new File(s)))")
                } else {
                    format!("Decoder[{target_name}].map({name}.apply)")
                };
                let decoder = format!("implicit lazy val {codec_name}Decoder: Decoder[{name}] = {decoder_body}");
                Ok(vec![
                    GenResult {
                        file: None,
                        content: decoder,
                        imports: imports,
                        package: vec![],
                        block: None,
                    }
                ])
            }
            p::ChoiceItem::Nil => GenResult::single("ERROR_CHOICE_ITEM_SHOULDNT_HAPPEN".to_string()),
        }
    }

    fn encoder_in_choice(&self, parent: &ChoiceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        match self {
            p::ChoiceItem::Structure(s) => {
                s.encoder_in_choice(&parent)
            }
            p::ChoiceItem::TypeTag{ doc: _, choice } => {
                let type_name = &choice.name;
                let name = parent.base_name.to_string() + "." + type_name;
                let codec_name = (parent.base_name.to_string() + &type_name).to_ascii_lowercase();
                let v = codec_name.to_string();
                let encoder = format!(r#"
                    |implicit lazy val {codec_name}Encoder: Encoder[{name}.type] = (_: {name}.type) => "{v}".asJson
                    |"#).strip_margin();
                Ok(vec![
                    GenResult {
                        file: None,
                        content: encoder,
                        imports: vec![],
                        package: vec![],
                        block: None,
                    }
                ])
            }
            p::ChoiceItem::Value{doc: _, name, value } => {
                let type_name = &name.name;
                let name = parent.base_name.to_string() + "." + type_name;
                let codec_name = (parent.base_name.to_string() + &type_name).to_ascii_lowercase();
                let v = match value {
                    p::Literal::Int{ pos: _, value } => format!("{}", value),
                    p::Literal::String{ pos: _, value } => format!("{}", value),
                    p::Literal::Nil => todo!(),
                };
                let encoder = format!(r#"
                    |implicit lazy val {codec_name}Encoder: Encoder[{name}.type] = (_: {name}.type) => {v}.asJson
                    |"#).strip_margin();
                Ok(vec![
                    GenResult {
                        file: None,
                        content: encoder,
                        imports: vec![],
                        package: vec![],
                        block: None,
                    }
                ])
            }
            p::ChoiceItem::Wrap{doc: _, name, field, target } => {
                let type_name = &name.name;
                let name = parent.base_name.to_string() + "." + type_name;
                let codec_name = (parent.base_name.to_string() + &type_name).to_ascii_lowercase();
                let target_name = &target.gen()?.to_string();
                let encoder_body = if target_name == "File" {
                    format!("(x: {name}) => x.{field}.getName.asJson")
                } else {
                    format!("(x: {name}) => x.{field}.asJson")
                };
                let encoder = format!("implicit lazy val {codec_name}Encoder: Encoder[{name}] = {encoder_body}");
                Ok(vec![
                    GenResult {
                        file: None,
                        content: encoder,
                        imports: vec![],
                        package: vec![],
                        block: None,
                    }
                ])
            }
            p::ChoiceItem::Nil => GenResult::single("ERROR_CHOICE_ITEM_SHOULDNT_HAPPEN".to_string()),
        }
    }
}

impl CirceInNspace for p::Choice {
    fn codec_in_nspace(&self, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_choice(self)?;
        let decoder_res = self.decoder_in_nspace(parent)?;
        let encoder_res = self.encoder_in_nspace(parent)?;
        let decoder = decoder_res.into_iter().map(|x| x.content).collect::<Vec<_>>().join("\n");
        let encoder = encoder_res.into_iter().map(|x| x.content).collect::<Vec<_>>().join("\n");
        let content = format!("{decoder}\n{encoder}\n");
        let file_path = parent.folder.join("package.scala");
        let mut items_decoders = self.choices
            .iter()
            .filter(|x| match x {
                p::ChoiceItem::Structure(_) => true,
                p::ChoiceItem::TypeTag{ doc: _, choice: _} => true,
                p::ChoiceItem::Value{doc: _, name: _, value: _ } => true,
                p::ChoiceItem::Wrap{doc: _, name: _, field: _, target: _ } => true,
                _ => false,
            })
            .map(|x| x.decoder_in_choice(&scope))
            .collect::<Result<Vec<Vec<GenResult>>, IozhError>>()
            .map(|vec| vec.into_iter().flatten())?
            .collect::<Vec<_>>();
        items_decoders
            .iter_mut()
            .for_each(|res| {
                res.file = Some(file_path.clone());
                res.package = scope.nspace.path.clone();
                res.block = Some("object CirceImplicits".to_string());
            });
        let mut items_encoders = self.choices
            .iter()
            .filter(|x| match x {
                p::ChoiceItem::Structure(_) => true,
                p::ChoiceItem::TypeTag{ doc: _, choice: _} => true,
                p::ChoiceItem::Value{doc: _, name: _, value: _ } => true,
                p::ChoiceItem::Wrap{doc: _, name: _, field: _, target: _ } => true,
                _ => false,
            })
            .map(|x| x.encoder_in_choice(&scope))
            .collect::<Result<Vec<Vec<GenResult>>, IozhError>>()
            .map(|vec| vec.into_iter().flatten())?
            .collect::<Vec<_>>();
        items_encoders
            .iter_mut()
            .for_each(|res| {
                res.file = Some(file_path.clone());
                res.package = scope.nspace.path.clone();
                res.block = Some("object CirceImplicits".to_string());
            });
        let body = GenResult {
            file: Some(file_path),
            content: content,
            imports: vec![
                "io.circe.Decoder".to_string(),
                "io.circe.Encoder".to_string(),
                "cats.syntax.functor._".to_string(),
            ],
            package: scope.nspace.path,
            block: Some("object CirceImplicits".to_string()),
        };
        items_decoders.append(&mut items_encoders);
        items_decoders.push(body);
        Ok(items_decoders)
    }

    fn decoder_in_nspace(&self, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        decoder_for_choice_in_nspace(self, "", parent)
    }

    fn encoder_in_nspace(&self, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        encoder_for_choice_in_nspace(self, "", parent)
    }
}