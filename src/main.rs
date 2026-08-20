use crate::linked_list::DoublyLinkedList;

mod linked_list;

fn main() {
    let mut list = DoublyLinkedList::new();

    list.insert(2);

    println!(" length : {} ", list.length);

    list.insert(3);

    println!(" length : {} ", list.length);

    list.insert(4);
    println!(" length : {} ", list.length);

    list.delete(&2);

    println!(" length : {} ", list.length);
}
