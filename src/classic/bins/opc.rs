use klvm_tools_rs::classic::klvm_tools::cmds::opc;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    opc(&args);
}
