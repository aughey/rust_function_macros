#[derive(Debug)]
pub struct RunnableState<const SIZE: usize> {
    pub runnable: [bool; SIZE],
}
impl<const SIZE: usize> Default for RunnableState<SIZE> {
    fn default() -> Self {
        Self {
            runnable: [true; SIZE],
        }
    }
}
