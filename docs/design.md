# lib_wfa2 Design and Implementation Details

This document provides technical details about the lib_wfa2 Rust wrapper implementation, focusing on the FFI interface, memory management, and CIGAR generation process.

## Architecture Overview

```
┌─────────────────┐
│   Rust API      │  (affine_wavefront.rs)
├─────────────────┤
│   FFI Bindings  │  (bindings_wfa.rs)
├─────────────────┤
│   C Library     │  (WFA2-lib)
└─────────────────┘
```

## FFI Interface Details

### Key Data Structures

#### 1. Wavefront Aligner Structure

The core aligner is represented as an opaque pointer in Rust:

```rust
pub struct AffineWavefronts {
    wf_aligner: *mut wfa::wavefront_aligner_t,
}
```

The underlying C structure contains:
- Alignment attributes and configuration
- Penalty scores
- Memory management structures
- CIGAR buffer
- Wavefront components

#### 2. CIGAR Representation

The CIGAR is stored internally in the C structure as:

```c
typedef struct {
    char* operations;      // Array of CIGAR operations
    int begin_offset;      // Start offset in operations array
    int end_offset;        // End offset in operations array
    int score;             // Alignment score
} cigar_t;
```

In Rust, we access this through unsafe pointer operations:

```rust
pub fn cigar(&self) -> &[u8] {
    unsafe {
        let cigar = (*self.wf_aligner).cigar;
        let ops = (*cigar).operations;
        let begin_offset = (*cigar).begin_offset;
        let end_offset = (*cigar).end_offset;
        let length = end_offset - begin_offset;
        
        std::slice::from_raw_parts(
            (ops as *const u8).add(begin_offset as usize),
            length as usize,
        )
    }
}
```

### Memory Safety Considerations

1. **Lifetime Management**: The `AffineWavefronts` struct owns the C aligner and deallocates it on drop
2. **Sequence Borrowing**: The `align` function borrows sequences, which are only valid during the alignment call
3. **CIGAR Access**: The CIGAR slice is tied to the aligner's lifetime

## Alignment Process Flow

### 1. Initialization

```rust
// Default initialization
let aligner = AffineWavefronts::default();
// Creates wavefront_aligner_new(NULL) with default attributes

// Custom initialization with two-piece affine
let aligner = AffineWavefronts::with_penalties_affine2p(...);
// Creates custom attributes struct and passes to wavefront_aligner_new()
```

### 2. Configuration

The aligner can be configured through various setter methods:
- `set_penalties()`: Updates penalty scores
- `set_heuristic()`: Configures alignment heuristics
- `set_alignment_scope()`: Score-only vs full alignment
- `set_memory_mode()`: Memory usage profile

### 3. Alignment Execution

```rust
pub fn align(&self, pattern: &[u8], text: &[u8]) -> AlignmentStatus {
    unsafe {
        // Convert &[u8] to *const i8 for C compatibility
        let pattern_ptr = pattern.as_ptr() as *const i8;
        let text_ptr = text.as_ptr() as *const i8;
        
        // Call C function
        let status = wfa::wavefront_align(
            self.wf_aligner,
            pattern_ptr,
            pattern.len() as i32,
            text_ptr,
            text.len() as i32,
        );
        
        // Convert status to Rust enum
        AlignmentStatus::from(status)
    }
}
```

### 4. Result Extraction

After alignment, results are extracted through unsafe operations:
- Score: Direct field access
- CIGAR: Slice construction from internal buffer

## CIGAR Generation Details

### Internal CIGAR Format

WFA2 generates CIGAR in a compact format during traceback:
1. Each cell in the DP matrix stores the operation that led to it
2. Traceback follows the optimal path, recording operations
3. Operations are stored as single characters in the operations buffer

### CIGAR Buffer Management

The CIGAR buffer uses offset-based management:
- `begin_offset`: Start of valid CIGAR data
- `end_offset`: End of valid CIGAR data
- This allows reuse of the buffer without reallocation

### Operation Encoding

| Internal Code | Character | Meaning |
|--------------|-----------|---------|
| 0 | 'M' or '=' | Match |
| 1 | 'X' | Mismatch |
| 2 | 'I' | Insertion |
| 3 | 'D' | Deletion |

## Critical Implementation Details

### 1. Parameter Order in C API

The WFA2 C API uses a specific parameter order that differs from some conventions:

```c
int wavefront_align(
    wavefront_aligner_t* const wf_aligner,
    const char* const pattern,    // Query sequence
    const int pattern_length,
    const char* const text,        // Reference sequence
    const int text_length
);
```

**Note**: Despite the naming, `pattern` is the query and `text` is the reference.

### 2. Two-Piece Affine Configuration

When using two-piece affine penalties, the attributes must be properly configured:

```rust
unsafe {
    let mut attributes = wfa::wavefront_aligner_attr_default;
    attributes.distance_metric = wfa::distance_metric_t_gap_affine_2p;
    attributes.affine2p_penalties.match_ = 0;
    attributes.affine2p_penalties.mismatch = mismatch;
    attributes.affine2p_penalties.gap_opening1 = gap_opening1;
    attributes.affine2p_penalties.gap_extension1 = gap_extension1;
    attributes.affine2p_penalties.gap_opening2 = gap_opening2;
    attributes.affine2p_penalties.gap_extension2 = gap_extension2;
}
```

### 3. Memory Modes and Bi-WFA

The memory mode affects the algorithm used:
- `MemoryHigh`: Standard WFA with full DP matrix
- `MemoryMedium`: Reduced memory with some pruning
- `MemoryLow`: Aggressive pruning
- `MemoryUltralow`: Bi-directional WFA (bi-WFA)

Bi-WFA works by:
1. Running forward and reverse wavefronts
2. Meeting in the middle
3. Reconstructing the full alignment path

### 4. Alignment Scope Impact

The alignment scope affects what data is computed:
- `ComputeScore`: Only calculates the alignment score
- `Alignment`: Computes score and generates CIGAR

**Important**: If scope is `ComputeScore`, the CIGAR will be empty or invalid.

## Debugging CIGAR Issues

### Common CIGAR Problems and Solutions

1. **Empty CIGAR**
   - Check alignment scope is set to `Alignment`
   - Verify alignment completed successfully
   - Check sequence lengths are > 0

2. **CIGAR Length Mismatch**
   - Verify query/reference order is correct
   - Check for integer overflow in length calculations
   - Ensure sequences don't contain null bytes

3. **Invalid Operations**
   - Verify sequences contain only valid DNA characters
   - Check memory corruption hasn't occurred
   - Ensure proper null-termination handling

### CIGAR Validation

```rust
fn validate_cigar(cigar: &[u8], query_len: usize, ref_len: usize) -> bool {
    let mut query_pos = 0;
    let mut ref_pos = 0;
    
    for &op in cigar {
        match op {
            b'M' | b'=' | b'X' => {
                query_pos += 1;
                ref_pos += 1;
            }
            b'I' => query_pos += 1,
            b'D' => ref_pos += 1,
            _ => return false, // Invalid operation
        }
    }
    
    query_pos == query_len && ref_pos == ref_len
}
```

### Debug Helper Functions

```rust
impl AffineWavefronts {
    #[cfg(debug_assertions)]
    pub fn debug_state(&self) {
        unsafe {
            let aligner = &*self.wf_aligner;
            println!("Aligner state:");
            println!("  Scope: {:?}", self.get_alignment_scope());
            println!("  Memory mode: {:?}", self.get_memory_mode());
            println!("  Distance metric: {:?}", self.get_distance_metric());
            
            if !aligner.cigar.is_null() {
                let cigar = &*aligner.cigar;
                println!("  CIGAR begin: {}", cigar.begin_offset);
                println!("  CIGAR end: {}", cigar.end_offset);
                println!("  Score: {}", cigar.score);
            }
        }
    }
}
```

## Performance Considerations

### 1. Memory Allocation

- The aligner reuses internal buffers between alignments
- CIGAR buffer grows as needed but doesn't shrink
- Thread-local allocators can improve performance

### 2. Sequence Preparation

- Sequences should be uppercase
- Invalid characters should be replaced with 'N'
- Consider pre-validation for better error messages

### 3. Heuristic Selection

- No heuristics: Exact but potentially slow
- Banded: Good for similar sequences
- Adaptive: Balances speed and accuracy
- WFMash: Optimized for whole-genome alignment

## Future Improvements

1. **Safe CIGAR Access**: Implement a CIGAR iterator that ensures memory safety
2. **Builder Pattern**: Add a builder for complex aligner configurations
3. **Parallel Alignment**: Support for batch alignment operations
4. **Custom Allocators**: Allow pluggable memory allocators
5. **Extended CIGAR**: Support for additional operations (e.g., soft clipping)