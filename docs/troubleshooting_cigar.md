# CIGAR Troubleshooting Guide for lib_wfa2

Quick reference for debugging invalid CIGAR strings when using lib_wfa2.

## Quick Checklist

When experiencing invalid CIGAR strings, check these items in order:

### 1. ✓ Parameter Order
```rust
// CORRECT - Query first, Reference second
aligner.align(query_sequence, reference_sequence);

// WRONG - Will produce invalid alignments
aligner.align(reference_sequence, query_sequence);
```

### 2. ✓ Alignment Scope
```rust
// Must set scope to get CIGAR
aligner.set_alignment_scope(AlignmentScope::Alignment);

// This will only compute score, no CIGAR!
aligner.set_alignment_scope(AlignmentScope::ComputeScore);
```

### 3. ✓ Check Alignment Status
```rust
let status = aligner.align(query, reference);
match status {
    AlignmentStatus::Completed => {
        // Safe to use CIGAR
        let cigar = aligner.cigar();
    },
    _ => {
        // CIGAR may be invalid or empty
        println!("Alignment failed with status: {:?}", status);
    }
}
```

### 4. ✓ Validate Sequences
```rust
// Ensure sequences contain only valid characters
fn is_valid_dna(seq: &[u8]) -> bool {
    seq.iter().all(|&b| matches!(b, b'A' | b'C' | b'G' | b'T' | b'N'))
}

// Convert to uppercase if needed
let query = query.to_ascii_uppercase();
let reference = reference.to_ascii_uppercase();
```

### 5. ✓ CIGAR Validation Function
```rust
fn validate_cigar_alignment(
    cigar: &[u8],
    query: &[u8],
    reference: &[u8]
) -> Result<(), String> {
    let mut q_pos = 0;
    let mut r_pos = 0;
    
    for &op in cigar {
        match op {
            b'M' | b'=' | b'X' => {
                if q_pos >= query.len() || r_pos >= reference.len() {
                    return Err(format!(
                        "CIGAR extends beyond sequences at M/=/X op: q_pos={}, r_pos={}",
                        q_pos, r_pos
                    ));
                }
                q_pos += 1;
                r_pos += 1;
            }
            b'I' => {
                if q_pos >= query.len() {
                    return Err(format!(
                        "CIGAR extends beyond query at I op: q_pos={}",
                        q_pos
                    ));
                }
                q_pos += 1;
            }
            b'D' => {
                if r_pos >= reference.len() {
                    return Err(format!(
                        "CIGAR extends beyond reference at D op: r_pos={}",
                        r_pos
                    ));
                }
                r_pos += 1;
            }
            _ => {
                return Err(format!("Invalid CIGAR operation: {}", op as char));
            }
        }
    }
    
    if q_pos != query.len() {
        return Err(format!(
            "CIGAR doesn't cover full query: {} vs {}",
            q_pos, query.len()
        ));
    }
    
    if r_pos != reference.len() {
        return Err(format!(
            "CIGAR doesn't cover full reference: {} vs {}",
            r_pos, reference.len()
        ));
    }
    
    Ok(())
}
```

## Common CIGAR Issues and Solutions

### Issue 1: CIGAR Length Doesn't Match Sequences

**Symptom**: CIGAR operations don't sum to sequence lengths

**Causes**:
- Query/reference swapped in align() call
- Partial alignment (check AlignmentStatus)
- Wrong alignment span setting

**Debug Code**:
```rust
let cigar = aligner.cigar();
let (cigar_query_len, cigar_ref_len) = count_cigar_lengths(cigar);
println!("Query length: {} vs CIGAR: {}", query.len(), cigar_query_len);
println!("Ref length: {} vs CIGAR: {}", reference.len(), cigar_ref_len);

fn count_cigar_lengths(cigar: &[u8]) -> (usize, usize) {
    let mut q_len = 0;
    let mut r_len = 0;
    for &op in cigar {
        match op {
            b'M' | b'=' | b'X' => { q_len += 1; r_len += 1; }
            b'I' => q_len += 1,
            b'D' => r_len += 1,
            _ => {}
        }
    }
    (q_len, r_len)
}
```

### Issue 2: Empty or Very Short CIGAR

**Symptom**: cigar() returns empty slice or just a few operations

**Causes**:
- Alignment scope not set to Alignment
- Sequences too dissimilar (with heuristics)
- Memory mode incompatible with sequence length

**Solution**:
```rust
// Ensure proper configuration for global alignment
let mut aligner = AffineWavefronts::default();
aligner.set_alignment_scope(AlignmentScope::Alignment);
aligner.set_heuristic(&HeuristicStrategy::None);
aligner.set_memory_mode(MemoryMode::Ultralow);
```

### Issue 3: CIGAR Contains Unexpected Characters

**Symptom**: CIGAR has characters other than M/=/X/I/D

**Causes**:
- Memory corruption
- Incorrect FFI usage
- Buffer overflow in C library

**Debug**:
```rust
let cigar = aligner.cigar();
for (i, &op) in cigar.iter().enumerate() {
    if !matches!(op, b'M' | b'=' | b'X' | b'I' | b'D') {
        println!("Invalid op at position {}: {} (0x{:02x})", 
                 i, op as char, op);
    }
}
```

### Issue 4: Alignment Position Issues

**Symptom**: CIGAR seems shifted or misaligned

**Note**: WFA2 always produces global alignments starting at position 0

**If you need local alignment positions**:
- WFA2 doesn't directly support local alignment
- You may need to add gaps to represent unaligned regions
- Consider using a different algorithm for local alignment

## Recommended Debug Workflow

1. **Start with test sequences**:
```rust
// Known good alignment
let query = b"ACGTACGT";
let ref_  = b"ACGTACGT";
// Should produce "MMMMMMMM" or "========"
```

2. **Add validation after each alignment**:
```rust
let status = aligner.align(query, reference);
if matches!(status, AlignmentStatus::Completed) {
    let cigar = aligner.cigar();
    match validate_cigar_alignment(cigar, query, reference) {
        Ok(()) => {
            // Process valid alignment
        },
        Err(e) => {
            eprintln!("CIGAR validation failed: {}", e);
            // Debug further
        }
    }
}
```

3. **Use debug builds** to catch memory issues early:
```bash
RUSTFLAGS="-C debug-assertions" cargo build
```

4. **Compare with reference implementation**:
- Use the WFA2 command-line tool to verify alignments
- Compare CIGAR outputs for the same sequences

## Configuration for Reliable Global Alignment

```rust
use lib_wfa2::affine_wavefront::{
    AffineWavefronts, AlignmentScope, MemoryMode, HeuristicStrategy
};

// Recommended configuration for global alignment with bi-WFA
pub fn create_global_aligner() -> AffineWavefronts {
    let mut aligner = AffineWavefronts::with_penalties_affine2p(
        0,   // match
        4,   // mismatch
        6,   // gap opening 1
        2,   // gap extension 1
        12,  // gap opening 2
        1    // gap extension 2
    );
    
    // Critical settings
    aligner.set_alignment_scope(AlignmentScope::Alignment);
    aligner.set_memory_mode(MemoryMode::Ultralow);
    aligner.set_heuristic(&HeuristicStrategy::None);
    
    aligner
}

// Usage
let aligner = create_global_aligner();
let status = aligner.align(query, reference);  // Query first!
if matches!(status, AlignmentStatus::Completed) {
    let cigar = aligner.cigar();
    // Validate before using...
}
```

## Still Having Issues?

If CIGARs are still invalid after checking all the above:

1. **Verify WFA2-lib version**: Ensure the submodule is up to date
2. **Check sequence encoding**: Ensure UTF-8/ASCII, no Unicode
3. **Test memory**: Run with valgrind or address sanitizer
4. **Isolate the issue**: Create minimal reproducible example
5. **Compare implementations**: Test same sequences with pywfa or WFA2 CLI