klvm_tools_rs
=

This is a second-hand port of chik's [klvm tools](https://github.com/Chik-Network/klvm_tools/) to rust via the work of
ChikMineJP porting to typescript.  This would have been a lot harder to
get to where it is without prior work mapping out the types of various
semi-dynamic things (thanks, ChikMineJP).

Some reasons for doing this are:

 - Chik switched the klvm implementation to rust: [klvm_rs](https://github.com/Chik-Network/klvm_rs), and this code may both pick up speed and track klvm better being in the same language.
 
 - I wrote a new compiler with a simpler, less intricate structure that should be easier to improve and verify in the future in ocaml: [ochiklisp](https://github.com/prozacchiwawa/ochiklisp).

 - Also it's faster even in this unoptimized form.

All acceptance tests i've brought over so far work, and more are being added.
As of now, I'm not aware of anything that shouldn't be authentic when running
these command line tools from klvm_tools in their equivalents in this repository

 - opc
 
 - opd
 
 - run
 
 - brun

 - repl
 
argparse was ported to javascript and I believe I have faithfully reproduced it
as it is used in cmds, so command line parsing should work similarly in all three
versions.

The directory structure is expected to be:

    src/classic  <-- any ported code with heritage pointing back to
                     the original chik repo.
                    
    src/compiler <-- a newer compiler (ochiklisp) with a simpler
                     structure.  Select new style compilation by
                     including a `(include *standard-cl-21*)`
                     form in your toplevel `mod` form.

Mac M1
===

Use ```cargo build --no-default-features``` due to differences in how mac m1 and
other platforms handle python extensions.

Use with chik-blockchain
===

    # Activate your venv, then
    $ maturin develop --release

