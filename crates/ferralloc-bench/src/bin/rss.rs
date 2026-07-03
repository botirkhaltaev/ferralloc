use std::{env, process::Command};

use ferralloc_bench::{
    allocator_target::{TARGETS, target_by_name},
    rss::RssReport,
    workload,
};

const CASES: &[RssCase] = &[
    RssCase {
        name: "single_size_churn_64",
        ops: 10_000,
    },
    RssCase {
        name: "size_boundary_sweep",
        ops: 10_000,
    },
    RssCase {
        name: "small_biased_random",
        ops: 20_000,
    },
    RssCase {
        name: "large_alloc_churn_256k",
        ops: 1_000,
    },
];

struct RssCase {
    name: &'static str,
    ops: usize,
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.get(1).is_some_and(|arg| arg == "--case") {
        run_case(&args);
        return;
    }

    RssReport::print_csv_header();

    for &target in TARGETS {
        for case in CASES {
            run_subprocess(target.name(), case.name);
        }
    }
}

fn run_subprocess(allocator: &str, workload: &str) {
    let exe = env::current_exe().unwrap();
    let output = Command::new(exe)
        .args(["--case", allocator, workload])
        .output()
        .unwrap();

    if !output.status.success() {
        panic!(
            "rss case failed for {allocator}/{workload}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    print!("{}", String::from_utf8_lossy(&output.stdout));
}

fn run_case(args: &[String]) {
    let allocator = args.get(2).map(String::as_str).expect("missing allocator");
    let workload_name = args.get(3).map(String::as_str).expect("missing workload");
    let target = target_by_name(allocator).expect("unknown allocator");
    let case = CASES
        .iter()
        .find(|case| case.name == workload_name)
        .expect("unknown workload");

    match case.name {
        "single_size_churn_64" => RssReport::measure(target.name(), case.name, case.ops, || {
            workload::single_size_churn(target, 64, case.ops);
        }),
        "size_boundary_sweep" => RssReport::measure(target.name(), case.name, case.ops, || {
            workload::size_boundary_sweep(target, case.ops);
        }),
        "small_biased_random" => RssReport::measure(target.name(), case.name, case.ops, || {
            workload::small_biased_random(target, 0xf3ee_a110_c001_cafe, case.ops, 1024);
        }),
        "large_alloc_churn_256k" => RssReport::measure(target.name(), case.name, case.ops, || {
            workload::large_alloc_churn(target, 256 * 1024, case.ops);
        }),
        _ => unreachable!(),
    }
    .print_csv();
}
