use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

struct Node<T> {
    // value
    value: T,
    // previous Node
    prev: Option<Weak<RefCell<Node<T>>>>,
    // next node
    next: Option<Rc<RefCell<Node<T>>>>,
}

pub struct DoublyLinkedList<T> {
    // head
    head: Option<Rc<RefCell<Node<T>>>>,
    // tail
    tail: Option<Rc<RefCell<Node<T>>>>,
    // maybe length ??
    pub length: usize,
}

impl<T> DoublyLinkedList<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            length: 0,
        }
    }

    pub fn insert(&mut self, value: T) {
        let new_node = Rc::new(RefCell::new(Node {
            value,
            prev: None,
            next: None,
        }));

        self.length += 1;

        if self.head.is_none() {
            self.head = Some(Rc::clone(&new_node));
            self.tail = Some(new_node);

            return;
        }

        let old_tail = self.tail.as_ref().unwrap();

        new_node.borrow_mut().prev = Some(Rc::downgrade(old_tail));

        old_tail.borrow_mut().next = Some(Rc::clone(&new_node));

        self.tail = Some(new_node);
    }

    fn find(&self, value: &T) -> Option<Rc<RefCell<Node<T>>>>
    where
        T: PartialEq,
    {
        let mut current = self.head.clone();

        while let Some(node) = current {
            if node.borrow().value == *value {
                return Some(Rc::clone(&node));
            }

            current = node.borrow().next.clone();
        }

        None
    }

    pub fn delete(&mut self, value: &T) -> bool
    where
        T: PartialEq,
    {
        let node = match self.find(value) {
            Some(node) => node,
            None => return false,
        };

        let prev = node.borrow().prev.clone();
        let next = node.borrow().next.clone();

        match (prev, next) {
            (None, None) => {
                self.head = None;
                self.tail = None;
            }

            (None, Some(next_node)) => {
                next_node.borrow_mut().prev = None;
                self.head = Some(next_node);
            }

            (Some(prev_node), None) => {
                if let Some(prev_node) = prev_node.upgrade() {
                    prev_node.borrow_mut().next = None;
                    self.tail = Some(prev_node);
                }
            }

            (Some(prev_node), Some(next_node)) => {
                if let Some(prev_node) = prev_node.upgrade() {
                    prev_node.borrow_mut().next = Some(Rc::clone(&next_node));
                    next_node.borrow_mut().prev = Some(Rc::downgrade(&prev_node));
                }
            }
        }

        self.length -= 1;
        true
    }
}
