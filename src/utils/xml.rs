//! Helpers for reading template definitions XML.
use std::rc::Rc;

use roxmltree::Node;

use crate::{Error, Result};

/// The namespace URI of template definition elements.
pub(crate) const TD_NAMESPACE: &str = "http://www.fixprotocol.org/ns/fast/td/1.1";

/// Returns `true` if the node is a template definition element.
/// Elements without namespace are treated as template definition elements too.
/// Other elements are foreign and must be ignored.
pub(crate) fn is_td_element(node: &Node) -> bool {
    node.is_element()
        && node
        .tag_name()
        .namespace()
        .is_none_or(|ns| ns == TD_NAMESPACE)
}

/// Returns template definition child elements of the node, skipping foreign elements.
pub(crate) fn td_children<'a, 'input>(
    node: &Node<'a, 'input>,
) -> impl Iterator<Item = Node<'a, 'input>> {
    node.children().filter(is_td_element)
}

/// Returns the value of the attribute on the nearest element in the ancestry (including the node itself)
/// that has it, or the empty string if none has.
/// Used for the inherited `ns` and `templateNs` attributes.
pub(crate) fn inherited_attribute(node: &Node, name: &str) -> Rc<str> {
    Rc::from(
        node.ancestors()
            .find_map(|n| n.attribute(name))
            .unwrap_or_default(),
    )
}

/// Parses the optional auxiliary identifier (`id` attribute); 0 if not specified.
/// # Errors
/// Returns error if the identifier is not a number.
pub(crate) fn parse_id(node: &Node) -> Result<u32> {
    let Some(id) = node.attribute("id") else {
        return Ok(0);
    };
    id.trim().parse::<u32>().map_err(|_| {
        Error::Static(format!(
            "<{}/> has unsupported 'id' attribute '{id}', only unsigned integers are supported",
            node.tag_name().name()
        ))
    })
}
