use io::Node;

mod compute;
mod io;

fn main() {
    let mut rbe = compute::ReportByException::new();
    let mut console_out = io::ConsoleOut::new("out");
    let mut val_changer = compute::StaticOutput::new("p");
    let mut mqtt_in = io::MqttIn::new("192.168.1.9", 1883, "prova");
    val_changer.link_to(&mut console_out);
    rbe.link_to(&mut val_changer);
    mqtt_in.link_to(&mut rbe);
    let sender = rbe.get_sender();
    std::thread::spawn(move || rbe.run());
    std::thread::spawn(move || console_out.run());
    std::thread::spawn(move || val_changer.run());
    std::thread::spawn(move || mqtt_in.run());
    loop {}
}
