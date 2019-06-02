mod any_state;
mod read;
mod write;

#[derive(Debug)]
pub struct Output<S> {
    state: S,
    buf: String,
}

#[derive(Debug)]
pub struct Read {
    /// Byte position that starts the input buffer.
    input_start: usize,
}

#[derive(Debug)]
pub struct Write;
