#[cfg(test)]
use rand::prelude::*;
#[cfg(test)]
use rand::Rng;
#[cfg(test)]
use rand_chacha::ChaChaRng;

use num_bigint::ToBigInt;

use std::borrow::Borrow;
use std::fs;
use std::rc::Rc;

use klvm_rs::allocator::Allocator;

use crate::classic::klvm::__type_compatibility__::{bi_one, bi_zero, Bytes, BytesFromType};
use crate::classic::klvm::casts::{bigint_to_bytes_klvm, bigint_to_bytes_unsigned};
use crate::classic::klvm_tools::stages::stage_0::DefaultProgramRunner;

use crate::compiler::klvm::{parse_and_run, sha256tree};
use crate::compiler::runtypes::RunFailure;
use crate::compiler::sexp::{parse_sexp, SExp};
use crate::compiler::srcloc::Srcloc;
use crate::tests::classic::run::RandomKlvmNumber;

use crate::util::Number;

fn test_compiler_klvm(to_run: &String, args: &String) -> Result<Rc<SExp>, RunFailure> {
    let mut allocator = Allocator::new();
    let runner = Rc::new(DefaultProgramRunner::new());
    parse_and_run(
        &mut allocator,
        runner,
        &"*test*".to_string(),
        &to_run,
        &args,
    )
}

#[test]
fn test_sexp_parse_1() {
    let loc = Srcloc::start(&"*test*".to_string());
    let res = parse_sexp(loc, "()".bytes()).map(|x| x[0].to_string());
    assert_eq!(res, Ok("()".to_string()));
}

#[test]
fn test_sexp_parse_2() {
    let loc = Srcloc::start(&"*test*".to_string());
    let res = parse_sexp(loc, "55".bytes()).and_then(|x| x[0].get_number());
    assert_eq!(res, Ok(55_i32.to_bigint().unwrap()));
}

#[test]
fn test_sexp_parse_3() {
    let loc = Srcloc::start(&"*test*".to_string());
    let res = parse_sexp(loc, "hello".bytes()).and_then(|x| x[0].get_number());
    assert_eq!(res, Ok(448378203247_i64.to_bigint().unwrap()));
}

#[test]
fn test_sexp_parse_4() {
    let loc = Srcloc::start(&"*test*".to_string());
    let res = parse_sexp(loc, "\"hello\"".bytes()).and_then(|x| x[0].get_number());
    assert_eq!(res, Ok(448378203247_i64.to_bigint().unwrap()));
}

#[test]
fn test_sexp_parse_5() {
    let loc = Srcloc::start(&"*test*".to_string());
    let res = parse_sexp(loc, "(3 . 4)".bytes()).map(|x| x[0].to_string());
    assert_eq!(res, Ok("(3 . 4)".to_string()));
}

#[test]
fn test_sexp_parse_6() {
    let loc = Srcloc::start(&"*test*".to_string());
    let res = parse_sexp(loc, "(\" \" . 3)".bytes()).map(|x| x[0].to_string());
    assert_eq!(res, Ok("(\" \" . 3)".to_string()));
}

#[test]
fn test_sexp_parse_7() {
    let loc = Srcloc::start(&"*test*".to_string());
    let res = parse_sexp(loc, "(3 . \" \")".bytes()).map(|x| x[0].to_string());
    assert_eq!(res, Ok("(3 . \" \")".to_string()));
}

#[test]
fn test_sexp_parse_8() {
    let loc = Srcloc::start(&"*test*".to_string());
    let res = parse_sexp(
        loc,
        "(a (q 2 4 (c 2 (c 6 ()))) (c (q 13 26729 \"there\" \"fool\") 1))".bytes(),
    )
    .map(|x| x[0].to_string());
    assert_eq!(
        res,
        Ok("(a (q 2 4 (c 2 (c 6 ()))) (c (q 13 26729 \"there\" \"fool\") 1))".to_string())
    );
}

#[test]
fn test_klvm_1() {
    let loc = Srcloc::start(&"*test*".to_string());
    let result = test_compiler_klvm(
        &"(a (q 2 4 (c 2 (c 6 ()))) (c (q 13 26729 \"there\" \"fool\") 1))".to_string(),
        &"()".to_string(),
    )
    .unwrap();
    let want = parse_sexp(loc, "(\"there\" \"fool\")".bytes()).unwrap();

    assert!(result.equal_to(want[0].borrow()));
}

#[test]
fn test_klvm_2() {
    let loc = Srcloc::start(&"*test*".to_string());
    let result =
        test_compiler_klvm(
            &"(a (q 2 (q 2 2 (c 2 (c 3 (q)))) (c (q 2 (i 5 (q 4 (q . 4) (c 9 (c (a 2 (c 2 (c 13 (q)))) (q)))) (q 1)) 1) 1)) 1)".to_string(),
            &"(1 2)".to_string(),
        ).unwrap();
    let want = parse_sexp(loc, "(4 1 (4 2 ()))".bytes()).unwrap();

    assert!(result.equal_to(want[0].borrow()));
}

#[test]
fn test_klvm_3() {
    let loc = Srcloc::start(&"*test*".to_string());
    let result = test_compiler_klvm(
        &"(2 (3 (1) (1 16 (1 . 1) (1 . 3)) (1 16 (1 . 5) (1 . 8))) 1)".to_string(),
        &"()".to_string(),
    )
    .unwrap();
    let want = parse_sexp(loc, "13".bytes()).unwrap();

    assert!(result.equal_to(want[0].borrow()));
}

#[test]
fn test_klvm_4() {
    let loc = Srcloc::start(&"*test*".to_string());
    let result = test_compiler_klvm(
        &"(divmod (1 . 300000003392) (1 . 10000000))".to_string(),
        &"()".to_string(),
    )
    .unwrap();
    let want = parse_sexp(loc, "(30000 . 3392)".bytes()).unwrap();

    assert!(result.equal_to(want[0].borrow()));
}

#[cfg(test)]
fn does_number_need_extension_byte(n: Number) -> bool {
    let mut iv = n.clone();
    let eight_ones = 255_u32.to_bigint().unwrap();
    while iv > eight_ones {
        iv /= eight_ones.clone() + bi_one();
    }
    iv > 127_u32.to_bigint().unwrap()
}

// This seems like a reasonable way to unit test just the conversion functions
// in a broad way i suppose.
#[test]
fn test_random_int_just_the_conversion_functions_and_no_other_things_from_the_stack_1() {
    let mut rng = ChaChaRng::from_entropy();
    for _ in 1..=200 {
        let number_spec: RandomKlvmNumber = rng.gen();

        let to_bytes_klvm = bigint_to_bytes_klvm(&number_spec.intended_value).raw();
        let to_bytes_unsigned = if number_spec.intended_value < bi_zero() {
            None
        } else {
            Some(bigint_to_bytes_unsigned(&number_spec.intended_value).raw())
        };

        if number_spec.intended_value == bi_zero() {
            assert!(to_bytes_klvm.is_empty());
            if let Some(usbi) = &to_bytes_unsigned {
                assert!(usbi.is_empty());
            }
            continue;
        }

        // Determine whether an extension byte would be needed.
        let need_ext_byte = does_number_need_extension_byte(number_spec.intended_value.clone());
        if need_ext_byte {
            assert_eq!(to_bytes_klvm[0], 0);
            if let Some(usbi) = &to_bytes_unsigned {
                assert_eq!(usbi[0] & 0x80, 0x80);
            }
        }

        // Check klvm repr
        let one_byte_size = 256_u32.to_bigint().unwrap();
        let mut check_value = number_spec.intended_value.clone();
        for b in to_bytes_klvm.iter().rev() {
            let isolated_byte = check_value.clone() & (one_byte_size.clone() - bi_one());
            check_value >>= 8;
            assert_eq!(isolated_byte, b.to_bigint().unwrap());
        }

        // Check unsigned repr
        check_value = number_spec.intended_value.clone();
        if let Some(usbi) = &to_bytes_unsigned {
            for b in usbi.iter().rev() {
                let isolated_byte = check_value.clone() & (one_byte_size.clone() - bi_one());
                check_value >>= 8;
                assert_eq!(isolated_byte, b.to_bigint().unwrap());
            }
        }
    }
}

#[test]
fn test_sha256_tree_hash() {
    let filename = "resources/tests/assert.klvm.out";
    let assert_klvm_compiled = fs::read_to_string(filename).expect("should exist");
    let parsed = parse_sexp(
        Srcloc::start(filename),
        assert_klvm_compiled.as_bytes().iter().copied(),
    )
    .expect("should parse");
    let hash_result = Bytes::new(Some(BytesFromType::Raw(sha256tree(parsed[0].clone())))).hex();
    assert_eq!(
        hash_result,
        "156e86309040ed6bbfee805c9c6ca7eebc140490bd1b97d6d18fb8ebc91fd05a"
    );
}
