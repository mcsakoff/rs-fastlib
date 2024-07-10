pub struct Stacked<T> {
    pub(crate) current: Option<T>,
    pub(crate) stack: Vec<T>,
}

impl<T> Stacked<T>  {
    pub fn new_empty() -> Self {
        Self {
            current: None,
            stack: Vec::new(),
        }
    }

    pub fn new(v: T) -> Self {
        Self {
            current: Some(v),
            stack: Vec::new(),
        }
    }

    pub fn push(&mut self, v: T) {
        match self.current.replace(v) {
            None => {}
            Some(old) => {
                self.stack.push(old);
            }
        };
    }

    pub fn pop(&mut self) -> Option<T> {
        let old = self.stack.pop();
        match old {
            None => self.current.take(),
            Some(v) => self.current.replace(v),
        }
    }

    pub fn peek(&self) -> Option<&T> {
        self.current.as_ref()
    }

    pub fn must_peek(&self) -> &T {
        self.peek().unwrap()
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.current.as_mut()
    }

    pub fn must_peek_mut(&mut self) -> &mut T {
        self.peek_mut().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_stacked() {
        let mut s: Stacked<u32> = Stacked::new_empty();
        assert_eq!(s.peek(), None);
        assert_eq!(s.current, None);
        assert_eq!(s.stack, vec![]);

        s.push(37);
        assert_eq!(s.peek(), Some(&37));
        assert_eq!(s.current, Some(37));
        assert_eq!(s.stack, vec![]);

        s.push(42);
        assert_eq!(s.peek(), Some(&42));
        assert_eq!(s.current, Some(42));
        assert_eq!(s.stack, vec![37]);

        let v = s.pop().unwrap();
        assert_eq!(v, 42);
        assert_eq!(s.peek(), Some(&37));
        assert_eq!(s.current, Some(37));
        assert_eq!(s.stack, vec![]);

        let v = s.pop().unwrap();
        assert_eq!(v, 37);
        assert_eq!(s.peek(), None);
        assert_eq!(s.current, None);
        assert_eq!(s.stack, vec![]);
    }
}
