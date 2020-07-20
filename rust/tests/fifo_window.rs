use rand::Rng;
use swag::*;
use std::collections::VecDeque;

mod common;
use common::*;

/// Macro for generating test cases for different algorithms.
macro_rules! test_matrix {
    {
        $(
            $name:ident => [$($module:ident::$algorithm:ident),*]
        ),*
    } => {
        $(
            mod $name {
                $(
                    #[test]
                    fn $module() {
                        super::$name::<swag::$module::$algorithm<_,_>>();
                    }
                )*
            }
        )*
    }
}

/// Basic test for integer sums.
fn test_basic<Window>()
where
    Window: FifoWindow<Int, Sum>,
{
    let mut window = Window::new();

    assert_eq!(window.query(), Int(0));

    window.push(Int(1));

    assert_eq!(window.query(), Int(1));

    window.push(Int(2));

    assert_eq!(window.query(), Int(3));

    window.push(Int(3));

    assert_eq!(window.query(), Int(6));

    window.pop();

    assert_eq!(window.query(), Int(5));
}

fn synthesize(size: usize) -> Vec<Int> {
    let mut rng = rand::thread_rng();
    (0..size)
        .map(|_| rng.gen_range(1, 5))
        .map(Int)
        .collect::<Vec<_>>()
}

/// Tries to aggregate the sum of 1K randomly generated integers.
fn test_sum<Window>()
where
    Window: FifoWindow<Int, Sum>,
{
    let values = synthesize(1_000);
    let sum = values.iter().fold(0, |acc, Int(x)| acc + x);
    let mut window = Window::new();
    for v in values.clone() {
        window.push(v);
    }
    assert_eq!(window.query(), Int(sum));
    for _ in values {
        window.pop();
    }
    assert_eq!(window.query(), Int(0));
}

/// Tries to find the maximum value out 1K randomly generated integers.
fn test_max<Window>()
where
    Window: FifoWindow<Int, Max>,
{
    let mut window = Window::new();
    let values = synthesize(1_000);
    let max = values.iter().map(|Int(x)| *x).max().unwrap();
    for v in values.clone() {
        window.push(v);
    }
    assert_eq!(window.query(), Int(max));
    for _ in values {
        window.pop();
    }
    assert_eq!(window.query(), Int(std::i64::MIN));
}

/// Fills a window with 1K elements and pushes/pops/queries 1K times.
fn test_push_pop_query<Window>()
where
    Window: FifoWindow<Int, Sum>,
{
    let mut window = Window::new();
    let values = synthesize(1_000);
    let sum = values.iter().fold(0, |acc, Int(x)| acc + x);
    for v in values.clone() {
        window.push(v);
    }
    for v in values {
        window.push(v);
        window.pop();
        window.query();
        assert_eq!(window.query(), Int(sum));
    }
}

/// Pops more elements from a window than what it contains.
fn test_pop_full<Window>()
where
    Window: FifoWindow<Int, Sum>,
{
    let mut window = Window::new();
    window.push(Int(0));
    window.push(Int(0));
    window.pop();
    window.pop();
    window.pop();
}

/// Pushes and pops some elements. Regression test for [fcde4cb].
fn test_push_pop<Window>()
where
    Window: FifoWindow<Int, Sum>,
{
    let mut window = Window::new();
    window.push(Int(1));
    window.push(Int(2));
    window.push(Int(3));
    window.pop();
    window.push(Int(4));
    window.push(Int(5));
}

enum Event {
    Push(i64),
    Pop,
}

type Workload = Vec<Event>;

fn generate_workload(size: usize) -> Workload {
    let mut rng = rand::thread_rng();
    (0..size)
        .map(|_| match rng.gen_range(0, 1) {
            0 => Event::Push(rng.gen_range(0, 100)),
            _ => Event::Pop,
        })
        .collect::<Vec<_>>()
}

fn test_random_workload<Window>()
where
    Window: FifoWindow<Int, Sum>,
{
    let mut window = Window::new();
    let workload = generate_workload(1_000);
    let mut sum = 0;
    let mut elems = VecDeque::new();
    for event in workload {
        match event {
            Event::Push(x) => {
                sum += x;
                elems.push_back(x);
                window.push(Int(x));
            },
            Event::Pop => {
                let x = elems.pop_front().unwrap();
                sum -= x;
                window.pop();
            },
        }
        assert_eq!(window.query(), Int(sum));
    }
}

test_matrix! {
    test_basic
        => [ recalc::ReCalc, soe::SoE, reactive::Reactive, two_stacks::TwoStacks, daba::DABA ],
    test_sum
        => [ recalc::ReCalc, soe::SoE, reactive::Reactive, two_stacks::TwoStacks, daba::DABA ],
    test_max
        => [ recalc::ReCalc,           reactive::Reactive, two_stacks::TwoStacks, daba::DABA ],
    test_push_pop_query
        => [ recalc::ReCalc, soe::SoE, reactive::Reactive, two_stacks::TwoStacks, daba::DABA ],
    test_pop_full
        => [ recalc::ReCalc, soe::SoE, reactive::Reactive, two_stacks::TwoStacks, daba::DABA ],
    test_push_pop
        => [ recalc::ReCalc, soe::SoE, reactive::Reactive, two_stacks::TwoStacks, daba::DABA ],
    test_random_workload
        => [ recalc::ReCalc, soe::SoE, reactive::Reactive, two_stacks::TwoStacks, daba::DABA ]
}
