use num::BigInt;

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBytes, PyList, PyTuple};

use std::borrow::Borrow;
use std::rc::Rc;

use crate::classic::klvm::__type_compatibility__::bi_zero;
use crate::compiler::runtypes::RunFailure;
use crate::compiler::sexp::SExp;
use crate::compiler::srcloc::Srcloc;

pub fn map_err_to_pyerr(srcloc: Srcloc, r: PyResult<Py<PyAny>>) -> Result<Py<PyAny>, RunFailure> {
    r.map_err(|e| RunFailure::RunErr(srcloc, format!("{}", e)))
}

pub fn python_value_to_klvm(py: Python, val: Py<PyAny>) -> Result<Rc<SExp>, RunFailure> {
    let srcloc = Srcloc::start("*python*");
    val.as_ref(py)
        .downcast::<PyList>()
        .ok()
        .map(|l| {
            if l.is_empty() {
                Ok(Rc::new(SExp::Nil(srcloc.clone())))
            } else {
                let mut result = SExp::Nil(srcloc.clone());
                for i_rev in 0..l.len() {
                    let i = l.len() - i_rev - 1;
                    let item = l.get_item(i as isize).extract();
                    let any_of_elt = map_err_to_pyerr(srcloc.clone(), item)?;
                    result = SExp::Cons(
                        srcloc.clone(),
                        python_value_to_klvm(py, any_of_elt)?,
                        Rc::new(result),
                    );
                }
                Ok(Rc::new(result))
            }
        })
        .map(Some)
        .unwrap_or_else(|| {
            val.as_ref(py)
                .downcast::<PyTuple>()
                .map(|t| {
                    if t.len() != 2 {
                        Err(RunFailure::RunErr(
                            srcloc.clone(),
                            "tuple must have len 2".to_string(),
                        ))
                    } else {
                        let any_of_e0 = map_err_to_pyerr(srcloc.clone(), t.get_item(0).extract())?;
                        let any_of_e1 = map_err_to_pyerr(srcloc.clone(), t.get_item(1).extract())?;
                        Ok(Rc::new(SExp::Cons(
                            srcloc.clone(),
                            python_value_to_klvm(py, any_of_e0)?,
                            python_value_to_klvm(py, any_of_e1)?,
                        )))
                    }
                })
                .ok()
        })
        .map(Some)
        .unwrap_or_else(|| {
            val.as_ref(py)
                .downcast::<PyBytes>()
                .map(|b| Ok(Rc::new(SExp::Atom(srcloc.clone(), b.as_bytes().to_vec()))))
                .ok()
        })
        .map(Some)
        .unwrap_or_else(|| {
            let stringified = format!("{}", val);
            stringified
                .parse::<BigInt>()
                .map(|i| {
                    if i == bi_zero() {
                        Ok(Rc::new(SExp::Nil(srcloc.clone())))
                    } else {
                        Ok(Rc::new(SExp::Integer(srcloc.clone(), i)))
                    }
                })
                .ok()
        })
        .unwrap_or_else(|| {
            Err(RunFailure::RunErr(
                srcloc.clone(),
                "no way to convert python value to klvm".to_string(),
            ))
        })
}

pub fn klvm_value_to_python(py: Python, val: Rc<SExp>) -> Py<PyAny> {
    val.proper_list()
        .map(|lst| {
            let mut vallist = Vec::new();
            for i in lst {
                vallist.push(klvm_value_to_python(py, Rc::new(i.clone())));
            }
            PyList::new(py, &vallist).into_py(py)
        })
        .unwrap_or_else(|| match val.borrow() {
            SExp::Cons(_, a, b) => PyTuple::new(
                py,
                vec![
                    klvm_value_to_python(py, a.clone()),
                    klvm_value_to_python(py, b.clone()),
                ],
            )
            .into_py(py),
            SExp::Integer(_, i) => {
                let int_val: Py<PyAny> = map_err_to_pyerr(
                    val.loc(),
                    py.eval(&format!("int({})", i), None, None)
                        .map(|x| x.into_py(py)),
                )
                .unwrap();
                int_val
            }
            SExp::Atom(_, v) => PyBytes::new(py, v).into_py(py),
            SExp::QuotedString(_, _, v) => PyBytes::new(py, v).into_py(py),
            SExp::Nil(_) => {
                let emptybytes: Vec<u8> = vec![];
                PyList::new(py, &emptybytes).into_py(py)
            }
        })
}
