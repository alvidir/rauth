pub struct Sequence<'a> {
    root: Option<Box<Node<'a>>>,
}

impl Sequence<'a> {
    pub fn Next() -> i32 {

    }
}

struct Node<'a> {
    val: 'a i32,
    left: Option<Box<Node<'a>>>,
    right: Option<Box<Node<'a>>>,
}

impl<'a> Node<'a> {
    fn new(value: 'a i32) -> Node {
        Node {
            value: value,
            left: None,
            right: None,
        }
    }

    pub fn insert(&mut self, new_val: 'a i32) {
        if self.val == new_val {
            return
        }

        let target_node = if new_val < self.val { &mut self.left } else { &mut self.right };
        
        match target_node {
            &mut Some(ref mut subnode) => subnode.insert(new_val),
            &mut None => {
                let new_node = Node { val: new_val, l: None, r: None };
                let boxed_node = Some(Box::new(new_node));
                *target_node = boxed_node;
            }
        }
    }

    pub fn search(&self, target: i32) -> Option<i32> {
        match self.value {
            value if target == value => Some(value),
            value if target < value => self.left.as_ref()?.search(target),
            value if target > value => self.right.as_ref()?.search(target),
            _ => None,
        }
    }
}