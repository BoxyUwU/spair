use crate::{
    component::Component,
    queue_render::{
        dom::{QrTextNode, QrTextNodeMap},
        value::{QrVal, QrValMap},
    },
    render::{
        base::NodesUpdaterMut,
        html::{Nodes, Render},
    },
};

impl<'n, 'h, C: Component> Nodes<'n, 'h, C> {
    pub fn create_qr_text_node(mut self) -> Option<QrTextNode> {
        self.nodes_updater_mut().create_qr_text_node()
    }
}

impl<C, T> Render<C> for &QrVal<T>
where
    C: Component,
    T: 'static + ToString,
{
    fn render(self, nodes: Nodes<C>) {
        if let Some(text_node) = nodes.create_qr_text_node() {
            match self.content().try_borrow_mut() {
                Ok(mut this) => {
                    text_node.update_text(&this.value().to_string());
                    this.add_render(Box::new(text_node));
                }
                Err(e) => log::error!("{}", e),
            }
        }
    }
}

impl<C, T, U> Render<C> for QrValMap<C, T, U>
where
    C: Component,
    T: 'static + ToString,
    U: 'static + ToString,
{
    fn render(self, nodes: Nodes<C>) {
        let state = nodes.state();
        let comp = nodes.comp();
        if let Some(text_node) = nodes.create_qr_text_node() {
            let (value, fn_map) = self.into_parts();
            let map_node = QrTextNodeMap::new(text_node, comp, fn_map);
            match value.content().try_borrow_mut() {
                Ok(mut this) => {
                    let u = map_node.map_with_state(state, this.value());
                    map_node.update_text(&u.to_string());
                    this.add_render(Box::new(map_node));
                }
                Err(e) => log::error!("{}", e),
            };
        }
    }
}
