use super::constants::ID_NAME;
use crate::node::Node;
use xml::attribute::OwnedAttribute;

fn find_id(attributes: &[OwnedAttribute]) -> Option<String> {
    attributes
        .iter()
        .find(|attr| attr.name.local_name == ID_NAME)
        .map(|id| id.value.clone())
}

pub fn find_ids_for_subtree(nodes: &Vec<Node>) -> Vec<String> {
    let mut ids = vec![];

    for node in nodes {
        if let Node::RegularNode {
            attributes,
            children,
            ..
        } = node
        {
            if let Some(id) = find_id(attributes) {
                ids.push(id);
            }

            ids.extend(find_ids_for_subtree(children));
        }
    }

    ids
}
