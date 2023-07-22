"""
These tests check that the `klvmc` utility methods
continue to work with the `include` keyword, and produce
the expected output. It's not intended to be a complete
test of the compiler, just the `klvmc` api.
"""

from tempfile import TemporaryDirectory

from klvm_tools import klvmc


INCLUDE_CODE = "((defconstant FOO 6001))"
MAIN_CODE = """(mod (VALUE) (include "include.klvm") (+ VALUE FOO))"""
EXPECTED_HEX_OUTPUT = "ff10ff02ffff0182177180"

# `EXPECTED_HEX_OUTPUT` disassembles to "(+ 2 (q . 6001))"


def test_compile_klvm_text():
    with TemporaryDirectory() as include_dir:
        include_path = f"{include_dir}/include.klvm"
        with open(include_path, "w") as f:
            f.write(INCLUDE_CODE)
        output = klvmc.compile_klvm_text(MAIN_CODE, search_paths=[include_dir])
        assert repr(output) == f"SExp({EXPECTED_HEX_OUTPUT})"


def test_compile_klvm():
    with TemporaryDirectory() as include_dir:
        with TemporaryDirectory() as source_dir:
            with open(f"{include_dir}/include.klvm", "w") as f:
                f.write(INCLUDE_CODE)
            main_path = f"{source_dir}/main.klvm"
            main_output = f"{source_dir}/main.hex"
            with open(main_path, "w") as f:
                f.write(MAIN_CODE)
            output = klvmc.compile_klvm(
                main_path, main_output, search_paths=[include_dir]
            )
            t = open(output).read()
            assert t == f"{EXPECTED_HEX_OUTPUT}\n"
