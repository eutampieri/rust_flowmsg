use std::sync::mpsc;

pub trait Node<'a> {
    fn link_to(&mut self, other_node: &dyn Node<'a>) {
        self.add_output(other_node.get_sender());
    }
    fn get_sender(&self) -> mpsc::Sender<&'a [u8]>;
    fn add_output(&mut self, sender: mpsc::Sender<&'a [u8]>);
    fn run(&mut self);
}

pub struct ConsoleOut<'a> {
    label: &'static str,
    chan: (mpsc::Sender<&'a [u8]>, mpsc::Receiver<&'a [u8]>),
}
impl<'a> ConsoleOut<'a> {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            chan: mpsc::channel(),
        }
    }
}
impl<'a> Node<'a> for ConsoleOut<'a> {
    fn get_sender(&self) -> mpsc::Sender<&'a [u8]> {
        self.chan.0.clone()
    }

    fn add_output(&mut self, sender: mpsc::Sender<&[u8]>) {
        panic!("Trying to add outputs to an input-only node");
    }

    fn run(&mut self) {
        loop {
            match self.chan.1.recv() {
                Ok(val) => {
                    println!("{}: {:?}", self.label, val);
                }
                _ => (),
            }
        }
    }
}
