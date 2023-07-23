import * as klvm_tools_rs from './build/klvm_tools_rs.js';
import {hexlify, unhexlify} from 'binascii';

export function bytestring(s) {
    return unhexlify(s);
}

export function run_program(program, args, symbols, overrides) {
    let runner = klvm_tools_rs.create_klvm_runner(program, args, symbols, overrides);
    if (runner.error) {
        console.log(runner.error);
        return;
    }

    var ended = null;

    do {
        var result = klvm_tools_rs.run_step(runner);
        if (result !== null) {
            if (result.Final !== undefined) {
                ended = result.Final;
                break;
            }
            if (result.Failure !== undefined) {
                ended = result.Failure;
                break;
            }
        }
    } while (ended === null);

    let finished = klvm_tools_rs.final_value(runner);
    klvm_tools_rs.remove_klvm_runner(runner);
    return finished;
};
