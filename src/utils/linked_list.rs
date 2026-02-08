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
    prev: *mut Self,
    next: *mut Self,
}

impl<T> LinkedListNode<T> {
    pub const fn new() -> Self {
        Self {
            prev: null_mut(),
            next: null_mut(),
        }
    }
}

impl<T> LinkedListNode<T> {
    fn insert(&mut self, n: &mut Self) {
        let next = unsafe { &mut *self.next };

        n.prev = self;
        n.next = self.next;

        self.next = n;
        next.prev = n;
    }

    pub unsafe fn erase(&mut self) {
        let prev = unsafe { &mut *self.prev };
        let next = unsafe { &mut *self.next };

        prev.next = next;
        next.prev = prev;
    }
}

#[derive(Debug)]
pub struct LinkedList<T> {
    field_offs: usize,
    dummy: LinkedListNode<T>,
}

impl<T> LinkedList<T> {
    pub const fn new() -> Self {
        Self {
            field_offs: 0,
            dummy: LinkedListNode::new()
        }
    }

    pub fn init(&mut self, field_offs: usize) {
        let dummy_addr = &self.dummy as *const _ as *mut _;

        *self = Self {
            field_offs,
            dummy: LinkedListNode { prev: dummy_addr, next: dummy_addr }
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
    unsafe fn into_node(n: &mut T, offs: usize) -> &mut LinkedListNode<T> {
        &mut *((n as *const T).byte_add(offs) as *mut LinkedListNode<T>)
    }

    unsafe fn into_owner(n: &mut LinkedListNode<T>, offs: usize) -> &mut T {
        &mut *((n as *const LinkedListNode<T>).byte_sub(offs) as *mut T)
    }
}

impl<T> LinkedList<T> {
    pub fn push_front(&mut self, n: &mut T) {
        self.dummy.insert(unsafe { Self::into_node(n, self.field_offs) });
    }

    pub fn push_back(&mut self, n: &mut T) {
        unsafe {
            (*(self.dummy.prev)).insert(Self::into_node(n, self.field_offs));
        }
    }

    pub fn pop_front(&mut self) -> Option<&mut T> {
        if self.is_empty() { return None; }

        unsafe {
            let next = &mut *self.dummy.next;
            next.erase();

            Some(Self::into_owner(next, self.field_offs))
        }
    }

    pub fn pop_back(&mut self) -> Option<&mut T> {
        if self.is_empty() { return None; }

        unsafe {
            let prev = &mut *self.dummy.prev;
            prev.erase();

            Some(Self::into_owner(prev, self.field_offs))
        }
    }
}

impl<T> LinkedList<T> {
    pub fn is_empty(&self) -> bool {
        self.dummy.next as *const _ == &self.dummy
    }

    pub fn front(&self) -> Option<&T> {
        if self.is_empty() { return None; }

        unsafe { Some(Self::into_owner(&mut *self.dummy.next, self.field_offs)) }
    }

    pub fn back(&self) -> Option<&T> {
        if self.is_empty() { return None; }

        unsafe { Some(Self::into_owner(&mut *self.dummy.prev, self.field_offs)) }
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
        unsafe { self.pos.erase(); }
    }

    pub fn value(&mut self) -> &mut T {
        unsafe {
            LinkedList::into_owner(self.pos, self.field_offs)
        }
    }
}

impl <T> LinkedList<T> {
    pub fn iterate<F: Fn(&mut LinkedListIter<T>) -> Option<Ret>, Ret>(&mut self, f: F) -> Option<Ret> {
        unsafe {
            let mut iter = LinkedListIter {
                field_offs: self.field_offs,
                pos: &mut *self.dummy.next
            };

            loop {
                if iter.pos as *const _ == &self.dummy { break; }

                match f(&mut iter) {
                    Some(x) => return Some(x),

                    None => {
                        iter.pos = &mut *iter.pos.next;
                    }
                }
            }
        }

        None
    }
    
    pub fn iterate_reversed<F: Fn(&mut LinkedListIter<T>) -> Option<Ret>, Ret>(&mut self, f: F) -> Option<Ret> {
        unsafe {
            let mut iter = LinkedListIter {
                field_offs: self.field_offs,
                pos: &mut *self.dummy.prev
            };

            loop {
                if iter.pos as *const _ == &self.dummy { break; }

                match f(&mut iter) {
                    Some(x) => return Some(x),

                    None => {
                        iter.pos = &mut *iter.pos.prev;
                    }
                }
            }
        }

        None
    }
}
