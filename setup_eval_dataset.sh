#!/bin/bash
echo ">>> [AURIGLYPH] Generating 4GB Synthetic Semantic Codebook for Local Evaluation..."
echo ">>> This simulates a massively bloated LLM KV-Cache state."
dd if=/dev/urandom of=dummy_dataset.bin bs=1M count=4000 status=progress
echo ">>> Done. Dataset ready at ./dummy_dataset.bin"
