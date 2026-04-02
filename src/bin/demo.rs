//! # leanSig SDK Demo
//!
//! This binary walks through every step of using leanSig as a library:
//!   1. Key generation
//!   2. Advancing key preparation (the "sliding window" mechanic)
//!   3. Signing a 32-byte message at a chosen epoch
//!   4. Verifying the signature
//!   5. Showing what happens with an invalid / tampered signature
//!
//! Run with:
//!   cargo run --release --bin demo

use leansig::signature::{
    SignatureScheme, SignatureSchemeSecretKey,
    // Concrete instantiation: Poseidon hash, 2^18 key lifetime, target-sum encoding, w=2
    generalized_xmss::instantiations_poseidon::lifetime_2_to_the_18::target_sum::SIGTargetSumLifetime18W2NoOff,
};

/// Convenience alias so the rest of the code doesn't repeat the long path.
type Sig = SIGTargetSumLifetime18W2NoOff;

fn main() {
    println!("╔══════════════════════════════════════════╗");
    println!("║       leanSig SDK Demo  (Rust)           ║");
    println!("╚══════════════════════════════════════════╝");
    println!();

    // ─── Parameters ──────────────────────────────────────────────────────────
    // leanSig keys are "synchronized": they are valid for a fixed interval of
    // epochs.  Pick a small window here so the demo stays fast.
    let activation_epoch: usize = 0;
    let num_active_epochs: usize = 512; // must be ≤ Sig::LIFETIME
    let target_epoch: u32 = 7;         // the epoch we want to sign at

    println!("── Parameters ─────────────────────────────────────────────────────");
    println!("  Scheme lifetime  : 2^{} = {} epochs", 18, Sig::LIFETIME);
    println!("  Activation epoch : {activation_epoch}");
    println!("  Active epochs    : {num_active_epochs}");
    println!("  Signing epoch    : {target_epoch}");
    println!();

    // ─── Step 1: Key generation ───────────────────────────────────────────────
    println!("── Step 1: Key generation ─────────────────────────────────────────");
    println!("  Generating key pair…  (this may take a few seconds)");

    let mut rng = rand::rng();
    let start = std::time::Instant::now();
    let (pk, mut sk) = Sig::key_gen(&mut rng, activation_epoch, num_active_epochs);
    println!("  ✓ Done in {:.2?}", start.elapsed());
    println!("  Activation interval : {:?}", sk.get_activation_interval());
    println!("  Prepared interval   : {:?}", sk.get_prepared_interval());
    println!();

    // ─── Step 2: Advance key preparation ─────────────────────────────────────
    // The secret key is internally structured as a "sliding window" Merkle tree.
    // Before signing at epoch E, the key must be prepared for E.
    // Call advance_preparation() to move the window forward.
    println!("── Step 2: Advancing key preparation ─────────────────────────────");
    let mut advances = 0usize;
    while !sk.get_prepared_interval().contains(&(target_epoch as u64)) {
        sk.advance_preparation();
        advances += 1;
    }
    println!("  Advanced {advances} time(s)");
    println!("  Prepared interval now : {:?}", sk.get_prepared_interval());
    println!("  Epoch {target_epoch} is prepared : ✓");
    println!();

    // ─── Step 3: Sign a message ───────────────────────────────────────────────
    // Messages must be exactly 32 bytes (leansig::MESSAGE_LENGTH).
    println!("── Step 3: Signing ────────────────────────────────────────────────");
    let message: [u8; 32] = *b"hello leanSig -- 32 byte message";
    println!("  Message (raw) : {:?}", std::str::from_utf8(&message).unwrap());

    let start = std::time::Instant::now();
    let signature = Sig::sign(&sk, target_epoch, &message)
        .expect("Signing failed — check epoch is in prepared interval");
    println!("  ✓ Signed in {:.2?}", start.elapsed());
    println!();

    // ─── Step 4: Verify the signature ────────────────────────────────────────
    println!("── Step 4: Verification ───────────────────────────────────────────");
    let start = std::time::Instant::now();
    let valid = Sig::verify(&pk, target_epoch, &message, &signature);
    println!("  Verify time  : {:.2?}", start.elapsed());
    println!("  Valid        : {valid}");
    assert!(valid, "Verification should succeed for a correctly generated signature");
    println!("  ✓ Signature is VALID");
    println!();

    // ─── Step 5: Tampered message / wrong epoch ───────────────────────────────
    println!("── Step 5: Tamper tests ───────────────────────────────────────────");

    // Wrong message
    let bad_message: [u8; 32] = *b"tampered msg -- 32 bytes exactly";
    let still_valid = Sig::verify(&pk, target_epoch, &bad_message, &signature);
    println!("  Wrong message  → valid={still_valid}  (expected: false)");
    assert!(!still_valid);

    // Wrong epoch
    let wrong_epoch: u32 = target_epoch + 1;
    let still_valid2 = Sig::verify(&pk, wrong_epoch, &message, &signature);
    println!("  Wrong epoch    → valid={still_valid2}  (expected: false)");
    assert!(!still_valid2);

    println!();
    println!("══════════════════════════════════════════════════════════════════");
    println!("  All checks passed! 🎉");
    println!("══════════════════════════════════════════════════════════════════");
    println!();
    println!("Next steps:");
    println!("  cargo run --release --bin multi_epoch   # sign across multiple epochs");
    println!("  cargo run --release --bin bench_quick   # quick timing comparison");
}
