use swc_next_allocator::Allocator;
use swc_next_ecma_codegen::{Options, print};

#[path = "../benches/support/static_blocks.rs"]
#[allow(dead_code)]
mod static_blocks_support;

fn main() {
    let allocator = Allocator::new();
    let mut case =
        static_blocks_support::setup_case(&allocator, static_blocks_support::CLASS_HEAVY);
    static_blocks_support::transform_case(&mut case);
    println!(
        "identifiers={}",
        static_blocks_support::count_identifier_references(&case.ast)
    );
    print!("{}", print(&allocator, &case.ast, Options::default()).code);
}
