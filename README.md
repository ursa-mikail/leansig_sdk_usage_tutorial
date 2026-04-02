# leanSig SDK Demo — Rust

> Usage examples for [leanSig](https://github.com/leanEthereum/leanSig), a post-quantum
> synchronized signature scheme designed for Ethereum validators.

---

## What is leanSig?

leanSig is an XMSS-style hash-based signature scheme built from **Poseidon** tweakable
hash functions and **incomparable encodings**. It is *synchronized*: every key pair is
tied to a fixed epoch range, and each epoch may only be signed **once**. This makes it
ideal for Ethereum proof-of-stake validators.

---

## Prerequisites

| Tool  | Minimum version |
|-------|----------------|
| Rust  | **1.87**        |
| Cargo | ships with Rust |

Install Rust via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update          # ensure you are on 1.87+
rustc --version
```

---

## Quick start

```bash
git clone https://github.com/YOUR_USERNAME/leansig-demo
cd leansig-demo

# First run downloads leanSig from GitHub and compiles everything (~2–5 min).
cargo run --release --bin demo
```

> **Tip:** always use `--release`. Debug builds are 10–50× slower due to the
> heavy field arithmetic inside Poseidon.

---

## Demos

### 1. `demo` — SDK hello-world

Covers every essential step: key generation, preparation, signing, verification,
and tamper detection.

```bash
cargo run --release --bin demo
```

Expected output (times vary by hardware):

```
── Step 1: Key generation ─────────────────────────────────────────
  ✓ Done in 3.41s
── Step 2: Advancing key preparation ──────────────────────────────
  Advanced 0 time(s)
── Step 3: Signing ────────────────────────────────────────────────
  ✓ Signed in 42ms
── Step 4: Verification ───────────────────────────────────────────
  Valid : true  ✓
── Step 5: Tamper tests ───────────────────────────────────────────
  Wrong message  → valid=false  (expected: false)
  Wrong epoch    → valid=false  (expected: false)
All checks passed! 🎉
```

---

### 2. `multi_epoch` — Sliding-window lifecycle

Shows how a validator signs across many epochs using the `advance_preparation()`
sliding-window mechanic.

```bash
cargo run --release --bin multi_epoch
```

---

### 3. `bench_quick` — Instant timing comparison

Compares key-gen / sign / verify times for three scheme variants (w=1, w=2, w=4)
with a single sample each.

```bash
cargo run --release --bin bench_quick
```

For full statistically robust benchmarks, use Criterion in the main repo:

```bash
cd leanSig && cargo bench
```

```
cargo clean

That deletes the entire target/ directory. If you only want to remove the release artifacts:

cargo clean --release
```
---

## Project layout

```
leansig-demo/
├── Cargo.toml               # depends on leansig from GitHub
└── src/
    └── bin/
        ├── demo.rs          # ← start here — full SDK walkthrough
        ├── multi_epoch.rs   # signing across many epochs
        └── bench_quick.rs   # quick timing comparison
```

---

## Choosing a scheme variant

leanSig ships many concrete instantiations. The main knobs are:

| Parameter | Effect |
|-----------|--------|
| **Lifetime** (`2^18`, `2^20`, …) | Max number of signable epochs |
| **Encoding** (`target_sum`, `hypercube`) | Trade-off between sig size and signing speed |
| **Chunk size w** (`w=1`, `w=2`, `w=4`, `w=8`) | Larger `w` → smaller sig, slower verify |

The `w=2` / lifetime `2^18` target-sum variant used in the demos is a good
general-purpose starting point.

```rust
// Swap this line to try a different variant:
use leansig::signature::generalized_xmss::instantiations_poseidon
    ::lifetime_2_to_the_18::target_sum::SIGTargetSumLifetime18W4NoOff;
```

---

## Core API reference

```rust
use leansig::signature::{SignatureScheme, SignatureSchemeSecretKey};

// 1. Generate a key pair active for epochs 0..512
let (pk, mut sk) = Sig::key_gen(&mut rng, 0, 512);

// 2. Advance the internal window until epoch E is covered
while !sk.get_prepared_interval().contains(&(epoch as u64)) {
    sk.advance_preparation();
}

// 3. Sign (message must be exactly 32 bytes)
let sig = Sig::sign(&sk, epoch, &message)?;

// 4. Verify
let ok = Sig::verify(&pk, epoch, &message, &sig);
```

> ⚠️ **Never** sign the same epoch twice with the same secret key.
> The scheme is *synchronized*: doing so breaks security.

---

## License

Apache 2.0 — same as the upstream [leanSig](https://github.com/leanEthereum/leanSig) library.
