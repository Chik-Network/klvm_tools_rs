extern crate klvmr as klvm_rs;

use ::serde_json;
use std::env;
use std::rc::Rc;

use klvm_tools_rs::compiler::compiler::DefaultCompilerOpts;
use klvm_tools_rs::compiler::frontend::frontend;
use klvm_tools_rs::compiler::sexp::parse_sexp;
use klvm_tools_rs::compiler::srcloc::Srcloc;

use klvm_tools_rs::util::ErrInto;

fn main() {
    let opts = Rc::new(DefaultCompilerOpts::new("*program*"));
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Give a chiklisp program to convert to json AST");
        return;
    }

    let loc = Srcloc::start("*program*");
    let result = parse_sexp(loc, args[1].bytes())
        .err_into()
        .and_then(|parsed_program| frontend(opts.clone(), &parsed_program));
    match result {
        Ok(program) => match serde_json::to_string(&program) {
            Ok(output) => println!("{output}"),
            Err(e) => {
                println!("{e:?}");
            }
        },
        Err(e) => {
            println!("{e:?}");
        }
    }
}
