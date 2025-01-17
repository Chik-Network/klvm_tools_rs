import os
import pytest
import random
from itertools import permutations
from typing import List
#from hsms.streamables.program import Program
#from steprun import diag_run_klvm, compile_module_with_symbols
#from lib.program import Program
from pathlib import Path
from klvm_rs import Program
from lib.steprun import diag_run_klvm, compile_module_with_symbols
from klvm_tools_rs import get_version

print(f"klvm_tools_rs version is {get_version()}")
#include_dirs = os.getcwd()
include_dirs = [Path(__file__).parent, Path(__file__).parent.parent / "lib"]
Program.set_run_unsafe_max_cost(11000000000)

print(f"XXX: {include_dirs}")
compile_module_with_symbols(include_dirs, 'smoke_test_deep_compare.clsp')
compare_program = Program.from_bytes(bytes.fromhex(open('smoke_test_deep_compare.klvm.hex').read()))

compile_module_with_symbols(include_dirs, 'smoke_test_sort.clsp')
sort_program = Program.from_bytes(bytes.fromhex(open('smoke_test_sort.klvm.hex').read()))

compile_module_with_symbols(include_dirs, 'test_sort.clsp')
test_sort_program = Program.from_bytes(bytes.fromhex(open('test_sort.klvm.hex').read()))

compile_module_with_symbols(include_dirs, 'test_permutations.clsp')
test_permutations_program = Program.from_bytes(bytes.fromhex(open('test_permutations.klvm.hex').read()))

compile_module_with_symbols(include_dirs, 'test_reverse.clsp')
test_reverse_program = Program.from_bytes(bytes.fromhex(open('test_reverse.klvm.hex').read()))

compile_module_with_symbols(include_dirs, 'test_prepend.clsp')
test_prepend_program = Program.from_bytes(bytes.fromhex(open('test_prepend.klvm.hex').read()))

compile_module_with_symbols(include_dirs, 'test_range.clsp')
test_range_program = Program.from_bytes(bytes.fromhex(open('test_range.klvm.hex').read()))

compile_module_with_symbols(include_dirs, 'smoke_test_permutations.clsp')
smoke_test_permutations_program = Program.from_bytes(bytes.fromhex(open('smoke_test_permutations.klvm.hex').read()))

compile_module_with_symbols(include_dirs, 'test_handcalc.clsp')
test_handcalc_program = Program.from_bytes(bytes.fromhex(open('test_handcalc.klvm.hex').read()))

def as_atom_list(prg: Program) -> List[bytes]:
    """
    Pretend `prg` is a list of atoms. Return the corresponding
    python list of atoms.

    At each step, we always assume a node to be an atom or a pair.
    If the assumption is wrong, we exit early. This way we never fail
    and always return SOMETHING.
    """
    items = []
    obj = prg
    while True:
        pair = obj.pair
        if pair is None:
            break
        atom = pair[0].atom
        if atom is None:
            break
        items.append(atom)
        obj = pair[1]
    return items

def test_smoke_compare():
    compare_program.run(Program.to([]))

def test_handcalc():
    diag_run_klvm(test_handcalc_program, Program.to([]), 'test_handcalc.sym', {'print': True})

def proper_list_inner(result,cl):
    if hasattr(cl, 'pair') and cl.pair is not None:
        result.append(cl.pair[0])
        return proper_list_inner(result,cl.pair[1])
    else:
        return result

def proper_list(cl):
    result = []
    return proper_list_inner(result,cl)

def int_list(cl):
    return [Program.to(x).as_int() for x in as_atom_list(Program.to(cl))]

def de_none_list(l):
    return [x if x is not None else [] for x in l]

def with_random_lists(n,f):
    for length in range(n): # 0-10 length
        for i in range(1 + (3 * length)): # A few orders each
            orig_list = [random.randint(0,100) for x in range(length)]
            f(orig_list)

def test_prepend():
    for length1 in range(5):
        list_1 = list(range(length1))
        for length2 in range(length1):
            prepend_result = test_prepend_program.run([Program.to(list_1[:length2]),Program.to(list_1[length2:])])
            assert list_1 == int_list(prepend_result)

def test_reverse():
    def test_reverse_list(l):
        rev_args = Program.to([l])
        reversed_result = Program.to(list(reversed(l)))
        reversed_by_prog = test_reverse_program.run(rev_args)
        assert reversed_result == reversed_by_prog

    with_random_lists(10,test_reverse_list)

def test_range():
    for length in range(10):
        want_list = list(range(length))
        result = test_range_program.run(Program.to([length]))
        assert want_list == result

def do_test_permutations_of_size_n(n):
    try_list = [random.randint(0,100) for x in range(n)]
    want_set = list([list(v) for v in sorted(permutations(try_list))])
    listed_result = smoke_test_permutations_program.run(Program.to([try_list]))
    pl = proper_list(listed_result)
    perms_result = sorted([int_list(x) for x in de_none_list(pl)])
    assert want_set == perms_result

def test_permutations_0():
    do_test_permutations_of_size_n(0)

def test_permutations_1():
    do_test_permutations_of_size_n(1)

def test_permutations_2():
    n = 2
    all_a_string = 0x616161616161
    all_b_string = 0x626262626262
    for try_list in [[all_a_string,all_b_string], [all_b_string,all_a_string]]:
        want_set = list([list(v) for v in sorted(permutations(try_list))])
        listed_result = diag_run_klvm(smoke_test_permutations_program, Program.to([try_list]), 'smoke_test_permutations.sym', {'print': True})
        pl = proper_list(listed_result)
        perms_result = sorted([int_list(x) for x in de_none_list(pl)])
        assert want_set == perms_result

def test_chiklisp_sort_program():
    diag_run_klvm(test_sort_program, Program.to([]), 'test_sort.sym', {'print': True})

def test_permutations_n():
    for i in range(3,6):
        do_test_permutations_of_size_n(i)

def test_chiklisp_permutations_program():
    diag_run_klvm(test_permutations_program, Program.to([3, 5]), 'test_permutations.sym', {'print': True})

def test_smoke_sort():
    for length in range(7): # 0-7 length
        for i in range(1 + (3 * length)): # A few orders each
            orig_list = [random.randint(0,100) for x in range(length)]
            sort_args = Program.to([orig_list])
            sorted_list = Program.to(sorted(orig_list))
            sort_res = sort_program.run(sort_args)
            assert sort_res == sorted_list

if __name__ == '__main__':
    test_smoke_sort()
