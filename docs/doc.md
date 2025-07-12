# lib_wfa2 Rust Wrapper Documentation

This document provides comprehensive guidance on using the lib_wfa2 Rust wrapper for the WFA2-lib (Wavefront Alignment Algorithm) library, with a focus on global alignment using bi-WFA (bidirectional wavefront) with two-piece affine gap scoring.

## Table of Contents
1. [Overview](#overview)
2. [Key Concepts](#key-concepts)
3. [API Usage](#api-usage)
4. [CIGAR Interpretation](#cigar-interpretation)
5. [Common Pitfalls](#common-pitfalls)
6. [Best Practices](#best-practices)
7. [Troubleshooting](#troubleshooting)

## Overview

The lib_wfa2 Rust wrapper provides safe bindings to the WFA2-lib C library, which implements the wavefront alignment algorithm for fast and memory-efficient sequence alignment. The library supports various gap penalty models including:
- Simple gap-affine penalties
- Dual-cost gap-affine penalties (two-piece affine)

## Key Concepts

### 1. Terminology Mapping

**CRITICAL**: The terminology between the Rust wrapper and the underlying C library can be confusing:

| Rust Wrapper | C Library (WFA2) | Biological Meaning |
|--------------|------------------|-------------------|
| `pattern`    | `pattern`        | Query sequence    |
| `text`       | `text`           | Reference/Target sequence |

**In the `align()` function**: The first parameter is the **query** (pattern), and the second parameter is the **reference** (text).

```rust
// Correct usage:
aligner.align(query_sequence, reference_sequence);
```

### 2. Coordinate System

- All positions are **0-based**
- Alignment positions refer to the start of the alignment in both sequences
- The CIGAR string describes operations from the beginning of the alignment

### 3. Scoring System

- WFA uses **edit distance** scoring (lower scores are better)
- Match score is typically 0
- Mismatches and gaps incur positive penalties
- The final score represents the total cost of the alignment

## API Usage

### Basic Global Alignment

```rust
use lib_wfa2::affine_wavefront::AffineWavefronts;

// Create aligner with default parameters
let aligner = AffineWavefronts::default();

// Sequences to align
let query = b"ACGTACGTACGT";      // First parameter (pattern)
let reference = b"ACGTACGTACGT";   // Second parameter (text)

// Perform alignment
let status = aligner.align(query, reference);

// Get results
let score = aligner.score();
let cigar = aligner.cigar();
```

### Two-Piece Affine Gap Penalties

For ultra-low memory bi-WFA with dual-cost gap penalties:

```rust
use lib_wfa2::affine_wavefront::{AffineWavefronts, MemoryMode, AlignmentScope};

// Create aligner with two-piece affine penalties
let mut aligner = AffineWavefronts::with_penalties_affine2p(
    0,   // match score (always 0)
    4,   // mismatch penalty
    6,   // gap opening penalty 1 (short gaps)
    2,   // gap extension penalty 1
    12,  // gap opening penalty 2 (long gaps)
    1    // gap extension penalty 2
);

// Set ultra-low memory mode for bi-WFA
aligner.set_memory_mode(MemoryMode::Ultralow);

// Ensure we compute full alignment (not just score)
aligner.set_alignment_scope(AlignmentScope::Alignment);

// Perform alignment
let query = b"ACGTACGTACGT";
let reference = b"ACGTACGTACGT";
let status = aligner.align(query, reference);
```

### Disabling Heuristics

For exact global alignment without heuristics:

```rust
use lib_wfa2::affine_wavefront::{AffineWavefronts, HeuristicStrategy};

let mut aligner = AffineWavefronts::default();

// Disable all heuristics for exact alignment
aligner.set_heuristic(&HeuristicStrategy::None);
```

## CIGAR Interpretation

### CIGAR Format

The WFA2 library returns CIGAR strings in a **compact character format** where each character represents a single operation:

| Character | Operation | Meaning |
|-----------|-----------|---------|
| `M` or `=` | Match | Bases are identical |
| `X` | Mismatch | Bases differ |
| `I` | Insertion | Base in query but not in reference |
| `D` | Deletion | Base in reference but not in query |

### CIGAR Direction and Position

**CRITICAL**: The CIGAR string describes how to transform the **reference** into the **query**:
- Start at position 0 in both sequences
- Apply operations left-to-right
- `I` means insert a base from the query
- `D` means delete a base from the reference

### Example CIGAR Interpretation

```rust
let query =     b"ACGTACGT";
let reference = b"ACGTTCGT";
//                    ^
//                 Mismatch at position 4

// CIGAR: "MMMMMMMM" or "====X===" (depending on match representation)
```

### Converting to Standard CIGAR

To convert the compact format to standard run-length encoded CIGAR:

```rust
fn compact_to_standard_cigar(cigar: &[u8]) -> String {
    let mut result = String::new();
    let mut current_op = None;
    let mut count = 0;
    
    for &op in cigar {
        if current_op == Some(op) {
            count += 1;
        } else {
            if let Some(prev_op) = current_op {
                result.push_str(&format!("{}{}", count, prev_op as char));
            }
            current_op = Some(op);
            count = 1;
        }
    }
    
    if let Some(op) = current_op {
        result.push_str(&format!("{}{}", count, op as char));
    }
    
    result
}
```

## Common Pitfalls

### 1. Query/Reference Order Confusion

**Problem**: Swapping query and reference sequences leads to invalid alignments.

**Solution**: Always remember:
```rust
aligner.align(query, reference);  // query first, reference second
```

### 2. Forgetting Alignment Scope

**Problem**: Getting only scores without CIGAR strings.

**Solution**: Set alignment scope explicitly:
```rust
aligner.set_alignment_scope(AlignmentScope::Alignment);
```

### 3. Memory Mode for Large Sequences

**Problem**: Running out of memory on large sequences.

**Solution**: Use ultra-low memory mode:
```rust
aligner.set_memory_mode(MemoryMode::Ultralow);
```

### 4. CIGAR Interpretation Errors

**Problem**: Misinterpreting CIGAR operations relative to sequences.

**Solution**: Remember that CIGAR describes transforming reference → query.

### 5. Incorrect Penalty Settings

**Problem**: Using incorrect penalty values leading to suboptimal alignments.

**Solution**: Use biologically meaningful penalties:
- Mismatch: 4-6
- Gap opening: 6-12
- Gap extension: 1-2

## Best Practices

### 1. For Global Alignment with Bi-WFA

```rust
// Recommended setup for global alignment
let mut aligner = AffineWavefronts::with_penalties_affine2p(
    0,   // match
    4,   // mismatch
    6,   // gap open 1
    2,   // gap extend 1
    12,  // gap open 2
    1    // gap extend 2
);

aligner.set_memory_mode(MemoryMode::Ultralow);
aligner.set_alignment_scope(AlignmentScope::Alignment);
aligner.set_heuristic(&HeuristicStrategy::None);
```

### 2. Sequence Validation

Always validate sequences before alignment:

```rust
fn validate_sequence(seq: &[u8]) -> bool {
    seq.iter().all(|&b| matches!(b, b'A' | b'C' | b'G' | b'T' | b'N'))
}
```

### 3. Error Handling

Check alignment status:

```rust
use lib_wfa2::affine_wavefront::AlignmentStatus;

match aligner.align(query, reference) {
    AlignmentStatus::Completed => {
        // Process successful alignment
    },
    AlignmentStatus::MaxStepsReached => {
        // Alignment too complex
    },
    AlignmentStatus::OOM => {
        // Out of memory
    },
    _ => {
        // Other errors
    }
}
```

## Troubleshooting

### Invalid CIGAR Strings

**Symptoms**: CIGAR doesn't match sequence lengths or contains unexpected operations.

**Possible Causes**:
1. Query/reference order swapped
2. Sequences contain invalid characters
3. Alignment scope not set to `Alignment`
4. Memory corruption (check sequence lengths)

**Debugging Steps**:
1. Verify sequence contents and lengths
2. Check parameter order in `align()` call
3. Ensure alignment scope is set correctly
4. Try with simple test sequences first

### Unexpected Alignment Scores

**Symptoms**: Scores don't match expected values.

**Possible Causes**:
1. Incorrect penalty settings
2. Heuristics affecting alignment
3. Wrong distance metric

**Debugging Steps**:
1. Disable heuristics with `HeuristicStrategy::None`
2. Verify penalty values
3. Check distance metric matches penalty type

### Performance Issues

**Symptoms**: Slow alignment or high memory usage.

**Solutions**:
1. Use `MemoryMode::Ultralow` for large sequences
2. Enable appropriate heuristics for approximate alignment
3. Consider banded alignment for similar sequences
4. Split very long sequences into overlapping chunks

## References

- [WFA2-lib GitHub Repository](https://github.com/smarco/WFA2-lib)
- [Wavefront Alignment Algorithm Paper](https://doi.org/10.1093/bioinformatics/btaa777)
- [lib_wfa2 Repository](https://github.com/AndreaGuarracino/lib_wfa2)