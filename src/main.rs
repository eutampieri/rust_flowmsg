use io::Node;

mod compute;
mod io;

fn main() {
    let mut rbe = compute::ReportByException::default();
    let mut console_out = io::ConsoleOut::default();
    rbe.link_to(&mut console_out);
    for input in &[[0x1u8], [0x1], [0x1], [0x0], [0x1], [0x0], [0x0], [0x1]] {
        rbe.input(input);
    }
}
