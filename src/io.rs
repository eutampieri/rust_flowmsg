use rumqttc::{Client, Connection, MqttOptions, QoS};
use std::sync::{mpsc, Arc};

pub type SharedBuffer = Arc<Vec<u8>>;

pub trait Node {
    fn link_to(&mut self, other_node: &dyn Node) {
        self.add_output(other_node.get_sender());
    }
    fn get_sender(&self) -> mpsc::Sender<SharedBuffer>;
    fn add_output(&mut self, sender: mpsc::Sender<SharedBuffer>);
    fn run(&mut self);
}

pub struct ConsoleOut {
    label: String,
    chan: (mpsc::Sender<SharedBuffer>, mpsc::Receiver<SharedBuffer>),
}
impl ConsoleOut {
    pub fn new(label: String) -> Self {
        Self {
            label,
            chan: mpsc::channel(),
        }
    }
}
impl Node for ConsoleOut {
    fn get_sender(&self) -> mpsc::Sender<SharedBuffer> {
        self.chan.0.clone()
    }

    fn add_output(&mut self, _: mpsc::Sender<SharedBuffer>) {
        panic!("Trying to add outputs to an input-only node");
    }

    fn run(&mut self) {
        loop {
            if let Ok(val) = self.chan.1.recv() {
                println!("{}: {:?}", self.label, val);
            }
        }
    }
}

pub struct MqttIn {
    connection: Connection,
    outputs: Vec<mpsc::Sender<SharedBuffer>>,
}

impl MqttIn {
    pub fn new(server: &str, port: u16, topic: &str) -> Self {
        let mut mqttoptions =
            MqttOptions::new(&format!("rust_flowmsg_{}", lolid::Uuid::v4()), server, port);
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
    fn get_sender(&self) -> mpsc::Sender<SharedBuffer> {
        panic!("Attempting to publish to a send-only node")
    }

    fn add_output(&mut self, sender: mpsc::Sender<SharedBuffer>) {
        self.outputs.push(sender)
    }

    fn run(&mut self) {
        for notification in self.connection.iter().flatten() {
            if let rumqttc::Event::Incoming(rumqttc::Packet::Publish(msg)) = notification {
                let payload = std::sync::Arc::new(msg.payload.to_vec());
                for output in &self.outputs {
                    if output.send(payload.clone()).is_err() {}
                }
            }
        }
    }
}

pub struct MqttOut {
    client: Client,
    chan: (mpsc::Sender<SharedBuffer>, mpsc::Receiver<SharedBuffer>),
    topic: String,
}

impl MqttOut {
    pub fn new(server: &str, port: u16, topic: String) -> Self {
        let mut mqttoptions =
            MqttOptions::new(&format!("rust_flowmsg_{}", lolid::Uuid::v4()), server, port);
        mqttoptions.set_keep_alive(5);

        let (client, mut connection) = Client::new(mqttoptions, 10);
        std::thread::spawn(move || for _ in connection.iter() {});
        Self {
            client,
            chan: mpsc::channel(),
            topic,
        }
    }
}

impl Node for MqttOut {
    fn get_sender(&self) -> mpsc::Sender<SharedBuffer> {
        self.chan.0.clone()
    }

    fn add_output(&mut self, _: mpsc::Sender<SharedBuffer>) {
        panic!("Attempting to subscribe to a recieve-only node")
    }

    fn run<'a>(&'a mut self) {
        loop {
            if let Ok(val) = self.chan.1.try_recv() {
                if self
                    .client
                    .publish(&self.topic, QoS::ExactlyOnce, false, val.to_vec())
                    .is_err()
                {}
            }
        }
    }
}
