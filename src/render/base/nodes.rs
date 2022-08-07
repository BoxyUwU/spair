use super::{ElementRender, ListRender};
use crate::{
    component::{Comp, Component},
    dom::{ElementStatus, GroupedNodes, NameSpace, Nodes},
};

pub trait NodesRenderMut<C: Component> {
    fn nodes_render_mut(&mut self) -> &mut NodesRender<C>;
}

pub struct NodesRender<'a, C: Component> {
    comp: &'a Comp<C>,
    state: &'a C,

    update_mode: bool,
    index: usize,
    parent_status: ElementStatus,
    parent: &'a web_sys::Node,
    next_sibling: Option<&'a web_sys::Node>,
    nodes: &'a mut Nodes,
}

impl<'a, C: Component> From<ElementRender<'a, C>> for NodesRender<'a, C> {
    fn from(er: ElementRender<'a, C>) -> Self {
        let (comp, state, parent_status, element) = er.into_parts();
        let (parent, nodes) = element.ws_node_and_nodes_mut();
        Self {
            comp,
            state,

            update_mode: true,
            index: 0,
            parent_status,
            parent,
            next_sibling: None,
            nodes,
        }
    }
}

impl<'a, C: Component> NodesRender<'a, C> {
    pub fn state(&self) -> &'a C {
        self.state
    }

    pub fn comp(&self) -> Comp<C> {
        self.comp.clone()
    }

    pub fn parent(&self) -> &web_sys::Node {
        self.parent
    }

    pub fn set_static_mode(&mut self) {
        self.update_mode = false;
    }

    pub fn set_update_mode(&mut self) {
        self.update_mode = true;
    }

    pub fn require_render(&self) -> bool {
        if self.update_mode {
            true
        } else {
            self.parent_status == ElementStatus::JustCreated
        }
    }

    pub fn next_index(&mut self) {
        self.index += 1;
    }

    pub fn update_text(&mut self, text: &str) {
        self.nodes
            .update_text(self.index, text, self.parent, self.next_sibling);
        self.index += 1;
    }

    pub fn static_text(&mut self, text: &str) {
        self.nodes
            .static_text(self.index, text, self.parent, self.next_sibling);
        self.index += 1;
    }

    pub fn get_element_render<N: NameSpace>(&mut self, tag: &str) -> ElementRender<C> {
        let status = self.nodes.check_or_create_element::<N>(
            tag,
            self.index,
            self.parent_status,
            self.parent,
            self.next_sibling,
        );
        let element = self.nodes.get_element_mut(self.index);
        // Don't do this here, because .get_element_render() is not always called
        // self.index += 1;
        ElementRender::new(self.comp, self.state, element, status)
    }

    pub fn get_match_if_render(&mut self) -> MatchIfRender<C> {
        let match_if = self
            .nodes
            .grouped_nodes(self.index, self.parent, self.next_sibling);
        self.index += 1;
        MatchIfRender {
            comp: self.comp,
            state: self.state,
            parent: self.parent,
            match_if,
        }
    }

    pub fn get_list_render(&mut self, tag: &'a str, use_template: bool) -> ListRender<C> {
        let gn = self
            .nodes
            .grouped_nodes(self.index, self.parent, self.next_sibling);
        self.index += 1;
        let (list, next_sibling) = gn.nodes_mut_and_end_flag_node();
        ListRender::new(
            self.comp,
            self.state,
            list,
            tag,
            self.parent,
            Some(next_sibling),
            use_template,
        )
    }
}

pub struct MatchIfRender<'a, C: Component> {
    comp: &'a Comp<C>,
    state: &'a C,

    parent: &'a web_sys::Node,
    match_if: &'a mut GroupedNodes,
}

impl<'a, C: Component> MatchIfRender<'a, C> {
    pub fn render_on_arm_index(self, index: u32) -> NodesRender<'a, C> {
        let status = self.match_if.set_active_index(index, self.parent);
        let (nodes, next_sibling) = self.match_if.nodes_mut_and_end_flag_node();

        NodesRender {
            comp: self.comp,
            state: self.state,

            update_mode: true,
            index: 0,
            parent_status: status,
            parent: self.parent,
            next_sibling: Some(next_sibling),
            nodes,
        }
    }
}
