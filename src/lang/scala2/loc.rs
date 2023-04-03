use crate::parser::{self as p, Literal};

pub trait Loc {
    fn get_pos(&self) -> p::Pos;
}

impl Loc for Literal {
    fn get_pos(&self) -> p::Pos {
        match self {
            Literal::Int { pos, .. } => pos.clone(),
            Literal::String { pos, .. } => pos.clone(),
            Literal::Nil => p::Pos{ line: 0, col: 0 },
        }
    }
}

impl Loc for p::TypeTag {
    fn get_pos(&self) -> p::Pos {
        self.pos.clone()
    }
}

impl Loc for p::TypePath {
    fn get_pos(&self) -> p::Pos {
        self.pos.clone()
    }
}

impl Loc for p::ChoiceItem {
    fn get_pos(&self) -> p::Pos {
        match self {
            p::ChoiceItem::Structure(v) => v.get_pos(),
            p::ChoiceItem::TypeTag { doc: _, choice } => choice.get_pos(),
            p::ChoiceItem::Value { doc: _, name, value: _ } => name.get_pos(),
            p::ChoiceItem::Nil => p::Pos{ line: 0, col: 0 },
        }
    }
}

impl Loc for p::Choice {
    fn get_pos(&self) -> p::Pos {
        self.pos.clone()
    }
}

impl Loc for p::Field {
    fn get_pos(&self) -> p::Pos {
        self.pos.clone()
    }
}

impl Loc for p::StructItem {
    fn get_pos(&self) -> p::Pos {
        match self {
            p::StructItem::Field(v) => v.pos.clone(),
            p::StructItem::Tag(v) => v.pos.clone(),
        }
    }
}

impl Loc for p::Structure {
    fn get_pos(&self) -> p::Pos {
        self.pos.clone()
    }
}

impl Loc for p::Method {
    fn get_pos(&self) -> p::Pos {
        self.pos.clone()
    }
}

impl Loc for p::Service {
    fn get_pos(&self) -> p::Pos {
        self.pos.clone()
    }
}

impl Loc for p::HttpRoute {
    fn get_pos(&self) -> p::Pos {
        self.pos.clone()
    }
}

impl Loc for p::HttpService {
    fn get_pos(&self) -> p::Pos {
        self.pos.clone()
    }
}

impl Loc for p::NspaceItem {
    fn get_pos(&self) -> p::Pos {
        match self {
            p::NspaceItem::Structure(v) => v.pos.clone(),
            p::NspaceItem::Choice(v) => v.pos.clone(),
            p::NspaceItem::Service(v) => v.pos.clone(),
            p::NspaceItem::HttpService(v) => v.pos.clone(),
            p::NspaceItem::Nspace(v) => v.pos.clone(),
        }
    }
}

impl Loc for p::Nspace {
    fn get_pos(&self) -> p::Pos {
        self.pos.clone()
    }
}