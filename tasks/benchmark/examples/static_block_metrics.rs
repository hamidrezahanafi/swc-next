use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, StorageMetrics, StoreTag};

#[path = "../benches/support/static_blocks.rs"]
#[allow(dead_code)]
mod static_blocks_support;

const STORES: [StoreTag; 9] = [
    StoreTag::Common,
    StoreTag::Expression,
    StoreTag::Statement,
    StoreTag::NamePattern,
    StoreTag::Class,
    StoreTag::Function,
    StoreTag::ClassElement,
    StoreTag::TypeJsx,
    StoreTag::ProgramAux,
];

fn print_metrics(phase: &str, ast: &Ast<'_>) {
    for store in STORES {
        let metrics = ast.storage_metrics(store);
        let density = if metrics.slots == 0 {
            1.0
        } else {
            metrics.live as f64 / metrics.slots as f64
        };
        print_row(phase, store, metrics, density);
    }
}

fn print_row(phase: &str, store: StoreTag, metrics: StorageMetrics, density: f64) {
    println!(
        "{phase},{store:?},{}+{},{},{},{:.4},{},{},{},{},{}",
        metrics.live_per_chunk,
        metrics.holes_per_chunk,
        metrics.live,
        metrics.slots,
        density,
        metrics.holes_created,
        metrics.holes_used,
        metrics.available_holes,
        metrics.grow_count,
        metrics.grow_copied_bytes,
    );
}

fn main() {
    let allocator = Allocator::new();
    let mut case =
        static_blocks_support::setup_case(&allocator, static_blocks_support::CLASS_HEAVY);
    println!(
        "phase,store,policy,live,slots,density,holes_created,holes_used,holes_remaining,grows,copied_bytes"
    );
    print_metrics("parsed", &case.ast);
    static_blocks_support::transform_case(&mut case);
    print_metrics("transformed", &case.ast);
}
