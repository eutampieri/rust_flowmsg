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
        .map(|x| x.split("\t").map(|x| x.to_owned()).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    for input in raw_input {
        match input[0].as_str() {
            "mqtt_in" => {
                let id = input[1].clone();
                let node = MqttIn::new(&input[2], input[3].parse().unwrap(), &input[4]);
                nodes.insert(id, Box::new(node));
            }
            "mqtt_out" => {
                let id = input[1].clone();
                let node = MqttOut::new(&input[2], input[3].parse().unwrap(), input[4].clone());
                nodes.insert(id, Box::new(node));
            }
            "rbe" => {
                let id = input[1].clone();
                nodes.insert(id, Box::new(ReportByException::new()));
            }
            "static_out" => {
                let id = input[1].clone();
                nodes.insert(id, Box::new(StaticOutput::new(input[2].clone())));
            }
            "dbg" => {
                let id = input[1].clone();
                nodes.insert(id, Box::new(ConsoleOut::new(input[2].clone())));
            }
            "link" => {
                let right = nodes
                    .get(&input[2].clone())
                    .expect("Right node not found")
                    .get_sender();
                let left = nodes
                    .get_mut(&input[1].clone())
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
