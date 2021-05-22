pub trait Node<'r> {
    fn input(&mut self, data: &[u8]);
    fn link_to(&mut self, node: &'r mut dyn Node<'r>);
}

#[derive(Default)]
pub struct ConsoleOut {
    label: &'static str,
}
impl<'r> Node<'r> for ConsoleOut {
    fn input(&mut self, data: &[u8]) {
        println!("{}: {:?}", self.label, data);
    }

    fn link_to(&mut self, _: &'r mut dyn Node<'r>) {}
}
