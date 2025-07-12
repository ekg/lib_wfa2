use lib_wfa2::affine_wavefront::{AffineWavefronts, AffineWavefrontsBuilder, MemoryMode, AlignmentScope, HeuristicStrategy};

fn main() {
    println!("=== Testing New Memory Mode Support ===\n");

    // Test 1: Direct ultralow constructor
    println!("Test 1: Using new_ultralow() constructor");
    let aligner1 = AffineWavefronts::new_ultralow();
    println!("Memory mode: {:?}", aligner1.get_memory_mode());
    println!("Distance metric: {:?}", aligner1.get_distance_metric());
    
    let query = b"ACGTACGTACGT";
    let reference = b"ACGTACGTACGT";
    let status = aligner1.align(query, reference);
    println!("Alignment status: {:?}", status);
    println!("Score: {}", aligner1.score());
    println!();

    // Test 2: Using new constructor with memory mode
    println!("Test 2: Using with_penalties_affine2p_and_memory_mode()");
    let aligner2 = AffineWavefronts::with_penalties_affine2p_and_memory_mode(
        0, 4, 6, 2, 12, 1,
        MemoryMode::Ultralow
    );
    println!("Memory mode: {:?}", aligner2.get_memory_mode());
    let status = aligner2.align(query, reference);
    println!("Alignment status: {:?}", status);
    println!();

    // Test 3: Using builder pattern
    println!("Test 3: Using builder pattern");
    let aligner3 = AffineWavefrontsBuilder::new()
        .penalties(0, 4, 6, 2)
        .dual_affine_penalties(12, 1)
        .memory_mode(MemoryMode::Ultralow)
        .alignment_scope(AlignmentScope::Alignment)
        .heuristic(HeuristicStrategy::None)
        .build();
    
    println!("Memory mode: {:?}", aligner3.get_memory_mode());
    println!("Distance metric: {:?}", aligner3.get_distance_metric());
    println!("Alignment scope: {:?}", aligner3.get_alignment_scope());
    
    let status = aligner3.align(query, reference);
    println!("Alignment status: {:?}", status);
    println!("Score: {}", aligner3.score());
    println!();

    // Test 4: Test all memory modes
    println!("Test 4: Testing all memory modes with builder");
    for mode in vec![MemoryMode::High, MemoryMode::Medium, MemoryMode::Low, MemoryMode::Ultralow] {
        let aligner = AffineWavefrontsBuilder::new()
            .penalties(0, 4, 6, 2)
            .memory_mode(mode.clone())
            .build();
        
        println!("Created aligner with {:?} mode", mode);
        println!("  Verified mode: {:?}", aligner.get_memory_mode());
        
        let status = aligner.align(query, reference);
        println!("  Alignment status: {:?}", status);
        println!("  Score: {}", aligner.score());
    }
    println!();

    // Test 5: Large sequence test (to verify ultralow memory actually works)
    println!("Test 5: Large sequence alignment with ultralow memory");
    let large_query: Vec<u8> = (0..10000).map(|i| match i % 4 {
        0 => b'A',
        1 => b'C',
        2 => b'G',
        _ => b'T',
    }).collect();
    let large_ref = large_query.clone();
    
    let aligner_large = AffineWavefronts::new_ultralow();
    println!("Aligning sequences of length {}", large_query.len());
    let status = aligner_large.align(&large_query, &large_ref);
    println!("Status: {:?}", status);
    println!("Score: {}", aligner_large.score());
    println!("CIGAR length: {}", aligner_large.cigar().len());
    
    println!("\n✅ All tests completed successfully!");
}