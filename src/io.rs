use std::{
    sync::{mpsc, Arc},
    time::Duration,
};

use paho_mqtt::{Client, ConnectOptionsBuilder, CreateOptions, CreateOptionsBuilder};

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
    client: Client,
    outputs: Vec<mpsc::Sender<SharedBuffer>>,
}

impl MqttIn {
    pub fn new(server: &str, port: u16, topic: &str) -> Self {
        let mqttoptions = CreateOptionsBuilder::new()
            .server_uri(&format!("tcp://{}:{}", server, port))
            .client_id(&format!("rust_flowmsg_{}", lolid::Uuid::v4()))
            .finalize();
        //mqttoptions. &format!("rust_flowmsg_{}", lolid::Uuid::v4()), server, port);
        //mqttoptions.set_keep_alive(5);
        let connection_options = ConnectOptionsBuilder::new()
            .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(10))
            .keep_alive_interval(Duration::from_secs(20))
            .clean_session(true)
            .finalize();

        let client = Client::new(mqttoptions).expect("Failed to create MQTT client");
        client.connect(connection_options);
        client.subscribe(topic, 2).expect("Could not subscribe");
        Self {
            client,
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
        let queue = self.client.start_consuming();
        loop {
            if let Ok(Some(msg)) = queue.recv() {
                let payload = std::sync::Arc::new(msg.payload().to_vec());
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
        let mqttoptions = CreateOptionsBuilder::new()
            .server_uri(&format!("tcp://{}:{}", server, port))
            .client_id(&format!("rust_flowmsg_{}", lolid::Uuid::v4()))
            .finalize();
        //mqttoptions. &format!("rust_flowmsg_{}", lolid::Uuid::v4()), server, port);
        //mqttoptions.set_keep_alive(5);
        let connection_options = ConnectOptionsBuilder::new()
            .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(10))
            .keep_alive_interval(Duration::from_secs(20))
            .clean_session(true)
            .finalize();

        let client = Client::new(mqttoptions).expect("Failed to create MQTT client");
        client
            .connect(connection_options)
            .expect("Failed to connect to topic");
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
            if let Ok(val) = self.chan.1.recv() {
                let msg = paho_mqtt::Message::new(&self.topic, val.to_vec(), 2);
                let tok = self.client.publish(msg);
                if tok.is_err() {}
            }
        }
    }
}
