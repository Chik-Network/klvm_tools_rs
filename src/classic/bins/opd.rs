use klvm_tools_rs::classic::klvm_tools::cmds::opd;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    opd(&args);
}
