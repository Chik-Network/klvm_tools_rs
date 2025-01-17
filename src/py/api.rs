#![allow(clippy::all)]
// #[allow(clippy::borrow_deref_ref)]
// Eventually this can be downgraded and applied just to compile_klvm
// re: https://github.com/rust-lang/rust-clippy/issues/8971
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyString, PyTuple};

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use klvm_rs::allocator::Allocator;
use klvm_rs::serde::node_to_bytes;

use crate::classic::klvm::__type_compatibility__::{
    Bytes, BytesFromType, Stream, UnvalidatedBytesFromType,
};
use crate::classic::klvm::serialize::sexp_to_stream;
use crate::classic::klvm_tools::cmds;
use crate::classic::klvm_tools::klvmc;
use crate::classic::klvm_tools::stages::stage_0::DefaultProgramRunner;
use crate::compiler::cldb::{
    hex_to_modern_sexp, CldbOverrideBespokeCode, CldbRun, CldbRunEnv, CldbSingleBespokeOverride,
};
use crate::compiler::compiler::{
    extract_program_and_env, path_to_function, rewrite_in_program, DefaultCompilerOpts,
};
use crate::compiler::comptypes::{CompileErr, CompilerOpts};
use crate::compiler::klvm::{convert_to_klvm_rs, start_step};
use crate::compiler::preprocessor::gather_dependencies;
use crate::compiler::prims;
use crate::compiler::runtypes::RunFailure;
use crate::compiler::sexp::{decode_string, SExp};
use crate::compiler::srcloc::Srcloc;

use crate::util::{gentle_overwrite, version};

use crate::py::pyval::{klvm_value_to_python, python_value_to_klvm};

use super::binutils::create_binutils_module;
use super::cmds::create_cmds_module;

create_exception!(mymodule, CldbError, PyException);
create_exception!(mymodule, CompError, PyException);
create_exception!(mymodule, ToolError, PyException);

#[pyfunction]
fn get_version() -> PyResult<String> {
    Ok(version())
}

enum CompileKlvmSource<'a> {
    SourcePath(&'a PyAny),
    SourceCode(String, String),
}

enum CompileKlvmAction {
    CheckDependencies,
    CompileCode(Option<String>),
}

fn get_source_from_input(input_code: CompileKlvmSource) -> PyResult<(String, String)> {
    match input_code {
        CompileKlvmSource::SourcePath(input_path) => {
            let has_atom = input_path.hasattr("atom")?;
            let has_pair = input_path.hasattr("pair")?;

            let real_input_path = if has_atom {
                input_path.getattr("atom").and_then(|x| x.str())
            } else if has_pair {
                input_path
                    .getattr("pair")
                    .and_then(|x| x.get_item(0))
                    .and_then(|x| x.str())
            } else {
                input_path.extract()
            }?;

            let mut path_string = real_input_path.to_string();

            if !std::path::Path::new(&path_string).exists() && !path_string.ends_with(".klvm") {
                path_string += ".klvm";
            }

            let file_data = fs::read_to_string(&path_string)
                .map_err(|e| PyException::new_err(format!("error reading {path_string}: {e:?}")))?;
            Ok((path_string, file_data))
        }
        CompileKlvmSource::SourceCode(name, code) => Ok((name.clone(), code.clone())),
    }
}

fn run_klvm_compilation(
    input_code: CompileKlvmSource,
    action: CompileKlvmAction,
    search_paths: Vec<String>,
    export_symbols: Option<bool>,
) -> PyResult<PyObject> {
    // Resolve the input, get the indicated path and content.
    let (path_string, file_content) = get_source_from_input(input_code)?;

    // Load up our compiler opts.
    let def_opts: Rc<dyn CompilerOpts> = Rc::new(DefaultCompilerOpts::new(&path_string));
    let opts = def_opts.set_search_paths(&search_paths);

    match action {
        CompileKlvmAction::CompileCode(output) => {
            let mut allocator = Allocator::new();
            let mut symbols = HashMap::new();

            // Output is a program represented as klvm data in allocator.
            let klvm_result = klvmc::compile_klvm_text(
                &mut allocator,
                opts.clone(),
                &mut symbols,
                &file_content,
                &path_string,
                true,
            )
            .map_err(|e| CompError::new_err(e.format(&allocator, opts)))?;

            // Get the text representation, which will go either to the output file
            // or result.
            let mut hex_text = Bytes::new(Some(BytesFromType::Raw(node_to_bytes(
                &allocator,
                klvm_result,
            )?)))
            .hex();
            let compiled = if let Some(output_file) = output {
                // Write output with eol.
                hex_text += "\n";
                gentle_overwrite(&path_string, &output_file, &hex_text)
                    .map_err(PyException::new_err)?;
                output_file.to_string()
            } else {
                hex_text
            };

            // Produce compiled output according to whether output with symbols
            // or just the standard result is required.
            Python::with_gil(|py| {
                if export_symbols == Some(true) {
                    let mut result_dict = HashMap::new();
                    result_dict.insert("output".to_string(), compiled.into_py(py));
                    result_dict.insert("symbols".to_string(), symbols.into_py(py));
                    Ok(result_dict.into_py(py))
                } else {
                    Ok(compiled.into_py(py))
                }
            })
        }
        CompileKlvmAction::CheckDependencies => {
            // Produce dependency results.
            let result_deps: Vec<String> =
                gather_dependencies(opts, &path_string.to_string(), &file_content)
                    .map_err(|e| CompError::new_err(format!("{}: {}", e.0, e.1)))
                    .map(|rlist| rlist.iter().map(|i| decode_string(&i.name)).collect())?;

            // Return all visited files.
            Python::with_gil(|py| Ok(result_deps.into_py(py)))
        }
    }
}

#[pyfunction]
#[pyo3(signature = (input_path, output_path, search_paths = Vec::new(), export_symbols = None))]
fn compile_klvm(
    input_path: &PyAny,
    output_path: String,
    search_paths: Vec<String>,
    export_symbols: Option<bool>,
) -> PyResult<PyObject> {
    run_klvm_compilation(
        CompileKlvmSource::SourcePath(input_path),
        CompileKlvmAction::CompileCode(Some(output_path)),
        search_paths,
        export_symbols,
    )
}

#[pyfunction]
#[pyo3(signature = (source, search_paths = Vec::new(), export_symbols = None))]
fn compile(
    source: String,
    search_paths: Vec<String>,
    export_symbols: Option<bool>,
) -> PyResult<PyObject> {
    run_klvm_compilation(
        CompileKlvmSource::SourceCode("*inline*".to_string(), source),
        CompileKlvmAction::CompileCode(None),
        search_paths,
        export_symbols,
    )
}

#[pyfunction]
#[pyo3(signature = (input_path, search_paths=Vec::new()))]
fn check_dependencies(input_path: &PyAny, search_paths: Vec<String>) -> PyResult<PyObject> {
    run_klvm_compilation(
        CompileKlvmSource::SourcePath(input_path),
        CompileKlvmAction::CheckDependencies,
        search_paths,
        None,
    )
}

#[pyclass]
struct PythonRunStep {
    ended: bool,

    tx: Sender<bool>,
    rx: Receiver<(bool, Option<BTreeMap<String, String>>)>,
}

fn runstep(myself: &mut PythonRunStep) -> PyResult<Option<PyObject>> {
    if myself.ended {
        return Ok(None);
    }

    // Let the runner know we want another step.
    myself
        .tx
        .send(false)
        .map_err(|_e| CldbError::new_err("error sending to service thread"))?;

    // Receive the step result.
    let res = myself
        .rx
        .recv()
        .map_err(|_e| CldbError::new_err("error receiving from service thread"))?;
    if res.0 {
        myself.ended = true;
    }

    // Return a dict if one was returned.
    let dict_result = res.1.map(|m| {
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            for (k, v) in m.iter() {
                let _ = dict.set_item(PyString::new(py, k), PyString::new(py, v));
            }
            dict.to_object(py)
        })
    });
    Ok(dict_result)
}

#[pymethods]
impl PythonRunStep {
    fn is_ended(&self) -> PyResult<bool> {
        Ok(self.ended)
    }

    fn drop(&mut self) {
        // If we can't send here then there's probably nothing we can do anyway.
        let _ = self.tx.send(true);
    }

    fn step(&mut self, py: Python) -> PyResult<Option<PyObject>> {
        py.allow_threads(|| runstep(self))
    }
}

struct CldbSinglePythonOverride {
    pycode: Py<PyAny>,
}

impl CldbSinglePythonOverride {
    fn new(pycode: &Py<PyAny>) -> Self {
        CldbSinglePythonOverride {
            pycode: pycode.clone(),
        }
    }
}

impl CldbSingleBespokeOverride for CldbSinglePythonOverride {
    fn get_override(&self, env: Rc<SExp>) -> Result<Rc<SExp>, RunFailure> {
        Python::with_gil(|py| {
            let arg_value = klvm_value_to_python(py, env.clone());
            let res = self
                .pycode
                .call1(py, PyTuple::new(py, &vec![arg_value]))
                .map_err(|e| RunFailure::RunErr(env.loc(), format!("{}", e)))?;
            let res_ref: &PyAny = res.as_ref(py);
            python_value_to_klvm(res_ref)
        })
    }
}

#[pyfunction]
#[pyo3(signature = (hex_prog, hex_args, symbol_table, overrides=None, run_options=None))]
fn start_klvm_program(
    hex_prog: String,
    hex_args: String,
    symbol_table: Option<HashMap<String, String>>,
    overrides: Option<HashMap<String, Py<PyAny>>>,
    run_options: Option<HashMap<String, Py<PyAny>>>,
) -> PyResult<PythonRunStep> {
    let (command_tx, command_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();

    let print_only_value = Python::with_gil(|py| {
        let print_only_option = run_options
            .and_then(|h| h.get("print").map(|p| p.clone()))
            .unwrap_or_else(|| {
                let any: Py<PyAny> = PyBool::new(py, false).into();
                any
            });

        PyBool::new(py, true).compare(print_only_option)
    })?;

    let print_only = print_only_value == Ordering::Equal;

    thread::spawn(move || {
        let mut allocator = Allocator::new();
        let runner = Rc::new(DefaultProgramRunner::new());
        let mut prim_map = HashMap::new();
        let cmd_input = command_rx;
        let result_output = result_tx;
        let prog_srcloc = Srcloc::start("*program*");
        let args_srcloc = Srcloc::start("*args*");

        for p in prims::prims().iter() {
            prim_map.insert(p.0.clone(), Rc::new(p.1.clone()));
        }

        let use_symbol_table = symbol_table.unwrap_or_default();
        let program =
            match hex_to_modern_sexp(&mut allocator, &use_symbol_table, prog_srcloc, &hex_prog) {
                Ok(v) => v,
                Err(_) => {
                    return;
                }
            };
        let args = match hex_to_modern_sexp(&mut allocator, &HashMap::new(), args_srcloc, &hex_args)
        {
            Ok(v) => v,
            Err(_) => {
                return;
            }
        };

        let mut overrides_table: HashMap<String, Box<dyn CldbSingleBespokeOverride>> =
            HashMap::new();
        if let Some(t) = overrides {
            for (k, v) in t.iter() {
                let override_fun_callable = CldbSinglePythonOverride::new(v);
                overrides_table.insert(k.clone(), Box::new(override_fun_callable));
            }
        }
        let override_runnable = CldbOverrideBespokeCode::new(use_symbol_table, overrides_table);

        let step = start_step(program, args);
        let cldbenv = CldbRunEnv::new(None, Rc::new(vec![]), Box::new(override_runnable));
        let mut cldbrun = CldbRun::new(runner, Rc::new(prim_map), Box::new(cldbenv), step);
        cldbrun.set_print_only(print_only);

        loop {
            match cmd_input.recv() {
                Ok(end_run) => {
                    if end_run || cldbrun.is_ended() {
                        let _ = result_output.send((true, None));
                        return;
                    }

                    let result = cldbrun.step(&mut allocator);
                    let is_ended = cldbrun.is_ended();
                    match result_output.send((is_ended, result)) {
                        Ok(_) => {}
                        Err(_) => {
                            return;
                        }
                    }
                }
                Err(_) => {
                    let _ = result_output.send((true, None));
                    return;
                }
            }
        }
    });

    Ok(PythonRunStep {
        ended: false,
        tx: command_tx,
        rx: result_rx,
    })
}

#[pyfunction]
#[pyo3(signature = (tool_name, args, default_stage=2))]
fn launch_tool(tool_name: String, args: Vec<String>, default_stage: u32) -> Vec<u8> {
    let mut stdout = Stream::new(None);
    cmds::launch_tool(&mut stdout, &args, &tool_name, default_stage);
    stdout.get_value().data().clone()
}

#[pyfunction]
fn call_tool(tool_name: String, args: Vec<String>) -> PyResult<Vec<u8>> {
    let mut allocator = Allocator::new();
    let mut stdout = Stream::new(None);
    cmds::call_tool(&mut stdout, &mut allocator, &tool_name, &args)
        .map_err(|e| ToolError::new_err(e))?;
    Ok(stdout.get_value().data().clone())
}

fn compile_err_to_cldb_err(err: &CompileErr) -> PyErr {
    CldbError::new_err(format!("{}: {}", err.0, err.1))
}

fn run_err_to_cldb_err(err: RunFailure) -> PyErr {
    match err {
        RunFailure::RunExn(l, v) => CldbError::new_err(format!("{}: {}", l, v)),
        RunFailure::RunErr(l, e) => CldbError::new_err(format!("{}: {}", l, e)),
    }
}

fn find_function_hash(symbol_table: &HashMap<String, String>, f: &String) -> Option<String> {
    for (k, v) in symbol_table.iter() {
        if v == f {
            return Some(k.clone());
        }
    }
    None
}

// Given a program hex and symbols, compose the program that executes a given
// function with some given arguments as though the program's primary expression
// had been that.
// returns a string or {"error"...}
#[pyfunction]
pub fn compose_run_function(
    hex_prog: String,
    symbol_table: HashMap<String, String>,
    function_name: String,
) -> PyResult<String> {
    let mut allocator = Allocator::new();
    let loc = Srcloc::start(&"*py*".to_string());
    let function_hash = match find_function_hash(&symbol_table, &function_name) {
        Some(f) => f,
        _ => {
            return Err(compile_err_to_cldb_err(&CompileErr(
                loc.clone(),
                format!("function not found in symbols: {}", function_name),
            )));
        }
    };
    let program = hex_to_modern_sexp(&mut allocator, &symbol_table, loc.clone(), &hex_prog)
        .map_err(run_err_to_cldb_err)?;
    let main_env = match extract_program_and_env(program.clone()) {
        Some(em) => em,
        _ => {
            return Err(compile_err_to_cldb_err(&CompileErr(
                program.loc(),
                "could not extract env from program".to_string(),
            )));
        }
    };
    let hash_bytes =
        match Bytes::new_validated(Some(UnvalidatedBytesFromType::Hex(function_hash.clone()))) {
            Ok(x) => x,
            Err(e) => {
                return Err(compile_err_to_cldb_err(&CompileErr(
                    program.loc(),
                    format!("bad function hash: ({}) {}", function_hash, e),
                )));
            }
        };
    let function_path = match path_to_function(main_env.1.clone(), &hash_bytes.data().clone()) {
        Some(p) => p,
        _ => {
            return Err(compile_err_to_cldb_err(&CompileErr(
                program.loc(),
                format!(
                    "could not find function with hash from symbols: {}",
                    function_name
                ),
            )));
        }
    };

    let new_program = rewrite_in_program(function_path, main_env.1);
    let mut result_stream = Stream::new(None);
    let klvm_rs_value =
        convert_to_klvm_rs(&mut allocator, new_program).map_err(run_err_to_cldb_err)?;
    sexp_to_stream(&mut allocator, klvm_rs_value, &mut result_stream);
    Ok(result_stream.get_value().hex())
}

#[pymodule]
fn klvm_tools_rs(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_submodule(create_cmds_module(py)?)?;
    m.add_submodule(create_binutils_module(py)?)?;

    m.add("CldbError", py.get_type::<CldbError>())?;
    m.add("CompError", py.get_type::<CompError>())?;

    m.add_function(wrap_pyfunction!(compile_klvm, m)?)?;
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    m.add_function(wrap_pyfunction!(get_version, m)?)?;
    m.add_function(wrap_pyfunction!(start_klvm_program, m)?)?;
    m.add_function(wrap_pyfunction!(launch_tool, m)?)?;
    m.add_function(wrap_pyfunction!(call_tool, m)?)?;
    m.add_function(wrap_pyfunction!(check_dependencies, m)?)?;
    m.add_function(wrap_pyfunction!(compose_run_function, m)?)?;
    m.add_class::<PythonRunStep>()?;
    Ok(())
}
