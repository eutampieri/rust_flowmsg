use io::Node;

mod compute;
mod io;

fn main() {
    let mut rbe = compute::ReportByException::new();
    let mut console_out = io::ConsoleOut::new("out");
    let mut val_changer = compute::StaticOutput::new("p");
    val_changer.link_to(&mut console_out);
    rbe.link_to(&mut val_changer);
    for input in &[[0x1u8], [0x1], [0x1], [0x0], [0x1], [0x0], [0x0], [0x1]] {
        rbe.get_sender().send(input);
    }
}
