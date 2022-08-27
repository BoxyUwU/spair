use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;

pub trait ListRender<I> {
    fn render(&mut self, items: &[I], diffs: &[Diff<I>]);
    fn unmounted(&self) -> bool;
}

pub struct QrVec<I>(Rc<RefCell<QrVecContent<I>>>);

impl<I> Clone for QrVec<I> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<I> QrVec<I> {
    pub(crate) fn content(&self) -> &Rc<RefCell<QrVecContent<I>>> {
        &self.0
    }
}

pub struct QrVecContent<I> {
    values: Vec<I>,
    a_render_is_queued: bool,
    diffs: Vec<Diff<I>>,
    // TODO: remove dropped renders
    renders: Vec<Box<dyn ListRender<I>>>,
}

impl<I> QrVecContent<I> {
    pub fn add_render(&mut self, r: Box<dyn ListRender<I>>) {
        self.renders.push(r);
    }

    fn render(&mut self) {
        for r in self.renders.iter_mut() {
            r.render(&self.values, &self.diffs);
        }
        self.diffs.clear();
        self.a_render_is_queued = false;
    }
}

// To support multi-changes, we have to store a copy of the item for some change here.
pub enum Diff<I> {
    New,
    Push(I),
    Pop,
    Insert { index: usize, value: I },
    RemoveAtIndex(usize),
    Change { index: usize, new_value: I },
    Move { old_index: usize, new_index: usize },
    Swap { index_1: usize, index_2: usize },
    Clear,
}

impl<I: 'static> QrVec<I> {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(QrVecContent {
            values: Vec::new(),
            a_render_is_queued: false,
            diffs: Vec::new(),
            renders: Vec::new(),
        })))
    }

    pub(crate) fn queue_me(&self) {
        {
            let mut this = self
                .0
                .try_borrow_mut()
                .expect_throw("queue_render::vec::QrVec::queue_me try_borrow_mut");
            if this.a_render_is_queued {
                return;
            }
            this.a_render_is_queued = true;
        }
        let this = self.clone();
        super::queue_render(move || this.render());
    }

    fn render(&self) {
        match self.0.try_borrow_mut() {
            Ok(mut this) => this.render(),
            Err(e) => log::error!("queue_render::vec::QrVec::render {}", e),
        }
    }
}

impl<I: 'static + Clone> QrVec<I> {
    pub fn pop(&mut self) -> Option<I> {
        None
    }
}
