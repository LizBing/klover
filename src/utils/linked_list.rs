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
// Safety: #[repr(C)]

use std::{fmt::Debug, marker::PhantomData, ops::Deref, pin::Pin, ptr::{NonNull, null_mut}};

#[derive(Debug)]
pub struct LinkedListNode {
    _prev: *mut Self,
    _next: *mut Self,
}

impl LinkedListNode {
    pub const fn new() -> Self {
        Self {
            _prev: null_mut(),
            _next: null_mut(),
        }
    }
}

impl LinkedListNode {
    pub fn insert(&mut self, n: &mut Self) {
        let next = unsafe { &mut *self._next };

        n._prev = self;
        n._next = self._next;

        self._next = n;
        next._prev = n;
    }

    pub fn erase(&mut self) {
        let prev = unsafe { &mut *self._prev };
        let next = unsafe { &mut *self._next };

        prev._next = next;
        next._prev = prev;
    }
}

#[derive(Debug)]
pub struct LinkedList<T> {
    __: PhantomData<T>,

    _field_offs: usize,
    _dummy: LinkedListNode,
}

impl<T> LinkedList<T> {
    pub fn new(field_offs: usize) -> Self {
        assert!(field_offs < size_of::<T>());

        Self {
            __: PhantomData,

            _field_offs: field_offs,
            _dummy: LinkedListNode::new()
        }
    }
}

#[macro_export]
macro_rules! create_ll {
    ($t:ty, $field_name:ident) => {
        crate::utils::linked_list::LinkedList::<$t>::new(std::mem::offset_of!($t, $field_name))
    };
}

impl<T> LinkedList<T> {
    unsafe fn into_node(n: &T, offs: usize) -> &mut LinkedListNode {
        &mut *((n as *const T).byte_add(offs) as *mut LinkedListNode)
    }

    unsafe fn into_owner(n: &LinkedListNode, offs: usize) -> &T {
        &*((n as *const LinkedListNode).byte_sub(offs) as *const T)
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

impl <T> LinkedList<T> {
    pub fn iterate<F: Fn(&T) -> Option<Ret>, Ret>(&self, f: F) -> Option<Ret> {
        let mut iter = self._dummy._next;
        loop {
            if iter as *const _ == &self._dummy { break; }

            unsafe {
                match f(Self::into_owner(&*iter, self._field_offs)) {
                    None => iter = (*iter)._next,

                    Some(x) => return Some(x)
                }
            }
        }

        None
    }
    
    pub fn iterate_reversed<F: Fn(&T) -> Option<Ret>, Ret>(&self, f: F) -> Option<Ret> {
        let mut iter = self._dummy._prev;
        loop {
            if iter as *const _ == &self._dummy { break; }

            unsafe {
                match f(Self::into_owner(&*iter, self._field_offs)) {
                    None => iter = (*iter)._prev,

                    Some(x) => return Some(x)
                }
            }
        }

        None
    }
}
