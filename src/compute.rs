use std::sync::mpsc;

use crate::io::Node;

pub struct ReportByException<'a> {
    last_value: Vec<u8>,
    chan: (mpsc::Sender<&'a [u8]>, mpsc::Receiver<&'a [u8]>),
    outs: Vec<mpsc::Sender<&'a [u8]>>,
}

impl<'a> Node<'a> for ReportByException<'a> {
    fn get_sender(&self) -> mpsc::Sender<&'a [u8]> {
        self.chan.0.clone()
    }

    fn add_output(&mut self, sender: mpsc::Sender<&'a [u8]>) {
        self.outs.push(sender)
    }

    fn run(&mut self) {
        loop {
            match self.chan.1.recv() {
                Ok(data) => {
                    if self.last_value != data {
                        self.last_value = data.to_vec();
                        for o in &self.outs {
                            if o.send(data).is_err() {}
                        }
                    }
                }
                _ => (),
            }
        }
    }
}

impl<'a> ReportByException<'a> {
    pub fn new() -> Self {
        Self {
            chan: mpsc::channel(),
            last_value: vec![],
            outs: vec![],
        }
    }
}

pub struct StaticOutput<'a> {
    output: &'static str,
    chan: (mpsc::Sender<&'a [u8]>, mpsc::Receiver<&'a [u8]>),
    output_nodes: Vec<mpsc::Sender<&'a [u8]>>,
}

impl<'a> Node<'a> for StaticOutput<'a> {
    fn get_sender(&self) -> mpsc::Sender<&'a [u8]> {
        self.chan.0.clone()
    }

    fn add_output(&mut self, sender: mpsc::Sender<&'a [u8]>) {
        self.output_nodes.push(sender)
    }

    fn run(&mut self) {
        loop {
            match self.chan.1.recv() {
                Ok(_) => {
                    for o in &self.output_nodes {
                        if o.send(self.output.as_bytes()).is_err() {}
                    }
                }
                _ => (),
            }
        }
    }
}

impl<'a> StaticOutput<'a> {
    pub fn new(output: &'static str) -> Self {
        Self {
            chan: mpsc::channel(),
            output_nodes: vec![],
            output,
        }
    }
}
