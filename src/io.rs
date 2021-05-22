pub trait Node {
    fn input(&mut self, data: &[u8]);
    fn link_to(&mut self, node: Box<dyn Node>);
}
