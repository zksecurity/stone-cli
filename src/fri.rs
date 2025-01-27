use crate::config::FriParameters;

const DEFAULT_N_QUERIES: u32 = 16;
const DEFAULT_PROOF_OF_WORK_BITS: u32 = 32;

/// Implements ceil(log2(x)).
fn ceil_log2(x: u32) -> u32 {
    x.next_power_of_two().ilog2()
}

/// Computes the FRI steps list based on the specified parameters.
///
/// This computation is based on the documentation of the Stone prover:
/// # log₂(#steps) + 4 = log₂(last_layer_degree_bound) + ∑fri_step_list
/// # log₂(#steps) = log₂(last_layer_degree_bound) + ∑fri_step_list - 4
/// # ∑fri_step_list = log₂(#steps) + 4 - log₂(last_layer_degree_bound)
///
/// * `nb_steps_log`: Ceiled log₂ of the number of Cairo steps of the program.
/// * `last_layer_degree_bound_log`: Ceiled log₂ of the last layer degree bound.
/// * `max_step_value`: Maximum value for each step. All elements will be in the range
///   [0, `max_step_value`].
///
/// Returns The FRI steps list.
fn compute_fri_steps(
    nb_steps_log: u32,
    last_layer_degree_bound_log: u32,
    max_step_value: u32,
) -> Vec<u32> {
    let sum_of_fri_steps = nb_steps_log + 4 - last_layer_degree_bound_log;
    let quotient = (sum_of_fri_steps / max_step_value) as usize;
    let remainder = sum_of_fri_steps % max_step_value;

    let mut fri_steps = vec![max_step_value; quotient];
    if remainder > 0 {
        fri_steps.push(remainder);
    }

    fri_steps
}

pub trait FriComputer {
    fn compute_fri_parameters(&self, nb_steps: u32) -> FriParameters;
}

pub struct DefaultFriComputer;

impl FriComputer for DefaultFriComputer {
    fn compute_fri_parameters(&self, nb_steps: u32) -> FriParameters {
        let last_layer_degree_bound = 64;

        let nb_steps_log = ceil_log2(nb_steps);
        let last_layer_degree_bound_log = ceil_log2(last_layer_degree_bound);
        let max_step_value = 4;

        // The first FRI step must be 0
        let mut fri_steps = vec![0];
        fri_steps.extend(compute_fri_steps(
            nb_steps_log,
            last_layer_degree_bound_log,
            max_step_value,
        ));

        FriParameters {
            fri_step_list: Some(fri_steps),
            last_layer_degree_bound: Some(last_layer_degree_bound),
            n_queries: Some(DEFAULT_N_QUERIES),
            proof_of_work_bits: Some(DEFAULT_PROOF_OF_WORK_BITS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(2, 1)]
    #[case(32, 5)]
    #[case(1000, 10)]
    #[case(524288, 19)]
    fn test_ceil_log2(#[case] x: u32, #[case] expected: u32) {
        let log = ceil_log2(x);
        assert_eq!(log, expected);
    }

    #[rstest]
    #[case(32768, vec ! [0, 4, 4, 4, 1])]
    #[case(524288, vec ! [0, 4, 4, 4, 4, 1])]
    #[case(768, vec ! [0, 4, 4])]
    fn test_compute_fri_parameters_default(#[case] nb_steps: u32, #[case] expected: Vec<u32>) {
        let expected_last_layer_degree_bound = 64;
        let fri_parameters = DefaultFriComputer.compute_fri_parameters(nb_steps);
        assert_eq!(fri_parameters.fri_step_list, Some(expected));
        assert_eq!(
            fri_parameters.last_layer_degree_bound,
            Some(expected_last_layer_degree_bound)
        );
    }
}
