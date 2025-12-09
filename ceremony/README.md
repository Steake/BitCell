# Ceremony Scripts

This directory contains scripts for coordinating the BitCell trusted setup ceremony.

## Scripts

- `README.md` - This file
- `participant_instructions.md` - Detailed instructions for ceremony participants
- `coordinator_checklist.md` - Checklist for ceremony coordinators

## Usage

These scripts are used during the multi-party computation (MPC) trusted setup ceremony to generate production-ready proving and verification keys for BitCell's Groth16 circuits.

For full ceremony documentation, see [../docs/CEREMONY.md](../docs/CEREMONY.md).

## Ceremony Tools (To Be Implemented)

The following tools will be implemented as part of the ceremony infrastructure:

### For Participants

1. **Contribution Tool** - Adds participant's randomness to the ceremony
   ```bash
   cargo run --release --bin ceremony-contribute -- \
     --circuit [battle|state] \
     --input <input_params.bin> \
     --output <output_params.bin> \
     --attestation <attestation.json>
   ```

2. **Verification Tool** - Verifies a contribution's integrity
   ```bash
   cargo run --release --bin ceremony-verify -- \
     --circuit [battle|state] \
     --prev <prev_params.bin> \
     --current <current_params.bin> \
     --attestation <attestation.json>
   ```

### For Coordinators

3. **Chain Verification Tool** - Verifies the entire contribution chain
   ```bash
   cargo run --release --bin ceremony-verify-chain -- \
     --circuit [battle|state] \
     --contributions keys/ceremony/<circuit>_*.bin
   ```

4. **Finalization Tool** - Generates final proving and verification keys
   ```bash
   cargo run --release --bin ceremony-finalize -- \
     --circuit [battle|state] \
     --params <final_params.bin> \
     --output-pk keys/<circuit>/proving_key.bin \
     --output-vk keys/<circuit>/verification_key.bin
   ```

## Implementation Status

⚠️ **Status**: Scripts pending implementation

The ceremony tools listed above are planned but not yet implemented. They will be developed as part of RC2 (Q1 2026) before the actual ceremony takes place.

Current status:
- [x] Documentation (CEREMONY.md)
- [x] Key management infrastructure (key_management.rs)
- [x] Key storage structure (keys/ directory)
- [ ] Ceremony contribution tool
- [ ] Ceremony verification tool
- [ ] Chain verification tool
- [ ] Finalization tool

## Security Notes

When implementing these tools:

1. **Memory Safety**: All secret values must be securely erased after use
2. **Entropy Sources**: Multiple independent sources of randomness required
3. **Verification**: Each step must be independently verifiable
4. **Logging**: Comprehensive logging for audit trail
5. **Error Handling**: Graceful handling of all error conditions

## Testing

Before the actual ceremony, all tools must be thoroughly tested:

```bash
# Run ceremony simulation with test parameters
cargo test --release ceremony_tools -- --nocapture

# Test full ceremony workflow end-to-end
./ceremony/test_ceremony_workflow.sh
```

## Contact

For questions about ceremony scripts:
- Documentation: See [CEREMONY.md](../docs/CEREMONY.md)
- Issues: https://github.com/Steake/BitCell/issues
- Email: ceremony@bitcell.org

---

**Last Updated**: December 2025  
**Status**: Documentation Complete, Tools Pending Implementation
