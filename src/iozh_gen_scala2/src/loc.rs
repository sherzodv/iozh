use iozh_parse::ast;

pub trait Loc {
    fn get_pos(&self) -> ast::Pos;
}

impl Loc for ast::Literal {
    fn get_pos(&self) -> ast::Pos {
        match self {
            ast::Literal::Int { pos, .. } => pos.clone(),
            ast::Literal::String { pos, .. } => pos.clone(),
            ast::Literal::Nil => ast::Pos{ line: 0, col: 0 },
        }
    }
}

impl Loc for ast::TypeTag {
    fn get_pos(&self) -> ast::Pos {
        self.pos.clone()
    }
}

impl Loc for ast::TypePath {
    fn get_pos(&self) -> ast::Pos {
        self.pos.clone()
    }
}

impl Loc for ast::ChoiceItem {
    fn get_pos(&self) -> ast::Pos {
        todo!()
    }
}

impl Loc for ast::Choice {
    fn get_pos(&self) -> ast::Pos {
        self.pos.clone()
    }
}

impl Loc for ast::Field {
    fn get_pos(&self) -> ast::Pos {
        self.pos.clone()
    }
}

impl Loc for ast::StructItem {
    fn get_pos(&self) -> ast::Pos {
        match self {
            ast::StructItem::Field(v) => v.pos.clone(),
            ast::StructItem::Tag(v) => v.pos.clone(),
        }
    }
}

impl Loc for ast::Structure {
    fn get_pos(&self) -> ast::Pos {
        self.pos.clone()
    }
}

impl Loc for ast::Method {
    fn get_pos(&self) -> ast::Pos {
        self.pos.clone()
    }
}

impl Loc for ast::Service {
    fn get_pos(&self) -> ast::Pos {
        self.pos.clone()
    }
}

impl Loc for ast::HttpRoute {
    fn get_pos(&self) -> ast::Pos {
        self.pos.clone()
    }
}

impl Loc for ast::HttpService {
    fn get_pos(&self) -> ast::Pos {
        self.pos.clone()
    }
}

impl Loc for ast::NspaceItem {
    fn get_pos(&self) -> ast::Pos {
        todo!()
    }
}

impl Loc for ast::Nspace {
    fn get_pos(&self) -> ast::Pos {
        self.pos.clone()
    }
}