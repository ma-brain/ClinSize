fn main() {
    println!(
        "clinsize-core engine version: {}",
        clinsize_core::engine_version()
    );
    println!(
        "registered methods: {}",
        clinsize_core::registry::list_methods().len()
    );
}
