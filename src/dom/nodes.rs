use super::{Element, ElementStatus, NameSpace, Node, ParentAndChild, TextNode};
use crate::component::{Comp, Component, ComponentHandle};
use wasm_bindgen::UnwrapThrowExt;

#[derive(Default, Clone)]
pub struct Nodes(Vec<Node>);

impl std::fmt::Debug for Nodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("Nodes of {} items", self.0.len()))
    }
}

impl Nodes {
    pub fn count(&self) -> usize {
        self.0.len()
    }
    pub fn clear(&mut self, parent: &web_sys::Node) {
        self.0.drain(..).for_each(|mut node| node.clear(parent));
    }

    pub fn clear_vec(&mut self) {
        self.0.clear();
    }

    pub fn clear_after(&mut self, index: usize, parent: &web_sys::Node) {
        self.0
            .drain(index..)
            .for_each(|mut node| node.clear(parent));
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        self.0.iter().for_each(|node| node.append_to(parent));
    }

    pub fn get_element_mut(&mut self, index: usize) -> &mut Element {
        match self
            .0
            .get_mut(index)
            .expect_throw("dom::nodes::Nodes::get_element_mut")
        {
            Node::Element(element) => element,
            _ => panic!("dom::nodes::Nodes::get_element_mut expected Node::Element"),
        }
    }

    pub fn create_new_element_ns(
        &mut self,
        ns: Option<&'static str>,
        tag: &str,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        let e = Element::new_ns(ns, tag);
        e.insert_before(parent, next_sibling);
        self.0.push(Node::Element(e));
    }

    pub fn check_or_create_element<N: NameSpace>(
        &mut self,
        tag: &str,
        index: usize,
        parent_status: ElementStatus,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) -> ElementStatus {
        if index == self.0.len() {
            self.create_new_element_ns(N::NAMESPACE, tag, parent, next_sibling);
            ElementStatus::JustCreated
        } else {
            parent_status
        }
    }

    pub fn check_or_create_element_for_list<N: NameSpace>(
        &mut self,
        tag: &str,
        index: usize,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
        use_template: bool,
    ) -> ElementStatus {
        let item_count = self.0.len();
        if index < item_count {
            ElementStatus::Existing
        } else if !use_template || item_count == 0 {
            self.create_new_element_ns(N::NAMESPACE, tag, parent, next_sibling);
            ElementStatus::JustCreated
        } else {
            let element = self.0[0].clone();
            match &element {
                Node::Element(element) => element.insert_before(parent, next_sibling),
                _ => panic!(
                    "dom::nodes::Nodes::check_or_create_element_for_list expected Node::Element"
                ),
            }
            self.0.push(element);
            ElementStatus::JustCloned
        }
    }

    pub fn grouped_nodes(
        &mut self,
        index: usize,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) -> &mut GroupedNodes {
        if index == self.0.len() {
            let fnl = GroupedNodes::new();
            parent
                .insert_before(fnl.end_flag_node.as_ref(), next_sibling)
                .expect_throw("dom::nodes::Nodes::grouped_nodes insert_before");
            self.0.push(Node::GroupedNodes(fnl));
        }

        match self
            .0
            .get_mut(index)
            .expect_throw("dom::nodes::Nodes::grouped_nodes get_mut")
        {
            Node::GroupedNodes(grouped_node_list) => grouped_node_list,
            _ => panic!("dom::nodes::Nodes::grouped_nodes expected Node::GroupedNodes"),
        }
    }

    pub fn store_component_handle(&mut self, any: AnyComponentHandle) {
        let any = Node::ComponentHandle(any);
        if let Some(first) = self.0.first_mut() {
            *first = any;
        } else {
            self.0.push(any);
        }
    }

    pub fn get_first_element(&self) -> Option<&Element> {
        self.0.first().and_then(|n| n.get_first_element())
    }

    pub fn get_last_element(&self) -> Option<&Element> {
        self.0.last().and_then(|n| n.get_last_element())
    }

    pub fn static_text(
        &mut self,
        index: usize,
        text: &str,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        if index == self.0.len() {
            self.add_text_node(text, parent, next_sibling);
        }
    }

    pub fn update_text(
        &mut self,
        index: usize,
        text: &str,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        if index == self.0.len() {
            self.add_text_node(text, parent, next_sibling);
        } else {
            match self
                .0
                .get_mut(index)
                .expect_throw("dom::nodes::Nodes::update_text get_mut")
            {
                Node::Text(text_node) => text_node.update_text(text),
                _ => panic!("dom::nodes::Nodes::update_text expected Node::Text"),
            }
        }
    }

    fn add_text_node(
        &mut self,
        text: &str,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        let text = TextNode::new(text);
        text.insert_before(parent, next_sibling);
        self.0.push(Node::Text(text));
    }
}

pub struct GroupedNodes {
    active_index: Option<u32>,
    // `end_node` marks the boundary of this fragment
    end_flag_node: web_sys::Node,
    nodes: Nodes,
}

impl Clone for GroupedNodes {
    fn clone(&self) -> Self {
        // a GroupedNodes should not be cloned?
        Self::new()
    }
}

impl GroupedNodes {
    fn new() -> Self {
        let end_flag_node = crate::utils::document()
            .create_comment("Mark the end of a grouped node list")
            .into();
        Self {
            active_index: None,
            end_flag_node,
            nodes: Nodes::default(),
        }
    }

    pub fn set_active_index(&mut self, index: u32, parent: &web_sys::Node) -> ElementStatus {
        if Some(index) != self.active_index {
            self.nodes.clear(parent);
            self.active_index = Some(index);
            ElementStatus::JustCreated
        } else {
            ElementStatus::Existing
        }
    }

    pub fn clear(&mut self, parent: &web_sys::Node) {
        self.nodes.clear(parent);
        parent
            .remove_child(&self.end_flag_node)
            .expect_throw("dom::nodes::GroupedNodes::clear remove_child");
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        self.nodes.append_to(parent);
        parent
            .append_child(&self.end_flag_node)
            .expect_throw("dom::nodes::GroupedNodes::append_to append_child");
    }

    pub fn nodes(&self) -> &Nodes {
        &self.nodes
    }

    pub fn nodes_mut_and_end_flag_node(&mut self) -> (&mut Nodes, &web_sys::Node) {
        (&mut self.nodes, &self.end_flag_node)
    }
}

pub struct AnyComponentHandle(Box<dyn std::any::Any>);

impl<C: Component> From<Comp<C>> for AnyComponentHandle {
    fn from(ch: Comp<C>) -> Self {
        Self(Box::new(ComponentHandle::from(ch)))
    }
}
impl Clone for AnyComponentHandle {
    fn clone(&self) -> Self {
        //
        panic!("Spair does not support mounting a component inside a list item");
    }
}
