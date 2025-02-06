%builtins keccak
from starkware.cairo.common.cairo_builtins import KeccakBuiltin
from starkware.cairo.common.keccak_state import KeccakBuiltinState

// A helper function that hashes the given state `n` times.
func repeat_hash{keccak_ptr: KeccakBuiltin*}(
    state: KeccakBuiltinState, n: felt
) -> KeccakBuiltinState {
    if (n == 0) {
        // If n is 0, we've done all the required hashing.
        return (state);
    }

    // Provide the current state as input to Keccak.
    assert keccak_ptr[0].input = state;
    // Read the output of the hash.
    let output = keccak_ptr[0].output;
    // Advance the keccak pointer for the next iteration.
    let keccak_ptr = keccak_ptr + KeccakBuiltin.SIZE;

    // Recursively call repeat_hash with output as the new state and n-1 as the new count.
    return repeat_hash(output, n - 1);
}

// The main function now accepts a parameter `n` that indicates how many times to hash.
func main{keccak_ptr: KeccakBuiltin*}() {
    alloc_locals;

    local iterations;
    %{ ids.iterations = program_input['iterations'] %}

    // Define an initial Keccak state (can be replaced with your desired initial input).
    let initial_state = KeccakBuiltinState(1, 2, 3, 4, 5, 6, 7, 8);

    // Apply the hash n times.
    let final_state = repeat_hash(initial_state, iterations);

    // final_state now holds the Keccak state after n iterations of hashing.
    return ();
}
