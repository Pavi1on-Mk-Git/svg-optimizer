use super::common::constants::*;
use super::common::id::make_id_usage_map;
use super::common::iter::EasyIter;
use crate::node::Node;
use anyhow::Result;
use std::collections::BTreeMap;
use xml::attribute::OwnedAttribute;

fn is_attribute_useless_id(
    attribute: &OwnedAttribute,
    id_usage_map: &BTreeMap<String, bool>,
) -> bool {
    attribute.name.local_name == ID_NAME && !id_usage_map[&attribute.value]
}

fn remove_useless_ids_for_node(node: Node, id_usage_map: &BTreeMap<String, bool>) -> Node {
    match node {
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => Node::RegularNode {
            node_type,
            namespace,
            attributes: attributes
                .filter(|attribute| !is_attribute_useless_id(attribute, id_usage_map)),
            children: children.map(|child| remove_useless_ids_for_node(child, id_usage_map)),
        },
        other => other,
    }
}

pub fn remove_useless_ids(nodes: Vec<Node>) -> Result<Vec<Node>> {
    let id_usage_map = make_id_usage_map(&nodes);
    Ok(nodes.map(|node| remove_useless_ids_for_node(node, &id_usage_map)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_remove_useless_ids,
        remove_useless_ids,
        r##"
        <svg width="120" height="120" viewBox="0 0 120 120" xmlns="http://www.w3.org/2000/svg">
        <style>
            #smallRect {
                stroke: #000066;
                fill: #00cc00;
            }
        </style>

        <use href="#bigRect" x="10" fill="blue"/>
        <rect id="smallRect" x="10" y="10" width="100" height="100"/>
        <rect id="bigRect" x="10" y="10" width="100" height="100"/>
        <rect id="thirdRect" x="10" y="10" width="100" height="100"/>
        </svg>
        "##,
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 120 120">
        <style>
            #smallRect {
                stroke: #000066;
                fill: #00cc00;
            }
        </style>

        <use href="#bigRect" x="10" fill="blue"/>
        <rect id="smallRect" x="10" y="10" width="100" height="100"/>
        <rect id="bigRect" x="10" y="10" width="100" height="100"/>
        <rect x="10" y="10" width="100" height="100"/>
        </svg>
        "##
    );

    test_optimize!(
        test_remove_useless_ids_style_attribute,
        remove_useless_ids,
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" xmlns:cc="http://creativecommons.org/ns#" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd" xmlns:svg="http://www.w3.org/2000/svg" width="3.1in" height="0.9in" viewBox="-12 -12 3699 1074" id="svg2" version="1.1" inkscape:version="0.48.4 r9939" sodipodi:docname="TreeMapping.fig">
        <defs id="defs64">
        <marker inkscape:stockid="Arrow2Lend" orient="auto" refY="0.0" refX="0.0" id="Arrow2Lend" style="overflow:visible;">
        <path id="path3836" style="fill-rule:evenodd;stroke-width:0.62500000;stroke-linejoin:round;" d="M 8.7185878,4.0337352 L -2.2072895,0.016013256 L 8.7185884,-4.0017078 C 6.9730900,-1.6296469 6.9831476,1.6157441 8.7185878,4.0337352 z " transform="scale(1.1) rotate(180) translate(1,0)"/></marker>
        <marker inkscape:stockid="Arrow2Mend" orient="auto" refY="0.0" refX="0.0" id="Arrow2Mend" style="overflow:visible;">
        <path id="path3842" style="fill-rule:evenodd;stroke-width:0.62500000;stroke-linejoin:round;" d="M 8.7185878,4.0337352 L -2.2072895,0.016013256 L 8.7185884,-4.0017078 C 6.9730900,-1.6296469 6.9831476,1.6157441 8.7185878,4.0337352 z " transform="scale(0.6) rotate(180) translate(0,0)"/>
        </marker>
        <marker inkscape:stockid="Arrow1Mend" orient="auto" refY="0.0" refX="0.0" id="Arrow1Mend" style="overflow:visible;">
        <path id="path3824" d="M 0.0,0.0 L 5.0,-5.0 L -12.5,0.0 L 5.0,5.0 L 0.0,0.0 z " style="fill-rule:evenodd;stroke:#000000;stroke-width:1.0pt;" transform="scale(0.4) rotate(180) translate(10,0)"/>
        </marker>
        </defs>
        <polyline points="675,375 675,150 300,150 300,358 " style="stroke:#000000;stroke-width:7.00088889;stroke-linejoin:miter;stroke-linecap:butt;stroke-miterlimit:4;stroke-dasharray:none;marker-end:url(#Arrow2Lend)" id="polyline20"/>
        </svg>
        "##,
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" xmlns:cc="http://creativecommons.org/ns#" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd" xmlns:svg="http://www.w3.org/2000/svg" width="3.1in" height="0.9in" viewBox="-12 -12 3699 1074" version="1.1" inkscape:version="0.48.4 r9939" sodipodi:docname="TreeMapping.fig">
        <defs>
        <marker inkscape:stockid="Arrow2Lend" orient="auto" refY="0.0" refX="0.0" id="Arrow2Lend" style="overflow:visible;">
        <path style="fill-rule:evenodd;stroke-width:0.62500000;stroke-linejoin:round;" d="M 8.7185878,4.0337352 L -2.2072895,0.016013256 L 8.7185884,-4.0017078 C 6.9730900,-1.6296469 6.9831476,1.6157441 8.7185878,4.0337352 z " transform="scale(1.1) rotate(180) translate(1,0)"/></marker>
        <marker inkscape:stockid="Arrow2Mend" orient="auto" refY="0.0" refX="0.0" style="overflow:visible;">
        <path style="fill-rule:evenodd;stroke-width:0.62500000;stroke-linejoin:round;" d="M 8.7185878,4.0337352 L -2.2072895,0.016013256 L 8.7185884,-4.0017078 C 6.9730900,-1.6296469 6.9831476,1.6157441 8.7185878,4.0337352 z " transform="scale(0.6) rotate(180) translate(0,0)"/>
        </marker>
        <marker inkscape:stockid="Arrow1Mend" orient="auto" refY="0.0" refX="0.0" style="overflow:visible;">
        <path d="M 0.0,0.0 L 5.0,-5.0 L -12.5,0.0 L 5.0,5.0 L 0.0,0.0 z " style="fill-rule:evenodd;stroke:#000000;stroke-width:1.0pt;" transform="scale(0.4) rotate(180) translate(10,0)"/>
        </marker>
        </defs>
        <polyline points="675,375 675,150 300,150 300,358 " style="stroke:#000000;stroke-width:7.00088889;stroke-linejoin:miter;stroke-linecap:butt;stroke-miterlimit:4;stroke-dasharray:none;marker-end:url(#Arrow2Lend)"/>
        </svg>
        "##
    );
}
