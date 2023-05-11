pub mod accumulator_id;
pub mod compute_state;
pub mod container;
pub mod shard_state;
pub mod shuffle_state;

pub trait StateType: PartialEq + Clone + std::fmt::Debug + Send + Sync + 'static {}

impl<T: PartialEq + Clone + std::fmt::Debug + Send + Sync + 'static> StateType for T {}

#[cfg(test)]
mod state_test {

    use rand::Rng;

    use crate::{
        core::state::{
            accumulator_id::accumulators,
            compute_state::{ComputeStateMap, ComputeStateVec},
            container::merge_2_vecs,
            shard_state::ShardComputeState,
            shuffle_state::ShuffleComputeState,
        },
        db::graph::Graph,
    };

    #[quickcheck]
    fn check_merge_2_vecs(mut a: Vec<usize>, b: Vec<usize>) {
        let len_a = a.len();
        let len_b = b.len();

        merge_2_vecs(&mut a, &b, |ia, ib| *ia = usize::max(*ia, *ib));

        assert_eq!(a.len(), usize::max(len_a, len_b));

        for (i, expected) in a.iter().enumerate() {
            match (a.get(i), b.get(i)) {
                (Some(va), Some(vb)) => assert_eq!(*expected, usize::max(*va, *vb)),
                (Some(va), None) => assert_eq!(*expected, *va),
                (None, Some(vb)) => assert_eq!(*expected, *vb),
                (None, None) => assert!(false, "value should exist in either a or b"),
            }
        }
    }

    fn tiny_graph(n_shards: usize) -> Graph {
        let g = Graph::new(n_shards);

        g.add_vertex(1, 1, &vec![]).unwrap();
        g.add_vertex(1, 2, &vec![]).unwrap();
        g.add_vertex(1, 3, &vec![]).unwrap();
        g
    }

    #[test]
    fn min_aggregates_for_3_keys() {
        let g = tiny_graph(1);

        let min = accumulators::min(0);

        let mut state_map: ShardComputeState<ComputeStateVec> = ShardComputeState::new();

        // create random vec of numbers
        let mut rng = rand::thread_rng();
        let mut vec = vec![];
        let mut actual_min = i32::MAX;
        for _ in 0..100 {
            let i = rng.gen_range(0..100);
            actual_min = actual_min.min(i);
            vec.push(i);
        }

        for a in vec {
            state_map.accumulate_into(0, 0, a, &min);
            state_map.accumulate_into(0, 1, a, &min);
            state_map.accumulate_into(0, 2, a, &min);
        }

        let actual = state_map.finalize(0, &min, 0, &g);
        assert_eq!(
            actual,
            Some(vec![(1, actual_min), (2, actual_min), (3, actual_min)])
        );
    }

    #[test]
    fn avg_aggregates_for_3_keys() {
        let g = tiny_graph(2);

        let avg = accumulators::avg(0);

        let mut state_map: ShardComputeState<ComputeStateMap> = ShardComputeState::new();

        // create random vec of numbers
        let mut rng = rand::thread_rng();
        let mut vec = vec![];
        let mut sum = 0;
        for _ in 0..100 {
            let i = rng.gen_range(0..100);
            sum += i;
            vec.push(i);
        }

        for a in vec {
            state_map.accumulate_into(0, 0, a, &avg);
            state_map.accumulate_into(0, 1, a, &avg);
            state_map.accumulate_into(0, 2, a, &avg);
        }

        let actual_avg = sum / 100;
        let actual = state_map.finalize(0, &avg, 0, &g);
        assert_eq!(
            actual,
            Some(vec![(0, actual_avg), (1, actual_avg), (2, actual_avg)])
        );
    }

    #[test]
    fn top3_aggregates_for_3_keys() {
        let g = tiny_graph(1);

        let top3 = accumulators::topk::<i32, 3>(0);

        let mut state_map: ShardComputeState<ComputeStateVec> = ShardComputeState::new();

        for a in 0..100 {
            state_map.accumulate_into(0, 0, a, &top3);
            state_map.accumulate_into(0, 1, a, &top3);
            state_map.accumulate_into(0, 2, a, &top3);
        }
        let expected = vec![99, 98, 97];

        let actual = state_map.finalize(0, &top3, 0, &g);
        assert_eq!(
            actual,
            Some(vec![
                (1, expected.clone()),
                (2, expected.clone()),
                (3, expected.clone())
            ])
        );
    }

    #[test]
    fn sum_aggregates_for_3_keys() {
        let g = tiny_graph(1);

        let sum = accumulators::sum(0);

        let mut state: ShardComputeState<ComputeStateMap> = ShardComputeState::new();

        // create random vec of numbers
        let mut rng = rand::thread_rng();
        let mut vec = vec![];
        let mut actual_sum = 0;
        for _ in 0..100 {
            let i = rng.gen_range(0..100);
            actual_sum += i;
            vec.push(i);
        }

        for a in vec {
            state.accumulate_into(0, 0, a, &sum);
            state.accumulate_into(0, 1, a, &sum);
            state.accumulate_into(0, 2, a, &sum);
        }

        let actual = state.finalize(0, &sum, 0, &g);

        assert_eq!(
            actual,
            Some(vec![(0, actual_sum), (1, actual_sum), (2, actual_sum)])
        );
    }

    #[test]
    fn sum_aggregates_for_3_keys_2_parts() {
        let g = tiny_graph(2);

        let sum = accumulators::sum(0);

        let mut part1_state: ShuffleComputeState<ComputeStateMap> = ShuffleComputeState::new(2);
        let mut part2_state: ShuffleComputeState<ComputeStateMap> = ShuffleComputeState::new(2);

        // create random vec of numbers
        let mut rng = rand::thread_rng();
        let mut vec1 = vec![];
        let mut vec2 = vec![];
        let mut actual_sum_1 = 0;
        let mut actual_sum_2 = 0;
        for _ in 0..100 {
            // data for first partition
            let i = rng.gen_range(0..100);
            actual_sum_1 += i;
            vec1.push(i);

            // data for second partition
            let i = rng.gen_range(0..100);
            actual_sum_2 += i;
            vec2.push(i);
        }

        // 1 gets all the numbers
        // 2 gets the numbers from part1
        // 3 gets the numbers from part2
        for a in vec1 {
            part1_state.accumulate_into(0, 1, a, &sum);
            part1_state.accumulate_into(0, 2, a, &sum);
        }

        for a in vec2 {
            part2_state.accumulate_into(0, 1, a, &sum);
            part2_state.accumulate_into(0, 3, a, &sum);
        }

        println!("part1_state: {:?}", part1_state);
        println!("part2_state: {:?}", part2_state);

        let actual = part1_state.finalize(0, &sum, &g);

        assert_eq!(
            actual,
            vec![Some(vec![(2, actual_sum_1)]), Some(vec![(1, actual_sum_1)])]
        );

        let actual = part2_state.finalize(0, &sum, &g);

        assert_eq!(
            actual,
            vec![None, Some(vec![(1, actual_sum_2), (3, actual_sum_2)])]
        );

        ShuffleComputeState::merge_mut(&mut part1_state, &part2_state, &sum, 0);
        let actual = part1_state.finalize(0, &sum, &g);

        assert_eq!(
            actual,
            vec![
                Some(vec![(2, actual_sum_1)]),
                Some(vec![(1, (actual_sum_1 + actual_sum_2)), (3, actual_sum_2)]),
            ]
        );
    }

    #[test]
    fn min_sum_aggregates_for_3_keys_2_parts() {
        let g = tiny_graph(2);

        let sum = accumulators::sum(0);
        let min = accumulators::min(1);

        let mut part1_state: ShuffleComputeState<ComputeStateMap> = ShuffleComputeState::new(2);
        let mut part2_state: ShuffleComputeState<ComputeStateMap> = ShuffleComputeState::new(2);

        // create random vec of numbers
        let mut rng = rand::thread_rng();
        let mut vec1 = vec![];
        let mut vec2 = vec![];
        let mut actual_sum_1 = 0;
        let mut actual_sum_2 = 0;
        let mut actual_min_1 = 100;
        let mut actual_min_2 = 100;
        for _ in 0..100 {
            // data for first partition
            let i = rng.gen_range(0..100);
            actual_sum_1 += i;
            actual_min_1 = actual_min_1.min(i);
            vec1.push(i);

            // data for second partition
            let i = rng.gen_range(0..100);
            actual_sum_2 += i;
            actual_min_2 = actual_min_2.min(i);
            vec2.push(i);
        }

        // 1 gets all the numbers
        // 2 gets the numbers from part1
        // 3 gets the numbers from part2
        for a in vec1 {
            part1_state.accumulate_into(0, 1, a, &sum);
            part1_state.accumulate_into(0, 2, a, &sum);
            part1_state.accumulate_into(0, 1, a, &min);
            part1_state.accumulate_into(0, 2, a, &min);
        }

        for a in vec2 {
            part2_state.accumulate_into(0, 1, a, &sum);
            part2_state.accumulate_into(0, 3, a, &sum);
            part2_state.accumulate_into(0, 1, a, &min);
            part2_state.accumulate_into(0, 3, a, &min);
        }

        let actual = part1_state.finalize(0, &sum, &g);
        assert_eq!(
            actual,
            vec![Some(vec![(2, actual_sum_1)]), Some(vec![(1, actual_sum_1)])]
        );

        let actual = part1_state.finalize(0, &min, &g);
        assert_eq!(
            actual,
            vec![Some(vec![(2, actual_min_1)]), Some(vec![(1, actual_min_1)])]
        );

        let actual = part2_state.finalize(0, &sum, &g);
        assert_eq!(
            actual,
            vec![None, Some(vec![(1, actual_sum_2), (3, actual_sum_2)])]
        );

        let actual = part2_state.finalize(0, &min, &g);
        assert_eq!(
            actual,
            vec![None, Some(vec![(1, actual_min_2), (3, actual_min_2)])]
        );

        ShuffleComputeState::merge_mut(&mut part1_state, &part2_state, &sum, 0);
        let actual = part1_state.finalize(0, &sum, &g);

        assert_eq!(
            actual,
            vec![
                Some(vec![((2, actual_sum_1))]),
                Some(vec![(1, (actual_sum_1 + actual_sum_2)), (3, actual_sum_2)]),
            ]
        );

        ShuffleComputeState::merge_mut(&mut part1_state, &part2_state, &min, 0);
        let actual = part1_state.finalize(0, &min, &g);
        assert_eq!(
            actual,
            vec![
                Some(vec![(2, actual_min_1)]),
                Some(vec![(1, actual_min_1.min(actual_min_2)), (3, actual_min_2)]),
            ]
        );
    }
}