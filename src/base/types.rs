use std::cell::Cell;
use std::rc::Rc;

use roxmltree::Node;

use crate::{Error, Result};
use crate::base::instruction::Instruction;

/// A template contains a sequence of instructions. The order of the instructions is significant and corresponds
/// to the order of the data in the stream.
pub(crate) struct Template {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) type_ref: TypeRef,
    pub(crate) dictionary: Dictionary,
    pub(crate) instructions: Vec<Instruction>,

    // This flag indicates if the template requires a presence map in case of statically referenced
    // from another template. If the flag is None, the presence map is not calculated yet.
    pub(crate) require_pmap: Cell<Option<bool>>,
}

impl Template {
    pub(crate) fn from_node(node: Node) -> Result<Self> {
        if node.tag_name().name() != "template" {
            return Err(Error::Static(format!("expected <template/> node, got <{}/>", node.tag_name().name())));
        }
        let id = node
            .attribute("id")
            .or(Some("0"))
            .unwrap()
            .parse::<u32>()?;
        let name = node
            .attribute("name")
            .ok_or_else(|| Error::Static("template name not found".to_string()))?
            .to_string();
        let type_ref = node
            .attribute("typeRef")
            .map(|d| TypeRef::from_str(d))
            .unwrap_or(TypeRef::Any);
        let dictionary = node
            .attribute("dictionary")
            .map(|d| Dictionary::from_str(d))
            .unwrap_or(Dictionary::Global);
        let mut instructions = Vec::new();
        for child in node.children() {
            if child.is_element() {
                instructions.push(Instruction::from_node(child)?);
            }
        }
        Ok(Self {
            id,
            name,
            type_ref,
            dictionary,
            instructions,
            require_pmap: Cell::new(None),
        })
    }
}


/// Field operators specify ways to optimize the encoding of a field.
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Operator {
    None,
    Constant,
    Default,
    Copy,
    Increment,
    Delta,
    Tail,
}

impl Operator {
    pub(crate) fn new_from_tag(t: &str) -> Result<Self> {
        match t {
            "constant" => Ok(Self::Constant),
            "default" => Ok(Self::Default),
            "copy" => Ok(Self::Copy),
            "increment" => Ok(Self::Increment),
            "delta" => Ok(Self::Delta),
            "tail" => Ok(Self::Tail),
            _ => Err(Error::Static(format!("Unknown operator: {}", t))),
        }
    }
}


/// The optional presence attribute indicates whether the field is mandatory or optional.
#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum Presence {
    Mandatory,
    Optional,
}

impl Presence {
    pub(crate) fn from_str(s: &str) -> Result<Self> {
        match s {
            "mandatory" => Ok(Self::Mandatory),
            "optional" => Ok(Self::Optional),
            _ => Err(Error::Static(format!("unknown presence: {s}"))),
        }
    }
}


/// The dictionary name is specified by the dictionary attribute on the field operator element.
/// There are three predefined dictionaries: "global", "template" and "type".
/// "inherit" means that the dictionary name is inherited from the parent element.
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Dictionary {
    Inherit,
    Global,
    Template,
    Type,
    UserDefined(Rc<str>),
}

impl Dictionary {
    pub(crate) fn from_str(name: &str) -> Self {
        match name {
            "global" => Self::Global,
            "template" => Self::Template,
            "type" => Self::Type,
            _ => Self::UserDefined(Rc::from(name)),
        }
    }
}


/// The current application type initially the special type any.
/// The current application type changes when the processor encounters an element containing a "typeRef" element.
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum TypeRef {
    Any,
    ApplicationType(Rc<str>),
}

impl TypeRef {
    pub(crate) fn from_str(name: &str) -> Self {
        Self::ApplicationType(Rc::from(name))
    }
}
