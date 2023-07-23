extern crate klvmr as klvm_rs;

use std::collections::HashMap;
use std::env;
use std::rc::Rc;

use klvm_rs::allocator::Allocator;

use klvm_tools_rs::compiler::compiler::DefaultCompilerOpts;
use klvm_tools_rs::compiler::comptypes::CompileErr;
use klvm_tools_rs::compiler::evaluate::Evaluator;
use klvm_tools_rs::compiler::frontend::frontend;
use klvm_tools_rs::compiler::sexp::parse_sexp;
use klvm_tools_rs::compiler::srcloc::Srcloc;

use klvm_tools_rs::classic::klvm_tools::stages::stage_0::DefaultProgramRunner;

fn main() {
    let mut allocator = Allocator::new();
    let runner = Rc::new(DefaultProgramRunner::new());
    let opts = Rc::new(DefaultCompilerOpts::new("*program*"));
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("give a chiklisp program to minify");
        return;
    }

    let loc = Srcloc::start("*program*");
    let _ = parse_sexp(loc, args[1].bytes())
        .map_err(|e| CompileErr(e.0.clone(), e.1))
        .and_then(|parsed_program| frontend(opts.clone(), &parsed_program))
        .and_then(|program| {
            let e = Evaluator::new(opts.clone(), runner.clone(), program.helpers.clone());
            e.shrink_bodyform(
                &mut allocator,
                program.args.clone(),
                &HashMap::new(),
                program.exp,
                false,
            )
        })
        .map(|result| {
            println!("shrunk: {}", result.to_sexp());
        })
        .map_err(|e| {
            println!("failed: {:?}", e);
        });
}
