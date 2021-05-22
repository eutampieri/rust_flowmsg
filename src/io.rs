use rumqttc::{Client, Connection, MqttOptions, QoS};
use std::sync::{mpsc, Arc};

pub trait Node {
    fn link_to(&mut self, other_node: &dyn Node) {
        self.add_output(other_node.get_sender());
    }
    fn get_sender(&self) -> mpsc::Sender<Arc<Vec<u8>>>;
    fn add_output(&mut self, sender: mpsc::Sender<Arc<Vec<u8>>>);
    fn run(&mut self);
}

pub struct ConsoleOut {
    label: &'static str,
    chan: (mpsc::Sender<Arc<Vec<u8>>>, mpsc::Receiver<Arc<Vec<u8>>>),
}
impl ConsoleOut {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            chan: mpsc::channel(),
        }
    }
}
impl Node for ConsoleOut {
    fn get_sender(&self) -> mpsc::Sender<Arc<Vec<u8>>> {
        self.chan.0.clone()
    }

    fn add_output(&mut self, _: mpsc::Sender<Arc<Vec<u8>>>) {
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

pub struct MqttIn {
    connection: Connection,
    outputs: Vec<mpsc::Sender<Arc<Vec<u8>>>>,
}

impl MqttIn {
    pub fn new(server: &str, port: u16, topic: &str) -> Self {
        let mut mqttoptions = MqttOptions::new(&format!("rust_flowmsg_"), server, port);
        mqttoptions.set_keep_alive(5);

        let (mut client, connection) = Client::new(mqttoptions, 10);
        client.subscribe(topic, QoS::ExactlyOnce).unwrap();
        Self {
            connection,
            outputs: vec![],
        }
    }
}

impl Node for MqttIn {
    fn get_sender(&self) -> mpsc::Sender<Arc<Vec<u8>>> {
        panic!("Attempting to publish to a send-only node")
    }

    fn add_output(&mut self, sender: mpsc::Sender<Arc<Vec<u8>>>) {
        self.outputs.push(sender)
    }

    fn run(&mut self) {
        for notification in self.connection.iter() {
            match notification {
                Ok(event) => match event {
                    rumqttc::Event::Incoming(pkt) => match pkt {
                        rumqttc::Packet::Publish(msg) => {
                            let payload = std::sync::Arc::new(msg.payload.to_vec());
                            for output in &self.outputs {
                                output.send(payload.clone());
                            }
                        }
                        _ => (),
                    },
                    _ => (),
                },
                _ => (),
            }
        }
    }
}

pub struct MqttOut {
    client: Client,
    outputs: Vec<mpsc::Sender<Arc<Vec<u8>>>>,
    chan: (mpsc::Sender<Arc<Vec<u8>>>, mpsc::Receiver<Arc<Vec<u8>>>),
    topic: &'static str,
}

impl MqttOut {
    pub fn new(server: &str, port: u16, topic: &'static str) -> Self {
        let mut mqttoptions = MqttOptions::new(&format!("rust_flowmsg_"), server, port);
        mqttoptions.set_keep_alive(5);

        let (mut client, connection) = Client::new(mqttoptions, 10);
        Self {
            client,
            outputs: vec![],
            chan: mpsc::channel(),
            topic,
        }
    }
}

impl Node for MqttOut {
    fn get_sender(&self) -> mpsc::Sender<Arc<Vec<u8>>> {
        self.chan.0.clone()
    }

    fn add_output(&mut self, _: mpsc::Sender<Arc<Vec<u8>>>) {
        panic!("Attempting to subscribe to a recieve-only node")
    }

    fn run(&mut self) {
        loop {
            match self.chan.1.recv() {
                Ok(val) => {
                    if self
                        .client
                        .publish(self.topic, QoS::ExactlyOnce, false, val.to_vec())
                        .is_err()
                    {}
                }
                _ => (),
            }
        }
    }
}
