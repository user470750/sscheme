//! Runtime environments: chains of frames that hold local variables.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Value;

/// A shared frame: closures keep the frame they were created in alive.
pub type Env = Rc<Frame>;

/// The arguments of one call, linked to the frame of the enclosing `lambda`.
///
/// Frames that refer to each other through closures form `Rc` cycles and are
/// never freed.
pub struct Frame {
    slots: RefCell<Vec<Value>>,
    parent: Option<Env>,
}

impl Frame {
    pub(crate) fn new(slots: Vec<Value>, parent: Option<Env>) -> Env {
        Rc::new(Self {
            slots: RefCell::new(slots),
            parent,
        })
    }

    /// Reads the local `index` of the frame `depth` levels up.
    pub(crate) fn get(&self, depth: usize, index: usize) -> Value {
        self.ancestor(depth).slots.borrow()[index].clone()
    }

    /// Assigns the local `index` of the frame `depth` levels up.
    pub(crate) fn set(&self, depth: usize, index: usize, value: Value) {
        self.ancestor(depth).slots.borrow_mut()[index] = value;
    }

    fn ancestor(&self, depth: usize) -> &Frame {
        let mut frame = self;
        for _ in 0..depth {
            frame = frame
                .parent
                .as_deref()
                .expect("compiler resolves locals within the enclosing frames");
        }
        frame
    }
}
