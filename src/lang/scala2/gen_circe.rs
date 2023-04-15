use crate::ast;
use crate::lang::scala2::*;
use crate::lang::scala2::utils::*;
use crate::lang::scala2::gen::*;

use stripmargin::StripMargin;

pub fn circe_pack(_project: &ProjectContext) -> std::result::Result<Vec<GenResult>, IozhError> {
    let content = r#"
    |implicit lazy val durationEncoder: Encoder[Duration] = (x: Duration) => x.toString.asJson
    |implicit lazy val durationDecoder: Decoder[Duration] = Decoder.decodeString.emap {
    |   str => Try(Duration(str)).toEither.left.map(_.getMessage)
    |}
    "#.strip_margin();
    Ok(vec![GenResult {
        unit: Some("package".to_string()),
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

fn literal_decoder(l: &ast::Literal) -> &str {
    match l {
        ast::Literal::Int{ pos: _, value: _ } => "decodeInt",
        ast::Literal::String{ pos: _, value: _ } => "decodeString",
        ast::Literal::Nil => "Nil"
    }
}

fn decoder_for_struct(s: &ast::Structure, ctx: &NspaceContext, parent: Option<&ChoiceContext>) -> std::result::Result<Vec<GenResult>, IozhError> {
    let scope = ctx.push_struct(s)?;
    let mut fields = Vec::new();
    fields.append(s.fields.iter().filter(|f| match f {
        ast::StructItem::Field(_) => true,
        _ => false,
    }).map(|f| f.clone()).collect::<Vec<_>>().as_mut());
    let name = if let Some(pp) = parent {
        fields.append(pp.p.fields.iter().map(|f| ast::StructItem::Field(f.clone())).collect::<Vec<_>>().as_mut());
        if fields.is_empty() {
            return ast::ChoiceItem::TypeTag {
                doc: "".to_string(),
                choice: s.name.clone(),
            }.decoder_in_choice(pp);
        }
        if pp.base_name.len() > 0 {
            pp.base_name.to_string() + "." + &scope.base_name
        } else {
            scope.base_name.to_string()
        }
    } else {
        scope.base_name.to_string()
    };
    let decoder_name = name.replace(".", "").to_ascii_lowercase();
    let postfix = if s.fields.is_empty() { ".type" } else { "" };

    let fields_decoders = fields.mapg(|x| x.decoder_in_struct(&scope))?;
    let decoder_fields_parse = fields_decoders.map_content().join("\n");
    let decoder_fields_list = fields
        .iter()
        .filter(|x| match x {
            ast::StructItem::Field(_) => true,
            _ => false,
        })
        .map(|x| match x {
            ast::StructItem::Field(f) => {
                let arg = sanitize(&f.name);
                let var = &f.name;
                format!("{arg} = _{var}")
            }
            _ => "".to_string(),
        })
        .collect::<Vec<_>>()
        .join(",");
    let decoder_body = if !fields.is_empty() {
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
    let fields_imports = fields_decoders.map_imports();
    Ok(vec![
        GenResult {
            unit: None,
            content: decoder,
            imports: fields_imports,
            package: vec![],
            block: None,
        }
    ])
}

fn encoder_for_struct(s: &ast::Structure, ctx: &NspaceContext, parent: Option<&ChoiceContext>) -> std::result::Result<Vec<GenResult>, IozhError> {
    let scope = ctx.push_struct(s)?;
    let mut fields = Vec::new();
    fields.append(s.fields.iter().filter(|f| match f {
        ast::StructItem::Field(_) => true,
        _ => false,
    }).map(|f| f.clone()).collect::<Vec<_>>().as_mut());
    let name = if let Some(pp) = parent {
        fields.append(pp.p.fields.iter().map(|f| ast::StructItem::Field(f.clone())).collect::<Vec<_>>().as_mut());
        if fields.is_empty() {
            return ast::ChoiceItem::TypeTag {
                doc: "".to_string(),
                choice: s.name.clone(),
            }.encoder_in_choice(pp);
        }
        if pp.base_name.len() > 0 {
            pp.base_name.to_string() + "." + &scope.base_name
        } else {
            scope.base_name.to_string()
        }
    } else {
        scope.base_name.to_string()
    };
    let encoder_name = name.replace(".", "").to_ascii_lowercase();
    let postfix = if fields.is_empty() { ".type" } else { "" };
    let fields_encoders = fields
        .mapg(|x| x.encoder_in_struct(&scope))?;
    let encoder_fields_parse = fields_encoders.map_content().join(",\n");
    let type_bounds = scope.type_args.join(": Encoder, ") + ": Encoder";
    let type_args = scope.type_args.join(",");
    let type_args_opt = if type_args.len() > 0 {
        format!("[{type_args}]", type_args = type_args)
    } else {
        "".to_string()
    };
    let encoder_body = if !fields.is_empty() {
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
    let fields_imports = fields_encoders.map_imports();
    Ok(vec![
        GenResult {
            unit: None,
            content: encoder,
            imports: fields_imports,
            package: vec![],
            block: None,
        }
    ])
}

fn decoder_for_choice_in_nspace(c: &ast::Choice, path: &str, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
    let scope = parent.push_choice(c)?;
    let name = if path.len() > 0 {
        path.to_owned() + "." + &scope.base_name
    } else {
        scope.base_name.to_string()
    };
    let decoder_name = name.replace(".", "").to_ascii_lowercase();
    let postfix = if c.choices.is_empty() { ".type" } else { "" };
    let items = c.choices
        .iter()
        .filter(|x| match x {
            ast::ChoiceItem::Structure(_) => true,
            ast::ChoiceItem::TypeTag{ doc: _, choice: _} => true,
            ast::ChoiceItem::Value{doc: _, name: _, value: _ } => true,
            ast::ChoiceItem::Wrap{doc: _, name: _, field: _, target: _ } => true,
            _ => false,
        })
        .map(|x| {
            let type_name = match x {
                ast::ChoiceItem::Structure(s) => s.name.name.clone(),
                ast::ChoiceItem::TypeTag{ doc: _, choice } => choice.name.clone(),
                ast::ChoiceItem::Value{doc: _, name, value: _ } => name.name.clone(),
                ast::ChoiceItem::Wrap{doc: _, name, field: _, target: _ } => name.name.clone(),
                _ => "ERROR_CHOICE_ITEM".to_string(),
            };
            (x, type_name)
        })
        .map(|(x, type_name)| {
            if let Some(tag_key) = &scope.most_common_tag_key {
                let path = &scope.base_name;
                let name = format!("{path}.{type_name}");
                let tag_value = x.get_tag_value(tag_key);
                format!(r#"case {tag_value} => Decoder[{name}]"#)
            } else {
                let path = &scope.base_name;
                let name = format!("{path}{type_name}").to_ascii_lowercase();
                format!("{name}Decoder.widen")
            }
        })
        .collect::<Vec<_>>();

    let decoder_body = if let Some(tag_key) = &scope.most_common_tag_key {
        let decoder_items = items.join("\n");
        format!(r#"
            |for {{
            |  fType <- Decoder[String].prepare(_.downField("{tag_key}"))
            |  value <- fType match {{
            |    {decoder_items}
            |  }}
            |}} yield value
            "#).strip_margin()
    } else {
        let decoder_items = items.join(",\n");
        format!(r#"
            |List[Decoder[{name}]](
            |{decoder_items}
            |).reduceLeft(_ or _)"#).strip_margin()
    };
    let decoder = format!("implicit lazy val {decoder_name}Decoder: Decoder[{name}{postfix}] = {decoder_body}");
    Ok(vec![
        GenResult {
            unit: None,
            content: decoder,
            imports: vec![],
            package: vec![],
            block: None,
        }
    ])
}

fn encoder_for_choice_in_nspace(c: &ast::Choice, path: &str, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
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
            ast::ChoiceItem::Structure(_) => true,
            ast::ChoiceItem::TypeTag{ doc: _, choice: _} => true,
            ast::ChoiceItem::Value{doc: _, name: _, value: _ } => true,
            ast::ChoiceItem::Wrap{doc: _, name: _, field: _, target: _ } => true,
            _ => false,
        })
        .map(|x| {
            let nn = match x {
                ast::ChoiceItem::Structure(s) => s.name.name.clone(),
                ast::ChoiceItem::TypeTag{ doc: _, choice } => choice.name.clone() + ".type",
                ast::ChoiceItem::Value{doc: _, name, value: _ } => name.name.clone() + ".type",
                ast::ChoiceItem::Wrap{doc: _, name, field: _, target: _ } => name.name.clone(),
                _ => "ERROR_CHOICE_ITEM".to_string(),
            };
            (x, nn)
        })
        .map(|(x, type_name)| {
            let path = &scope.base_name;
            let postfix = if let Some(tag_key) = &scope.most_common_tag_key {
                let tag_value = x.get_tag_value(tag_key);
                format!(r#".mapObject(_.add("{tag_key}", Json.fromString({tag_value})))"#)
            } else {
                "".to_string()
            };
            if path.len() > 0 {
                format!("case x: {path}.{type_name} => x.asJson{postfix}")
            } else {
                format!("case x: {type_name} => x.asJson{postfix}")
            }
        })
        .collect::<Vec<_>>().join("\n");
    let encoder_body = format!(r#"{{
        |{encoder_items}
        |}}"#).strip_margin();
    let encoder = format!("implicit lazy val {encoder_name}Encoder: Encoder[{name}{postfix}] = {encoder_body}");
    Ok(vec![
        GenResult {
            unit: None,
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

impl CirceInStruct for ast::Field {
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
                unit: None,
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
        let fsname = sanitize(fname);
        let content = format!(r#""{fname}" -> x.{fsname}.asJson"#);
        let imports = encoder_imports_for(tp);
        Ok(vec![
            GenResult {
                unit: None,
                content,
                imports,
                package: vec![],
                block: None,
            }
        ])
    }
}

impl CirceInStruct for ast::StructItem {
    fn decoder_in_struct(&self, parent: &StructContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        match self {
            ast::StructItem::Field(v) => v.decoder_in_struct(parent),
            ast::StructItem::Tag(_v) => GenResult::empty()
        }
    }

    fn encoder_in_struct(&self, parent: &StructContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        match self {
            ast::StructItem::Field(v) => v.encoder_in_struct(parent),
            ast::StructItem::Tag(_v) => GenResult::empty()
        }
    }
}

impl CirceInNspace for ast::Structure {
    fn decoder_in_nspace(&self, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        decoder_for_struct(&self, parent, None)
    }

    fn encoder_in_nspace(&self, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        encoder_for_struct(&self, parent, None)
    }

    fn codec_in_nspace(&self, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_struct(self)?;
        let decoder_res = self.decoder_in_nspace(&parent)?;
        let encoder_res = self.encoder_in_nspace(&parent)?;
        let decoder = decoder_res.into_iter().map(|x| x.content).collect::<Vec<_>>().join("\n");
        let encoder = encoder_res.into_iter().map(|x| x.content).collect::<Vec<_>>().join("\n");
        let content = format!("{decoder}\n{encoder}\n");
        Ok(vec![
            GenResult {
                unit: Some("package".to_string()),
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

impl CirceInChoice for ast::Structure {
    fn decoder_in_choice(&self, parent: &ChoiceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        decoder_for_struct(self, &parent.nspace, Some(parent))
    }

    fn encoder_in_choice(&self, parent: &ChoiceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        encoder_for_struct(self, &parent.nspace, Some(&parent))
    }
}

impl CirceInChoice for ast::ChoiceItem {
    fn decoder_in_choice(&self, parent: &ChoiceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        match self {
            ast::ChoiceItem::Structure(s) => {
                s.decoder_in_choice(&parent)
            }
            ast::ChoiceItem::TypeTag{ doc: _, choice } => {
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
                        unit: None,
                        content: decoder,
                        imports: vec![],
                        package: vec![],
                        block: None,
                    }
                ])
            }
            ast::ChoiceItem::Value{doc: _, name, value } => {
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
                        unit: None,
                        content: decoder,
                        imports: vec![],
                        package: vec![],
                        block: None,
                    }
                ])
            }
            ast::ChoiceItem::Wrap { doc: _, name, field: _, target } => {
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
                        unit: None,
                        content: decoder,
                        imports: imports,
                        package: vec![],
                        block: None,
                    }
                ])
            }
            ast::ChoiceItem::Nil => GenResult::single("ERROR_CHOICE_ITEM_SHOULDNT_HAPPEN".to_string()),
        }
    }

    fn encoder_in_choice(&self, parent: &ChoiceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        match self {
            ast::ChoiceItem::Structure(s) => {
                s.encoder_in_choice(&parent)
            }
            ast::ChoiceItem::TypeTag{ doc: _, choice } => {
                let type_name = &choice.name;
                let name = parent.base_name.to_string() + "." + type_name;
                let codec_name = (parent.base_name.to_string() + &type_name).to_ascii_lowercase();
                let v = codec_name.to_string();
                let encoder = format!(r#"
                    |implicit lazy val {codec_name}Encoder: Encoder[{name}.type] = (_: {name}.type) => "{v}".asJson
                    |"#).strip_margin();
                Ok(vec![
                    GenResult {
                        unit: None,
                        content: encoder,
                        imports: vec![],
                        package: vec![],
                        block: None,
                    }
                ])
            }
            ast::ChoiceItem::Value{doc: _, name, value } => {
                let type_name = &name.name;
                let name = parent.base_name.to_string() + "." + type_name;
                let codec_name = (parent.base_name.to_string() + &type_name).to_ascii_lowercase();
                let v = match value {
                    ast::Literal::Int{ pos: _, value } => format!("{}", value),
                    ast::Literal::String{ pos: _, value } => format!("{}", value),
                    ast::Literal::Nil => todo!(),
                };
                let encoder = format!(r#"
                    |implicit lazy val {codec_name}Encoder: Encoder[{name}.type] = (_: {name}.type) => {v}.asJson
                    |"#).strip_margin();
                Ok(vec![
                    GenResult {
                        unit: None,
                        content: encoder,
                        imports: vec![],
                        package: vec![],
                        block: None,
                    }
                ])
            }
            ast::ChoiceItem::Wrap{doc: _, name, field, target } => {
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
                        unit: None,
                        content: encoder,
                        imports: vec![],
                        package: vec![],
                        block: None,
                    }
                ])
            }
            ast::ChoiceItem::Nil => GenResult::single("ERROR_CHOICE_ITEM_SHOULDNT_HAPPEN".to_string()),
        }
    }
}

impl CirceInNspace for ast::Choice {
    fn codec_in_nspace(&self, parent: &NspaceContext) -> std::result::Result<Vec<GenResult>, IozhError> {
        let scope = parent.push_choice(self)?;
        let decoder_res = self.decoder_in_nspace(parent)?;
        let encoder_res = self.encoder_in_nspace(parent)?;
        let decoder = decoder_res.into_iter().map(|x| x.content).collect::<Vec<_>>().join("\n");
        let encoder = encoder_res.into_iter().map(|x| x.content).collect::<Vec<_>>().join("\n");
        let content = format!("{decoder}\n{encoder}\n");
        let unit = Some("package".to_string());
        let mut items_decoders = self.choices
            .filter_gen(|x| match x {
                ast::ChoiceItem::Structure(_) => true,
                ast::ChoiceItem::TypeTag{ doc: _, choice: _} => true,
                ast::ChoiceItem::Value{doc: _, name: _, value: _ } => true,
                ast::ChoiceItem::Wrap{doc: _, name: _, field: _, target: _ } => true,
                _ => false,
            }, |x| x.decoder_in_choice(&scope))?;
        items_decoders
            .iter_mut()
            .for_each(|res| {
                res.unit = unit.clone();
                res.package = scope.nspace.path.clone();
                res.block = Some("object CirceImplicits".to_string());
            });
        let mut items_encoders = self.choices
            .filter_gen(|x| match x {
                ast::ChoiceItem::Structure(_) => true,
                ast::ChoiceItem::TypeTag{ doc: _, choice: _} => true,
                ast::ChoiceItem::Value{doc: _, name: _, value: _ } => true,
                ast::ChoiceItem::Wrap{doc: _, name: _, field: _, target: _ } => true,
                _ => false,
            }, |x| x.encoder_in_choice(&scope))?;
        items_encoders
            .iter_mut()
            .for_each(|res| {
                res.unit = unit.clone();
                res.package = scope.nspace.path.clone();
                res.block = Some("object CirceImplicits".to_string());
            });
        let body = GenResult {
            unit,
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