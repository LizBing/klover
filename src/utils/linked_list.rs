/*
 * Copyright 2026 Lei Zaakjyu
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

// Intrusive Doubly Linked List

use std::{fmt::Debug, ops::{Deref, DerefMut}, ptr::null_mut};

#[derive(Debug)]
pub struct LinkedListNode<T> {
    _prev: *mut Self,
    _next: *mut Self,
}

impl<T> LinkedListNode<T> {
    pub const fn new() -> Self {
        Self {
            _prev: null_mut(),
            _next: null_mut(),
        }
    }
}

impl<T> LinkedListNode<T> {
    fn insert(&mut self, n: &mut Self) {
        let next = unsafe { &mut *self._next };

        n._prev = self;
        n._next = self._next;

        self._next = n;
        next._prev = n;
    }

    fn erase(&mut self) {
        let prev = unsafe { &mut *self._prev };
        let next = unsafe { &mut *self._next };

        prev._next = next;
        next._prev = prev;
    }
}


#[derive(Debug)]
pub struct LinkedList<T> {
    _field_offs: usize,
    _dummy: LinkedListNode<T>,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self {
            _field_offs: 0,
            _dummy: LinkedListNode::new()
        }
    }

    pub fn init(&mut self, field_offs: usize) {
        let dummy_addr = &self._dummy as *const _ as *mut _;

        *self = Self {
            _field_offs: field_offs,
            _dummy: LinkedListNode { _prev: dummy_addr, _next: dummy_addr }
        };
    }
}

#[macro_export]
macro_rules! init_ll {
    ($this:expr, $t:ty, $field_name:ident) => {
        crate::utils::linked_list::LinkedList::init($this, std::mem::offset_of!($t, $field_name))
    }
}

impl<T> LinkedList<T> {
    unsafe fn into_node(n: &T, offs: usize) -> &mut LinkedListNode<T> {
        &mut *((n as *const T).byte_add(offs) as *mut LinkedListNode<T>)
    }

    unsafe fn into_owner(n: &LinkedListNode<T>, offs: usize) -> &T {
        &*((n as *const LinkedListNode<T>).byte_sub(offs) as *const T)
    }
}

impl<T> LinkedList<T> {
    pub fn push_front(&mut self, n: &T) {
        self._dummy.insert(unsafe { Self::into_node(n, self._field_offs) });
    }

    pub fn push_back(&mut self, n: &T) {
        unsafe {
            (*(self._dummy._prev)).insert(Self::into_node(n, self._field_offs));
        }
    }

    pub fn pop_front(&mut self) -> Option<&T> {
        if self.is_empty() { return None; }

        unsafe {
            let next = &mut *self._dummy._next;
            next.erase();

            Some(Self::into_owner(next, self._field_offs))
        }
    }

    pub fn pop_back(&mut self) -> Option<&T> {
        if self.is_empty() { return None; }

        unsafe {
            let prev = &mut *self._dummy._prev;
            prev.erase();

            Some(Self::into_owner(prev, self._field_offs))
        }
    }
}

impl<T> LinkedList<T> {
    pub fn is_empty(&self) -> bool {
        self._dummy._next as *const _ == &self._dummy
    }
}

pub struct LinkedListIter<'a, T> {
    field_offs: usize,
    pos: &'a mut LinkedListNode<T>
}

impl<T> LinkedListIter<'_, T> {
    pub fn insert(&mut self, n: &mut LinkedListNode<T>) {
        self.pos.insert(n);
    }

    pub fn erase(&mut self) {
        self.pos.erase();
    }

    pub fn value(&self) -> &T {
        unsafe {
            LinkedList::into_owner(self.pos, self.field_offs)
        }
    }
}

impl <T> LinkedList<T> {
    pub fn iterate<F: Fn(&LinkedListIter<T>) -> Option<Ret>, Ret>(&self, f: F) -> Option<Ret> {
        unsafe {
            let mut iter = LinkedListIter {
                field_offs: self._field_offs,
                pos: &mut *self._dummy._next
            };

            loop {
                if iter.pos as *const _ == &self._dummy { break; }

                match f(&iter) {
                    Some(x) => return Some(x),

                    None => {
                        iter.pos = &mut *iter.pos._next;
                    }
                }
            }
        }

        None
    }
    
    pub fn iterate_reversed<F: Fn(&LinkedListIter<T>) -> Option<Ret>, Ret>(&self, f: F) -> Option<Ret> {
        unsafe {
            let mut iter = LinkedListIter {
                field_offs: self._field_offs,
                pos: &mut *self._dummy._prev
            };

            loop {
                if iter.pos as *const _ == &self._dummy { break; }

                match f(&iter) {
                    Some(x) => return Some(x),

                    None => {
                        iter.pos = &mut *iter.pos._prev;
                    }
                }
            }
        }

        None
    }
}
