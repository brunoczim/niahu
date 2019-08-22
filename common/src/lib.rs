pub trait Machine: Default {
    fn step(&mut self);
    fn execute(&mut self);
}
