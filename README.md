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

---

# Understanding LeanSig: Post-Quantum Aggregation for Ethereum

LeanSig is a new **post-quantum signature scheme** designed specifically for Ethereum's consensus layer. The core challenge it solves is that existing quantum-resistant signatures (like XMSS) are either too large or cannot be efficiently combined (aggregated), which is a critical requirement for blockchain scalability.

## Comparison of Signature Schemes

| Scheme | Signature Size | Quantum Safe | Aggregation | Key Insight |
| :--- | :--- | :--- | :--- | :--- |
| **ECDSA** (Current) | ~64 bytes | No | No | Standard for transactions. |
| **BLS** (Current) | ~48 bytes | No | Yes (Native) | Used for consensus, but vulnerable to quantum computers. |
| **XMSS** (NIST Standard) | ~2,500 bytes | Yes | No (Native) | Quantum-safe, but signatures are large and cannot be aggregated. |
| **LeanSig** (New) | ~3,000 bytes (Proof) | Yes | Yes (Via STARKs) | Combines XMSS security with aggregation. |

## The Core Idea: XMSS + "Incomparable Encoding" + Aggregation

To understand LeanSig, it helps to break down its three main components.

### 1. The Foundation: Stateful XMSS

XMSS (eXtended Merkle Signature Scheme) is a NIST-standardized hash-based signature scheme. Unlike ECDSA or BLS, which rely on mathematical problems that quantum computers could solve, XMSS relies on the security of hash functions, making it resistant to quantum attacks.

However, a standard XMSS signature is large (around 2,500 bytes) and, crucially, **cannot be aggregated**. This means if you have 1,000 signatures, you must store and process 1,000 separate, large signatures.

### 2. The Innovation: Incomparable Encoding

This is the key technique that enables LeanSig. "Incomparable encoding" is a way of structuring the signature data so that individual signatures can be proven to be valid **without revealing the specific XMSS one-time key** that signed them. This property is essential for the next step.

### 3. The Scaling Solution: Aggregation via STARKs

Because signatures are now "incomparable," LeanSig can use a **STARK** (a type of zero-knowledge proof) to prove the validity of **many** signatures at once.

- **Without aggregation:** 1,000 signatures × ~2,500 bytes = ~2.5 MB total data
- **With LeanSig aggregation:** 1,000 signatures produce a single STARK proof of only **~3,000 bytes**, regardless of how many signatures are aggregated.

This explains the 30 MB per slot figure. An Ethereum slot might need to process hundreds of thousands of signatures. While the *aggregated* proof is small, the underlying signature data must still be collected and processed to generate that proof, leading to a large data footprint for the node generating the aggregate.

## LeanSig in Practice: Performance and Status

While the theory is promising, the practical implementation involves significant trade-offs.

### Performance Metrics

| Metric | Value |
| :--- | :--- |
| **Signature Size (Aggregated Proof)** | ~3,000 bytes |
| **Verification Time** | ~1-2 milliseconds |
| **Aggregation Data Overhead** | ~30 MB per slot (for `lifetime_2^32` configuration) |
| **Key Generation (Largest Config)** | ~15 minutes |

### Current Implementation Status

LeanSig is still in the **research and development phase**. A reference implementation exists in Rust, and there is a compatible implementation in the Zig programming language that aims for high performance through parallelism and SIMD optimizations. It is currently considered a prototype for research, not yet ready for production use.

## Summary

In short, LeanSig is an advanced construction that:

1. Uses a **stateful hash-based scheme (XMSS)** as its foundation for quantum security
2. Applies a novel **"incomparable encoding"** technique to enable aggregation
3. Achieves massive **aggregation via STARKs**, solving the scalability problem that has plagued quantum-resistant signatures on blockchains

**The trade-off:** On-chain verification becomes incredibly cheap and scalable, but the off-chain computational and data overhead for aggregators becomes significant (like the 30 MB per slot figure).

