use crate::io::Node;

#[derive(Default)]
pub struct ReportByException {
    output: Vec<Box<dyn Node>>,
    last_value: Vec<u8>,
}

impl Node for ReportByException {
    fn input(&mut self, data: &[u8]) {
        if self.last_value != data {
            self.last_value = data.to_vec();
            for o in self.output.iter_mut() {
                o.input(data);
            }
        }
    }

    fn link_to(&mut self, node: Box<dyn Node>) {
        self.output.push(Box::from(node));
    }
}
