pub struct RingBuffer<T> {
    // TODO: fill this in.
}

impl<T: Copy + Default> RingBuffer<T> {
    pub fn new(length: usize) -> Self {
        // Create a new RingBuffer with `length` slots and "default" values.
        // Hint: look into `vec!` and the `Default` trait.
        todo!()
    }

    pub fn reset(&mut self) {
        // Clear internal buffer and reset indices.
        todo!()
    }

    // `put` and `peek` write/read without advancing the indices.
    pub fn put(&mut self, value: T) {
        todo!()
    }

    pub fn peek(&self) -> T {
        todo!()
    }

    pub fn get(&self, offset: usize) -> T {
        todo!()
    }

    // `push` and `pop` write/read and advance the indices.
    pub fn push(&mut self, value: T) {
        todo!()
    }

    pub fn pop(&mut self) -> T {
        todo!()
    }

    pub fn get_read_index(&self) -> usize {
        todo!()
    }

    pub fn set_read_index(&mut self, index: usize) {
        todo!()
    }

    pub fn get_write_index(&self) -> T {
        todo!()
    }

    pub fn set_write_index(&mut self, index: usize) {
        todo!()
    }

    pub fn len(&self) -> usize {
        // Return number of values currently in the buffer.
        todo!()
    }

    pub fn capacity(&self) -> usize {
        // Return the length of the internal buffer.
        todo!()
    }
}
