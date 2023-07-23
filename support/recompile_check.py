import os
import subprocess
import traceback

recompile_list = [
    'block_program_zero.klvm',
    'calculate_synthetic_public_key.klvm',
    'chiklisp_deserialisation.klvm',
    'decompress_coin_spend_entry.klvm',
    'decompress_coin_spend_entry_with_prefix.klvm',
    'decompress_puzzle.klvm',
    'delegated_tail.klvm',
    'did_innerpuz.klvm',
    'everything_with_signature.klvm',
    'genesis_by_coin_id.klvm',
    'genesis_by_puzzle_hash.klvm',
    'lock.inner.puzzle.klvm',
    'nft_metadata_updater_default.klvm',
    'nft_metadata_updater_updateable.klvm',
    'nft_ownership_layer.klvm',
    'nft_ownership_transfer_program_one_way_claim_with_royalties.klvm',
    'nft_state_layer.klvm',
    'p2_conditions.klvm',
    'p2_delegated_conditions.klvm',
    'p2_delegated_puzzle.klvm',
    'p2_delegated_puzzle_or_hidden_puzzle.klvm',
    'p2_m_of_n_delegate_direct.klvm',
    'p2_puzzle_hash.klvm',
    'p2_singleton.klvm',
    'p2_singleton_or_delayed_puzhash.klvm',
    'pool_member_innerpuz.klvm',
    'pool_waitingroom_innerpuz.klvm',
    'rom_bootstrap_generator.klvm',
    'settlement_payments.klvm',
    'sha256tree_module.klvm',
    'singleton_launcher.klvm',
    'singleton_top_layer.klvm',
    'singleton_top_layer_v1_1.klvm',
    'test_generator_deserialize.klvm',
    'test_multiple_generator_input_arguments.klvm'
]

for fname in recompile_list:
    hexfile = f'./chik/wallet/puzzles/{fname}.hex'
    hexdata = open(hexfile).read().strip()
    os.unlink(hexfile)
    try:
        compiled = subprocess.check_output(['../target/release/run', '-i', 'chik/wallet/puzzles/', f'chik/wallet/puzzles/{fname}']).strip()
        recompile = subprocess.check_output(['../target/release/opc', compiled]).decode('utf8').strip()
    except:
        print(f'compiling {fname}')
        traceback.print_exc()

    if hexdata != recompile:
        print(f'*** COMPILE RESULTED IN DIFFERENT OUTPUT FOR FILE {fname}')
        assert hexdata == recompile
