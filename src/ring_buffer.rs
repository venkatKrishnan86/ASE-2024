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

    pub fn peek(&self) -> T {
        match self.tail {
            None => T::default(),
            Some(t) => self.ring_buffer[t]
        }
    }

    pub fn get(&self, offset: usize) -> T {
        match self.tail {
            None => T::default(),
            Some(t) => self.ring_buffer[(t+offset)%self.capacity]
        }
    }

    // `push` and `pop` write/read and advance the indices.
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

    pub fn pop(&mut self) -> T {
        match self.len() == 0 {
            true => T::default(),
            false => {
                let popped_val = self.ring_buffer[self.head.unwrap()];
                if self.head.unwrap() == self.tail.unwrap() {
                    self.head = None;
                    self.tail = None;
                } else {
                    self.head = Some((self.head.unwrap() + 1)%self.capacity);
                }
                popped_val
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
        // Return the length of the internal buffer.
        self.capacity
    }
}
