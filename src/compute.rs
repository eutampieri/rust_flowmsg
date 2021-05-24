use std::{
    cmp::Ordering,
    sync::{mpsc, Arc},
};

use crate::io::{Node, SharedBuffer};

pub struct ReportByException {
    last_value: Vec<u8>,
    chan: (mpsc::Sender<SharedBuffer>, mpsc::Receiver<SharedBuffer>),
    outs: Vec<mpsc::Sender<SharedBuffer>>,
}

impl Node for ReportByException {
    fn get_sender(&self) -> mpsc::Sender<SharedBuffer> {
        self.chan.0.clone()
    }

    fn add_output(&mut self, sender: mpsc::Sender<SharedBuffer>) {
        self.outs.push(sender)
    }

    fn run(&mut self) {
        loop {
            if let Ok(data) = self.chan.1.recv() {
                if data.iter().cmp(&self.last_value) != Ordering::Equal {
                    self.last_value = data.to_vec();
                    for o in &self.outs {
                        if o.send(data.clone()).is_err() {}
                    }
                }
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
    output: String,
    chan: (mpsc::Sender<SharedBuffer>, mpsc::Receiver<SharedBuffer>),
    output_nodes: Vec<mpsc::Sender<SharedBuffer>>,
}

impl Node for StaticOutput {
    fn get_sender(&self) -> mpsc::Sender<SharedBuffer> {
        self.chan.0.clone()
    }

    fn add_output(&mut self, sender: mpsc::Sender<SharedBuffer>) {
        self.output_nodes.push(sender)
    }

    fn run(&mut self) {
        loop {
            if self.chan.1.recv().is_ok() {
                for o in &self.output_nodes {
                    if o.send(Arc::new(self.output.as_bytes().to_vec())).is_err() {}
                }
            }
        }
    }
}

impl StaticOutput {
    pub fn new(output: String) -> Self {
        Self {
            chan: mpsc::channel(),
            output_nodes: vec![],
            output,
        }
    }
}
