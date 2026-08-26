use swc_next_allocator::Allocator;

#[path = "../benches/support/insertion.rs"]
#[allow(dead_code)]
mod insertion_support;

fn main() {
    let count = std::env::args()
        .nth(1)
        .map(|value| value.parse().expect("insertion count must be an integer"))
        .unwrap_or(64);
    let allocator = Allocator::new();
    let mut ast = insertion_support::parse_program(&allocator);
    insertion_support::insert_program_statements(&mut ast, count);

    println!("policy={}", insertion_support::policy_label(&ast));
    println!(
        "store,live_per_chunk,holes_per_chunk,live,slots,available_holes,created,used,lookups,grows,copied_bytes"
    );
    for (store, metrics) in insertion_support::insertion_metrics(&ast) {
        println!(
            "{store:?},{},{},{},{},{},{},{},{},{},{}",
            metrics.live_per_chunk,
            metrics.holes_per_chunk,
            metrics.live,
            metrics.slots,
            metrics.available_holes,
            metrics.holes_created,
            metrics.holes_used,
            metrics.hole_lookups,
            metrics.grow_count,
            metrics.grow_copied_bytes,
        );
    }
}
