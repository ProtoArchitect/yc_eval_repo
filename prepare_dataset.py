import numpy as np
import sys
import os

def prepare_dataset(input_file, output_file="investor_vectors.bin", dim=384, rows=1000000):
    """
    Simulates preparing an investor's custom float32 vector dataset into 
    the Q32.32 fixed-point binary format expected by Auriglyph's Zero-Allocation engine.
    """
    print(f"Generating synthetic {dim}-D float32 dataset (or load from {input_file})...")
    # In a real scenario, you would load the investor's vectors here:
    # vectors = np.load(input_file) 
    # For demo purposes, we generate random data:
    vectors_f32 = np.random.randn(rows, dim).astype(np.float32)
    
    print("Quantizing to Q32.32 fixed-point format...")
    # Multiply by 2^32 and cast to int64
    scale_factor = (1 << 32)
    vectors_q32_32 = (vectors_f32 * scale_factor).astype(np.int64)
    
    print(f"Writing binary output to {output_file}...")
    with open(output_file, 'wb') as f:
        f.write(vectors_q32_32.tobytes())
        
    size_mb = os.path.getsize(output_file) / (1024 * 1024)
    print(f"Done! Prepared {size_mb:.2f} MB of fixed-point vectors.")
    print(f"Run `cargo run --release` (point it to {output_file}) to test zero-allocation math.")

if __name__ == "__main__":
    if len(sys.argv) > 1:
        prepare_dataset(sys.argv[1])
    else:
        print("Usage: python prepare_dataset.py <input_vectors.npy>")
        print("Running with synthetic defaults to demonstrate...")
        prepare_dataset("dummy.npy")
