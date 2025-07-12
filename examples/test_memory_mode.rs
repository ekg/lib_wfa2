use lib_wfa2::affine_wavefront::{AffineWavefronts, MemoryMode};

fn test_memory_mode_setting() {
    println!("=== Testing Memory Mode Setting ===\n");

    // Test 1: Default constructor
    println!("Test 1: Default constructor");
    let aligner1 = AffineWavefronts::default();
    println!("Default memory mode: {:?}", aligner1.get_memory_mode());
    println!("Note: Memory mode cannot be changed after creation");
    println!();

    // Test 2: with_penalties constructor
    println!("Test 2: with_penalties constructor (defaults to High)");
    let aligner2 = AffineWavefronts::with_penalties(0, 4, 6, 2);
    println!("Memory mode: {:?}", aligner2.get_memory_mode());
    println!();

    // Test 3: with_penalties_affine2p constructor
    println!("Test 3: with_penalties_affine2p constructor (defaults to High)");
    let aligner3 = AffineWavefronts::with_penalties_affine2p(0, 4, 6, 2, 12, 1);
    println!("Memory mode: {:?}", aligner3.get_memory_mode());
    println!();

    // Test 4: Using new constructor with memory mode
    println!("Test 4: Using new constructor with memory mode");
    let aligner4 = AffineWavefronts::with_penalties_affine2p_and_memory_mode(
        0, 4, 6, 2, 12, 1,
        MemoryMode::Ultralow
    );
    
    println!("Created with Ultralow mode:");
    println!("  Memory mode: {:?}", aligner4.get_memory_mode());
    println!("  Distance metric: {:?}", aligner4.get_distance_metric());
    
    // Test alignment
    let query = b"ACGTACGTACGT";
    let reference = b"ACGTACGTACGT";
    let status = aligner4.align(query, reference);
    
    println!("\nAlignment status: {:?}", status);
    println!("Score: {}", aligner4.score());
    println!("CIGAR: {}", String::from_utf8_lossy(aligner4.cigar()));
    
    // Verify memory mode is still Ultralow after alignment
    println!("\nMemory mode after alignment: {:?}", aligner4.get_memory_mode());
}

fn test_memory_modes_behavior() {
    println!("\n=== Testing Different Memory Modes Behavior ===\n");
    
    let test_seq1 = b"ACGTACGTACGTACGTACGTACGTACGTACGT";
    let test_seq2 = b"ACGTACGTACGTACGTACGTACGTACGTACGT";
    
    // Test each memory mode using proper constructors
    for mode in vec![MemoryMode::High, MemoryMode::Medium, MemoryMode::Low, MemoryMode::Ultralow] {
        println!("Testing with {:?} memory mode:", mode);
        
        // Create aligner with specific memory mode
        let aligner = AffineWavefronts::with_penalties_and_memory_mode(
            0, 4, 6, 2,
            mode.clone()
        );
        
        let status = aligner.align(test_seq1, test_seq2);
        
        println!("  Status: {:?}", status);
        println!("  Score: {}", aligner.score());
        println!("  CIGAR length: {}", aligner.cigar().len());
        println!("  Memory mode: {:?}", aligner.get_memory_mode());
        println!();
    }
}

fn main() {
    test_memory_mode_setting();
    test_memory_modes_behavior();
    
    println!("\n=== Summary ===");
    println!("Memory modes must be set at creation time using the appropriate constructors.");
    println!("Notice that Ultralow mode returns INT_MIN score but produces valid alignments.");
}