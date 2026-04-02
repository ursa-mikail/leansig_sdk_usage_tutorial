//! # leanSig Multi-Epoch Demo
//!
//! Demonstrates the full "sliding window" signing lifecycle that is typical
//! in an Ethereum validator:
//!
//!   • Sign messages for many consecutive epochs in order.
//!   • Call advance_preparation() proactively in the background once you've
//!     passed the midpoint of the current prepared window.
//!   • Handle the case where the prepared window hasn't caught up yet.
//!
//! Run with:
//!   cargo run --release --bin multi_epoch

use leansig::signature::{
    SignatureScheme, SignatureSchemeSecretKey,
    generalized_xmss::instantiations_poseidon::lifetime_2_to_the_18::target_sum::SIGTargetSumLifetime18W2NoOff,
};

type Sig = SIGTargetSumLifetime18W2NoOff;

fn main() {
    println!("╔══════════════════════════════════════════╗");
    println!("║    leanSig Multi-Epoch Demo  (Rust)      ║");
    println!("╚══════════════════════════════════════════╝");
    println!();

    // ── Configuration ─────────────────────────────────────────────────────
    let activation_epoch: usize = 0;
    let num_active_epochs: usize = 512;
    // Sign for just a handful of epochs so the demo finishes quickly.
    let epochs_to_sign: Vec<u32> = vec![0, 1, 2, 7, 15, 31];

    println!("  Generating key pair for {num_active_epochs} epochs…");
    let mut rng = rand::rng();
    let t0 = std::time::Instant::now();
    let (pk, mut sk) = Sig::key_gen(&mut rng, activation_epoch, num_active_epochs);
    println!("  ✓ Key gen: {:.2?}", t0.elapsed());
    println!("  Activation interval : {:?}", sk.get_activation_interval());
    println!();

    // ── Signing loop ──────────────────────────────────────────────────────
    //
    // Real-world pattern
    // ------------------
    // 1. Keep track of the current epoch counter.
    // 2. When you reach the midpoint of the prepared window, call
    //    advance_preparation() in the background (e.g., a background thread).
    // 3. Never sign the same epoch twice.
    //
    println!("── Signing across epochs ──────────────────────────────────────────");
    println!("{:<8} {:<20} {:<20} {:<8}", "Epoch", "Prepared interval", "Advances", "Valid?");
    println!("{}", "─".repeat(60));

    let mut total_advances = 0usize;

    for epoch in epochs_to_sign {
        // ── Advance preparation until this epoch is covered ──────────────
        let mut advances_this_round = 0usize;
        while !sk.get_prepared_interval().contains(&(epoch as u64)) {
            sk.advance_preparation();
            advances_this_round += 1;
            total_advances += 1;
        }

        // ── Build a message that encodes the epoch (real validators would
        //    hash a beacon block here) ──────────────────────────────────
        let mut message = [0u8; 32];
        message[..4].copy_from_slice(&epoch.to_le_bytes());
        message[4..12].copy_from_slice(b"validtr1");

        // ── Sign ─────────────────────────────────────────────────────────
        let sig = Sig::sign(&sk, epoch, &message)
            .expect("Signing failed");

        // ── Verify ───────────────────────────────────────────────────────
        let valid = Sig::verify(&pk, epoch, &message, &sig);
        let interval = sk.get_prepared_interval();

        println!(
            "{:<8} {:<20} {:<20} {}",
            epoch,
            format!("[{}, {})", interval.start, interval.end),
            format!("+{advances_this_rounds}",
                    advances_this_rounds = advances_this_round),
            if valid { "✓" } else { "✗ FAIL" }
        );

        assert!(valid, "Verification failed at epoch {epoch}");
    }

    println!("{}", "─".repeat(60));
    println!("  Total advance_preparation() calls : {total_advances}");
    println!();
    println!("  ✓ All signatures verified successfully!");
    println!();

    // ── Guidance on the background-advance pattern ─────────────────────
    println!("── Background advance pattern (pseudocode) ────────────────────────");
    println!(r#"
  // In production you'd do something like this:
  //
  //   let midpoint = (sk.get_prepared_interval().start
  //                   + sk.get_prepared_interval().end) / 2;
  //
  //   // Spawn a background task once current_epoch >= midpoint
  //   if current_epoch >= midpoint {{
  //       sk.advance_preparation();
  //   }}
  //
  // This keeps the window ahead of the current epoch without blocking.
"#);
}
