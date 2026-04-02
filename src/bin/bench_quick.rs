//! # leanSig Quick Timing Benchmark
//!
//! Times key-gen, sign, and verify for several concrete instantiations so you
//! can compare scheme variants at a glance without needing to run Criterion.
//!
//! Run with:
//!   cargo run --release --bin bench_quick

use std::time::Instant;

use leansig::signature::{
    SignatureScheme, SignatureSchemeSecretKey,
    // w=1 (more chains, smaller signatures)
    generalized_xmss::instantiations_poseidon::lifetime_2_to_the_18::target_sum::SIGTargetSumLifetime18W1NoOff,
    // w=2  ← sweet spot for many use cases
    generalized_xmss::instantiations_poseidon::lifetime_2_to_the_18::target_sum::SIGTargetSumLifetime18W2NoOff,
    // w=4
    generalized_xmss::instantiations_poseidon::lifetime_2_to_the_18::target_sum::SIGTargetSumLifetime18W4NoOff,
};

/// Run one benchmark for a given scheme T.
fn bench<T: SignatureScheme>(label: &str) {
    println!("  {label}");

    let mut rng = rand::rng();

    // --- Key generation ---
    // Use a small activation window so keygen completes quickly.
    let num_epochs = 256usize;
    let t = Instant::now();
    let (pk, mut sk) = T::key_gen(&mut rng, 0, num_epochs);
    let keygen_ms = t.elapsed().as_millis();

    // --- Advance preparation to epoch 0 (already prepared, but be explicit) ---
    while !sk.get_prepared_interval().contains(&0u64) {
        sk.advance_preparation();
    }

    // --- Sign ---
    let message: [u8; 32] = *b"bench message -- 32 bytes exact!";
    let epoch: u32 = 0;
    let t = Instant::now();
    let sig = T::sign(&sk, epoch, &message).expect("signing failed");
    let sign_us = t.elapsed().as_micros();

    // --- Verify ---
    let t = Instant::now();
    let valid = T::verify(&pk, epoch, &message, &sig);
    let verify_us = t.elapsed().as_micros();

    println!(
        "    key_gen : {:>6} ms   |  sign : {:>5} µs   |  verify : {:>5} µs   |  ok={}",
        keygen_ms, sign_us, verify_us, valid
    );
    assert!(valid);
}

fn main() {
    println!("╔══════════════════════════════════════════╗");
    println!("║   leanSig Quick Benchmark  (Rust)        ║");
    println!("╚══════════════════════════════════════════╝");
    println!();
    println!("  All timings use a single sample — run `cargo bench` for");
    println!("  statistically robust Criterion results.");
    println!();
    println!("── Poseidon  /  Lifetime 2^18  /  Target-Sum encoding ─────────────");

    bench::<SIGTargetSumLifetime18W1NoOff>("w=1  (155 chunks, larger sig, faster verify)");
    bench::<SIGTargetSumLifetime18W2NoOff>("w=2  (78 chunks — recommended default)");
    bench::<SIGTargetSumLifetime18W4NoOff>("w=4  (39 chunks, smaller sig, slower verify)");

    println!();
    println!("  ✓ Benchmark complete.");
    println!();
    println!("  For full Criterion benchmarks across all 40+ instantiations:");
    println!("    cd leanSig && cargo bench");
}
