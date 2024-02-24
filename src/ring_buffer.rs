pub struct RingBuffer<T> {
    buffer: Vec<T>,
    head: usize,
    tail: usize,
}

impl<T: Copy + Default> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        RingBuffer {
            buffer: vec![T::default(); capacity],
            head: 0,
            tail: 0,
        }
    }

    pub fn reset(&mut self) {
        self.buffer.fill(T::default());
        self.head = 0;
        self.tail = 0;
    }

    // `put` and `peek` write/read without advancing the indices.
    pub fn put(&mut self, value: T) {
        self.buffer[self.head] = value
    }

    pub fn peek(&self) -> T {
        self.buffer[self.tail]
    }

    pub fn get(&self, offset: usize) -> T {
        self.buffer[(self.tail + offset) % self.capacity()]
    }

    // `push` and `pop` write/read and advance the indices.
    pub fn push(&mut self, value: T) {
        self.buffer[self.head] = value;
        self.head = (self.head + 1) % self.capacity();
    }

    pub fn pop(&mut self) -> T {
        let value = self.buffer[self.tail];
        self.tail = (self.tail + 1) % self.capacity();
        value
    }

    pub fn get_read_index(&self) -> usize {
        self.tail
    }

    pub fn set_read_index(&mut self, index: usize) {
        self.tail = index % self.capacity()
    }

    pub fn get_write_index(&self) -> usize {
        self.head
    }

    pub fn set_write_index(&mut self, index: usize) {
        self.head = index % self.capacity()
    }

    pub fn len(&self) -> usize {
        // Return number of values currently in the ring buffer.
        if self.head >= self.tail {
            self.head - self.tail
        } else {
            self.head + self.capacity() - self.tail
        }
    }

    pub fn capacity(&self) -> usize {
        // Return the size of the internal buffer.
        self.buffer.len()
    }
}

impl RingBuffer<f32> {
    // Return the value at at an offset from the current read index.
    // To handle fractional offsets, linearly interpolate between adjacent values. 
    pub fn get_frac(&self, offset: f32) -> f32 {
        assert!(offset>=0.0, "Offset must be greater than or equal to 0");
        if offset >= self.len() as f32 - 1.0 { return f32::default() }
        match self.len() == 0 {
            true => f32::default(),
            false => (1.0-(offset - offset.trunc())) * self.buffer[(self.tail+offset.trunc() as usize)%self.capacity()] 
                + (offset - offset.trunc()) * self.buffer[(self.tail+offset.trunc() as usize + 1)%self.capacity()]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrapping() {
        // Test that ring buffer is a ring (wraps after more than `length` elements have entered).
        let capacity = 17;
        let delay = 5;
        let mut ring_buffer: RingBuffer<f32> = RingBuffer::new(capacity);

        for i in 0..delay {
            ring_buffer.push(i as f32);
        }

        for i in delay..capacity + 13 {
            assert_eq!(ring_buffer.len(), delay);
            assert_eq!(ring_buffer.pop(), (i - delay) as f32);
            ring_buffer.push(i as f32)
        }
    }

    #[test]
    fn test_api() {
        // Basic test of all API functions.
        let capacity = 3;
        let mut ring_buffer = RingBuffer::new(capacity);
        assert_eq!(ring_buffer.capacity(), capacity);

        ring_buffer.put(3);
        assert_eq!(ring_buffer.peek(), 3);

        ring_buffer.set_write_index(1);
        assert_eq!(ring_buffer.get_write_index(), 1);

        ring_buffer.push(17);
        assert_eq!(ring_buffer.get_write_index(), 2);

        assert_eq!(ring_buffer.get_read_index(), 0);
        assert_eq!(ring_buffer.get(1), 17);
        assert_eq!(ring_buffer.pop(), 3);
        assert_eq!(ring_buffer.get_read_index(), 1);

        assert_eq!(ring_buffer.len(), 1);
        ring_buffer.push(42);
        assert_eq!(ring_buffer.len(), 2);

        assert_eq!(ring_buffer.get_write_index(), 0);

        // Should be unchanged.
        assert_eq!(ring_buffer.capacity(), capacity);
    }

    #[test]
    fn test_capacity() {
        // Tricky: does `capacity` mean "size of internal buffer" or "number of elements before this is full"?
        let capacity = 3;
        let mut ring_buffer = RingBuffer::new(3);
        for i in 0..(capacity - 1) {
            ring_buffer.push(i);
            dbg!(ring_buffer.len());
            assert_eq!(ring_buffer.len(), i+1);
        }
    }

    #[test]
    fn test_reset() {
        // Test state after initialization and reset.
        let mut ring_buffer = RingBuffer::new(512);

        // Check initial state.
        assert_eq!(ring_buffer.get_read_index(), 0);
        assert_eq!(ring_buffer.get_write_index(), 0);
        for i in 0..ring_buffer.capacity() {
            assert_eq!(ring_buffer.get(i), 0.0);
        }

        // Fill ring buffer, mess with indices.
        let fill = 123.456;
        for i in 0..ring_buffer.capacity() {
            ring_buffer.push(fill);
            assert_eq!(ring_buffer.get(i), fill);
        }

        ring_buffer.set_write_index(17);
        ring_buffer.set_read_index(42);

        // Check state after reset.
        ring_buffer.reset();
        assert_eq!(ring_buffer.get_read_index(), 0);
        assert_eq!(ring_buffer.get_write_index(), 0);
        for i in 0..ring_buffer.capacity() {
            assert_eq!(ring_buffer.get(i), 0.0);
        }
    }

    #[test]
    fn test_weird_inputs() {
        let capacity = 5;
        let mut ring_buffer = RingBuffer::<f32>::new(capacity);

        ring_buffer.set_write_index(capacity);
        assert_eq!(ring_buffer.get_write_index(), 0);
        ring_buffer.set_write_index(capacity * 2 + 3);
        assert_eq!(ring_buffer.get_write_index(), 3);

        ring_buffer.set_read_index(capacity);
        assert_eq!(ring_buffer.get_read_index(), 0);
        ring_buffer.set_read_index(capacity * 2 + 3);
        assert_eq!(ring_buffer.get_read_index(), 3);

        // NOTE: Negative indices are also weird, but we can't even pass them due to type checking!
    }

    mod get_frac_tests {
        use super::*;

        #[test]
        fn test_1() {
            let mut buffer = RingBuffer::<f32>::new(4);
            for i in 1..4 {
                buffer.push(i as f32);
            }
            assert_eq!(1.5, buffer.get_frac(0.5));
        }

        #[test]
        fn test_2() {
            let mut buffer = RingBuffer::<f32>::new(4);
            for i in 1..4 {
                buffer.push(i as f32);
            }
            assert_eq!(2.8, buffer.get_frac(1.8));
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
            assert_eq!(0.0, buffer.get_frac(2.1));
        }
    }
}

// impl RingBuffer<f32> {
//     // Return the value at at an offset from the current read index.
//     // To handle fractional offsets, linearly interpolate between adjacent values. 
//     pub fn get_frac(&self, offset: f32) -> Option<f32> {
//         assert!(offset>=0.0, "Offset must be greater than or equal to 0");
//         if offset >= self.len() as f32 - 1.0 { return None }
//         match self.head {
//             None => None,
//             Some(t) => Some(
//                 (1.0-(offset - offset.trunc())) * self.ring_buffer[(t+offset.trunc() as usize)%self.capacity] 
//                 + (offset - offset.trunc()) * self.ring_buffer[(t+offset.trunc() as usize + 1)%self.capacity]
//             )
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     mod push_tests {
//         use super::*;

//         #[test]
//         fn test_1() {
//             let mut buffer = RingBuffer::<i16>::new(3);
//             for i in 1..5 {
//                 buffer.push(i);
//             }
            
//             assert_eq!(vec![1,2,3], buffer.ring_buffer);
//         }
//     }

//     mod get_frac_tests {
//         use super::*;

//         #[test]
//         fn test_1() {
//             let mut buffer = RingBuffer::<f32>::new(3);
//             for i in 1..4 {
//                 buffer.push(i as f32);
//             }
//             assert_eq!(1.5, buffer.get_frac(0.5).unwrap());
//         }

//         #[test]
//         fn test_2() {
//             let mut buffer = RingBuffer::<f32>::new(3);
//             for i in 1..4 {
//                 buffer.push(i as f32);
//             }
//             assert_eq!(2.8, buffer.get_frac(1.8).unwrap());
//         }

//         #[test]
//         #[should_panic]
//         fn test_3() {
//             let mut buffer = RingBuffer::<f32>::new(3);
//             for i in 1..4 {
//                 buffer.push(i as f32);
//             }
//             let _ = buffer.get_frac(-0.2);
//         }

//         #[test]
//         fn test_4() {
//             let mut buffer = RingBuffer::<f32>::new(3);
//             for i in 1..4 {
//                 buffer.push(i as f32);
//             }
//             assert_eq!(None, buffer.get_frac(2.1));
//         }
//     }
    
// }
