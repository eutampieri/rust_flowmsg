use std::collections::HashMap;

use compute::{ReportByException, StaticOutput};
use io::{ConsoleOut, MqttIn, MqttOut, Node};

mod compute;
mod io;

fn main() {
    let mut nodes: std::collections::HashMap<String, Box<dyn Node + Send>> = HashMap::new();
    let raw_input = std::fs::read_to_string(std::env::args().nth(1).expect("Provide conf file"))
        .expect("Failed to read conf file")
        .split("\n")
        .map(|x| x.split("\t"));
    for mut input in raw_input {
        let node_name = input.next().expect("No node name provided");
        let node_id = input.next().expect("No node id provided");
        match node_name {
            "mqtt_in" => {
                let node = MqttIn::new(input.next().unwrap(), input.next().unwrap().parse().unwrap(), input.next().unwrap());
                nodes.insert(node_id.to_string(), Box::new(node));
            }
            "mqtt_out" => {
                let node = MqttOut::new(input.next().unwrap(), input.next().unwrap().parse().unwrap(), input.next().unwrap().to_string());
                nodes.insert(node_id.to_string(), Box::new(node));
            }
            "rbe" => {
                nodes.insert(node_id.to_string(), Box::new(ReportByException::new(node_id)));
            }
            "static_out" => {
                nodes.insert(node_id.to_string(), Box::new(StaticOutput::new(input.next().unwrap().to_string())));
            }
            "dbg" => {
                nodes.insert(node_id.to_string(), Box::new(ConsoleOut::new(input.next().unwrap().to_string())));
            }
            "link" => {
                let right = nodes
                    .get(input.next().unwrap())
                    .expect("Right node not found")
                    .get_sender();
                let left = nodes
                    .get_mut(node_id)
                    .expect("Left node not found");
                left.add_output(right);
            }
            _ => (),
        }
    }
    let handles = nodes
        .into_iter()
        .map(|(_, mut node)| std::thread::spawn(move || node.run()))
        .collect::<Vec<_>>();
    for handle in handles {
        handle.join().unwrap();
    }
}
