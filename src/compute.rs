use std::{
    cmp::Ordering,
    sync::{mpsc, Arc},
};

use crate::io::Node;

pub struct ReportByException {
    last_value: Vec<u8>,
    chan: (mpsc::Sender<Arc<Vec<u8>>>, mpsc::Receiver<Arc<Vec<u8>>>),
    outs: Vec<mpsc::Sender<Arc<Vec<u8>>>>,
}

impl Node for ReportByException {
    fn get_sender(&self) -> mpsc::Sender<Arc<Vec<u8>>> {
        self.chan.0.clone()
    }

    fn add_output(&mut self, sender: mpsc::Sender<Arc<Vec<u8>>>) {
        self.outs.push(sender)
    }

    fn run(&mut self) {
        loop {
            match self.chan.1.recv() {
                Ok(data) => {
                    if data.iter().cmp(&self.last_value) != Ordering::Equal {
                        self.last_value = data.to_vec();
                        for o in &self.outs {
                            if o.send(data.clone()).is_err() {}
                        }
                    }
                }
                _ => (),
            }
        }
    }
}

impl ReportByException {
    pub fn new() -> Self {
        Self {
            chan: mpsc::channel(),
            last_value: vec![],
            outs: vec![],
        }
    }
}

pub struct StaticOutput {
    output: &'static str,
    chan: (mpsc::Sender<Arc<Vec<u8>>>, mpsc::Receiver<Arc<Vec<u8>>>),
    output_nodes: Vec<mpsc::Sender<Arc<Vec<u8>>>>,
}

impl Node for StaticOutput {
    fn get_sender(&self) -> mpsc::Sender<Arc<Vec<u8>>> {
        self.chan.0.clone()
    }

    fn add_output(&mut self, sender: mpsc::Sender<Arc<Vec<u8>>>) {
        self.output_nodes.push(sender)
    }

    fn run(&mut self) {
        loop {
            match self.chan.1.recv() {
                Ok(_) => {
                    for o in &self.output_nodes {
                        if o.send(Arc::new(self.output.as_bytes().to_vec())).is_err() {}
                    }
                }
                _ => (),
            }
        }
    }
}

impl StaticOutput {
    pub fn new(output: &'static str) -> Self {
        Self {
            chan: mpsc::channel(),
            output_nodes: vec![],
            output,
        }
    }
}
