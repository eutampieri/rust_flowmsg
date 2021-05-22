use crate::io::Node;

#[derive(Default)]
pub struct ReportByException<'r> {
    output: Vec<&'r mut dyn Node<'r>>,
    last_value: Vec<u8>,
}

impl<'r> Node<'r> for ReportByException<'r> {
    fn input(&mut self, data: &[u8]) {
        if self.last_value != data {
            self.last_value = data.to_vec();
            for o in &mut self.output {
                o.input(data);
            }
        }
    }

    fn link_to(&mut self, node: &'r mut dyn Node<'r>) {
        self.output.push(node);
    }
}
