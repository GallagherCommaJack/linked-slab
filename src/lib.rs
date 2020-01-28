use slab::*;
use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone)]
pub struct NodeId(usize);

#[derive(Copy, Clone)]
pub struct Node<T> {
    item: T,
    next: Option<usize>,
    prev: Option<usize>,
}

impl<T> Deref for Node<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.item
    }
}

impl<T> DerefMut for Node<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.item
    }
}

impl<T> Node<T> {
    pub fn into_inner(self) -> T {
        self.item
    }

    pub fn next(&self) -> Option<NodeId> {
        Some(NodeId(self.next?))
    }

    pub fn prev(&self) -> Option<NodeId> {
        Some(NodeId(self.prev?))
    }
}

#[derive(Clone)]
pub struct List<T> {
    inner: Slab<Node<T>>,
    init: Option<usize>,
    last: Option<usize>,
}

impl<T> Default for List<T> {
    fn default() -> Self {
        List {
            inner: Slab::new(),
            init: None,
            last: None,
        }
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(cap: usize) -> Self {
        List {
            inner: Slab::with_capacity(cap),
            ..Self::default()
        }
    }

    pub fn get(&self, id: NodeId) -> Option<&Node<T>> {
        self.inner.get(id.0)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node<T>> {
        self.inner.get_mut(id.0)
    }

    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        self.inner.reserve_exact(additional);
    }

    pub fn push_front(&mut self, item: T) -> NodeId {
        let vacant = self.inner.vacant_entry();
        let key = vacant.key();
        let next = self.init.replace(key);
        vacant.insert(Node {
            item,
            next,
            prev: None,
        });
        if let Some(ix) = next {
            debug_assert!(self.inner.contains(ix));
            let next = unsafe { self.inner.get_unchecked_mut(ix) };
            let old_prev = next.prev.replace(key);
            debug_assert_eq!(old_prev, None);
        }
        NodeId(key)
    }

    pub fn push_back(&mut self, item: T) -> NodeId {
        let vacant = self.inner.vacant_entry();
        let key = vacant.key();
        let prev = self.last.replace(key);
        vacant.insert(Node {
            item,
            prev,
            next: None,
        });
        if let Some(ix) = prev {
            debug_assert!(self.inner.contains(ix));
            let prev = unsafe { self.inner.get_unchecked_mut(ix) };
            let old_next = prev.next.replace(key);
            debug_assert_eq!(old_next, None);
        }
        NodeId(key)
    }

    pub fn remove(&mut self, id: NodeId) -> Option<Node<T>> {
        if !self.inner.contains(id.0) {
            return None;
        }

        let node = self.inner.remove(id.0);

        if let Some(prev_ix) = node.prev {
            debug_assert!(self.inner.contains(prev_ix));
            let prev = unsafe { self.inner.get_unchecked_mut(prev_ix) };
            debug_assert_eq!(prev.next, Some(id.0));
            prev.next = node.next;
        } else {
            self.init = node.next;
        }

        if let Some(next_ix) = node.next {
            debug_assert!(self.inner.contains(next_ix));
            let next = unsafe { self.inner.get_unchecked_mut(next_ix) };
            debug_assert_eq!(next.prev, Some(id.0));
            next.prev = node.prev;
        } else {
            self.last = node.prev;
        }

        Some(node)
    }

    pub fn init(&self) -> Option<NodeId> {
        Some(NodeId(self.init?))
    }

    pub fn last(&self) -> Option<NodeId> {
        Some(NodeId(self.last?))
    }

    pub fn pop_front(&mut self) -> Option<Node<T>> {
        self.remove(self.init()?)
    }

    pub fn pop_back(&mut self) -> Option<Node<T>> {
        self.remove(self.last()?)
    }

    pub fn contains(&self, id: NodeId) -> bool {
        self.inner.contains(id.0)
    }

    pub fn cursor_at(&self, id: NodeId) -> Option<Cursor<T>> {
        if self.contains(id) {
            Some(Cursor {
                current: id,
                backing: self,
            })
        } else {
            None
        }
    }

    pub fn cursor_mut_at(&mut self, id: NodeId) -> Option<CursorMut<T>> {
        if self.contains(id) {
            Some(CursorMut {
                current: id,
                backing: self,
            })
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
pub struct Cursor<'a, T> {
    current: NodeId,
    backing: &'a List<T>,
}

impl<'a, T> Cursor<'a, T> {
    pub fn current(&self) -> &'a Node<T> {
        unsafe { self.backing.inner.get_unchecked(self.current.0) }
    }

    pub fn try_next(&mut self) -> bool {
        let node = self.current();
        if let Some(ix) = node.next {
            self.current = NodeId(ix);
            true
        } else {
            false
        }
    }

    pub fn try_prev(&mut self) -> bool {
        let node = self.current();
        if let Some(ix) = node.prev {
            self.current = NodeId(ix);
            true
        } else {
            false
        }
    }
}

pub struct CursorMut<'a, T> {
    current: NodeId,
    backing: &'a mut List<T>,
}

impl<'a, T> CursorMut<'a, T> {
    pub fn current(&mut self) -> &mut Node<T> {
        unsafe { self.backing.inner.get_unchecked_mut(self.current.0) }
    }

    pub fn try_next(&mut self) -> bool {
        let node = self.current();
        if let Some(ix) = node.next {
            self.current = NodeId(ix);
            true
        } else {
            false
        }
    }

    pub fn try_prev(&mut self) -> bool {
        let node = self.current();
        if let Some(ix) = node.prev {
            self.current = NodeId(ix);
            true
        } else {
            false
        }
    }
}
