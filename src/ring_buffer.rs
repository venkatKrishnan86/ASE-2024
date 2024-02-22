#[derive(Debug)]
pub struct RingBuffer<T> {
    ring_buffer: Vec<T>,
    capacity: usize,
    head: Option<usize>,
    tail: Option<usize>,
}

impl<T: Copy + Default> RingBuffer<T> {
    pub fn new(length: usize) -> Self {
        // Create a new RingBuffer with `length` slots and "default" values.
        // Hint: look into `vec!` and the `Default` trait.
        Self {
            ring_buffer: vec![T::default(); length],
            capacity: length,
            head: None,
            tail: None,
        }
    }

    pub fn reset(&mut self) {
        // Clear internal buffer and reset indices.
        self.head = None;
        self.tail = None;
    }

    // `put` and `peek` write/read without advancing the indices.
    pub fn put(&mut self, value: T) {
        match self.tail {
            None => (),
            Some(t) => self.ring_buffer[t] = value
        }
    }

    pub fn peek(&self) -> Option<T> {
        match self.tail {
            None => None,
            Some(t) => Some(self.ring_buffer[t])
        }
    }

    pub fn get(&self, offset: usize) -> Option<T> {
        if offset >= self.len() - 1{ return None }
        match self.head {
            None => None,
            Some(t) => Some(self.ring_buffer[(t+offset)%self.capacity])
        }
    }

    /// Pushes a value onto the ring buffer. Also this operation advances the tail index by one.
    ///
    /// Note: If the buffer is full, nothing will happen.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to be pushed onto the buffer.
    ///
    /// # Returns
    ///
    /// * ()
    ///
    /// # Examples
    ///
    /// ```
    /// let mut buffer = RingBuffer::<i16>::new(3);
    /// for i in 1..5 {
    ///     buffer.push(i);
    /// }
    /// 
    /// assert_eq!(vec![1,2,3], buffer.ring_buffer);
    /// ```
    pub fn push(&mut self, value: T) {
        match self.len() == self.capacity() {
            true => (),
            false => match self.tail {
                None => {
                    self.head = Some(0);
                    self.tail = Some(0);
                    self.ring_buffer[0] = value;
                },
                Some(t) => {
                    self.tail = Some((t+1)%self.capacity);
                    self.ring_buffer[self.tail.unwrap() as usize] = value;
                }
            }
        }
        
    }

    pub fn pop(&mut self) -> Option<T> {
        match self.len() == 0 {
            true => None,
            false => {
                let popped_val = self.ring_buffer[self.head.unwrap()];
                if self.head.unwrap() == self.tail.unwrap() {
                    self.head = None;
                    self.tail = None;
                } else {
                    self.head = Some((self.head.unwrap() + 1)%self.capacity);
                }
                Some(popped_val)
            }
        }
    }

    pub fn get_read_index(&self) -> usize {
        self.head.unwrap()
    }

    pub fn set_read_index(&mut self, index: usize) {
        self.head = Some(index)
    }

    pub fn get_write_index(&self) -> usize {
        self.tail.unwrap()
    }

    pub fn set_write_index(&mut self, index: usize) {
        self.tail = Some(index)
    }

    pub fn len(&self) -> usize {
        // Return number of values currently in the buffer.
        match (self.head, self.tail) {
            (Some(h), Some(t)) => {
                if t >= h {
                    return t-h+1;
                } else {
                    return self.capacity - (h-t-1);
                }
            },
            (_, _) => 0
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl RingBuffer<f32> {
    // Return the value at at an offset from the current read index.
    // To handle fractional offsets, linearly interpolate between adjacent values. 
    pub fn get_frac(&self, offset: f32) -> Option<f32> {
        assert!(offset>=0.0, "Offset must be greater than or equal to 0");
        if offset >= self.len() as f32 - 1.0 { return None }
        match self.head {
            None => None,
            Some(t) => Some(
                (1.0-(offset - offset.trunc())) * self.ring_buffer[(t+offset.trunc() as usize)%self.capacity] 
                + (offset - offset.trunc()) * self.ring_buffer[(t+offset.trunc() as usize + 1)%self.capacity]
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod push_tests {
        use super::*;

        #[test]
        fn test_1() {
            let mut buffer = RingBuffer::<i16>::new(3);
            for i in 1..5 {
                buffer.push(i);
            }
            
            assert_eq!(vec![1,2,3], buffer.ring_buffer);
        }
    }

    mod get_frac_tests {
        use super::*;

        #[test]
        fn test_1() {
            let mut buffer = RingBuffer::<f32>::new(3);
            for i in 1..4 {
                buffer.push(i as f32);
            }
            assert_eq!(1.5, buffer.get_frac(0.5).unwrap());
        }

        #[test]
        fn test_2() {
            let mut buffer = RingBuffer::<f32>::new(3);
            for i in 1..4 {
                buffer.push(i as f32);
            }
            assert_eq!(2.8, buffer.get_frac(1.8).unwrap());
        }

        #[test]
        #[should_panic]
        fn test_3() {
            let mut buffer = RingBuffer::<f32>::new(3);
            for i in 1..4 {
                buffer.push(i as f32);
            }
            let _ = buffer.get_frac(-0.2);
        }

        #[test]
        fn test_4() {
            let mut buffer = RingBuffer::<f32>::new(3);
            for i in 1..4 {
                buffer.push(i as f32);
            }
            assert_eq!(None, buffer.get_frac(2.1));
        }
    }
    
}
