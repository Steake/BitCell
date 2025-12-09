# Ceremony Participant Instructions

Thank you for participating in the BitCell trusted setup ceremony! Your contribution is crucial to the security of the BitCell network.

## Before You Start

### Prerequisites

- [ ] You have received confirmation of your participation slot
- [ ] You have access to a suitable machine (see hardware requirements)
- [ ] You have reviewed the [ceremony documentation](../docs/CEREMONY.md)
- [ ] You understand the security implications of the ceremony

### Hardware Requirements

Your machine must have:
- **CPU**: 8+ cores (16+ cores recommended)
- **RAM**: 64GB minimum (128GB recommended)
- **Storage**: 100GB+ free space
- **Network**: Reliable internet connection for file transfers
- **Time**: 4-8 hours for your contribution

### Software Setup

1. **Install Rust** (version 1.70 or later):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup update stable
   ```

2. **Clone BitCell repository**:
   ```bash
   git clone https://github.com/Steake/BitCell.git
   cd BitCell
   git checkout <ceremony-commit>  # Use specific commit provided by coordinator
   ```

3. **Build ceremony tools**:
   ```bash
   cargo build --release --package bitcell-ceremony
   ```

4. **Verify build**:
   ```bash
   sha256sum target/release/ceremony-contribute
   # Compare with checksum provided by coordinator
   ```

## Contribution Process

### Step 1: Receive Previous Contribution

You will receive files from the previous participant (or initial parameters if you're first):

For BattleCircuit:
- `battle_params_<N-1>.bin` (where N is your participant number)
- `battle_attestation_<N-1>.json.asc` (GPG-signed attestation)

For StateCircuit:
- `state_params_<N-1>.bin`
- `state_attestation_<N-1>.json.asc`

**Verify the files**:
```bash
# Verify GPG signatures
gpg --verify battle_attestation_<N-1>.json.asc
gpg --verify state_attestation_<N-1>.json.asc

# Verify SHA256 checksums (provided separately by coordinator)
sha256sum battle_params_<N-1>.bin
sha256sum state_params_<N-1>.bin
```

### Step 2: Prepare Randomness Sources

**CRITICAL**: You must use multiple independent sources of randomness.

#### Required Sources:

1. **System Entropy**:
   - Linux: `/dev/urandom`
   - Windows: `CryptGenRandom`
   - macOS: `/dev/random`

2. **Hardware RNG** (if available):
   - Check: `ls /dev/hwrng`
   - Intel: RDRAND instruction
   - ARM: TrustZone RNG

3. **Physical Randomness**:
   - Roll dice (20+ rolls, record results)
   - Flip coins (100+ flips, record results)
   - Shuffle deck of cards (record order)

4. **Environmental Noise**:
   - Record microphone input (ambient room noise)
   - Take photo with camera (sensor noise)
   - Mouse movements (timing variation)

5. **User Input**:
   - Random keyboard typing
   - Timestamp of keystrokes

#### Document Your Sources:

Create a file `entropy_sources.txt`:
```
System: /dev/urandom (Linux 5.15)
Hardware: Intel RDRAND (verified working)
Physical: 
  - Dice rolls: [3,1,6,2,5,4,...]
  - Coin flips: [H,T,H,H,T,...]
Environmental:
  - Microphone: 10 seconds @ 44.1kHz
  - Camera: 5 photos, 12MP each
User Input:
  - Typing: 500+ random keystrokes
  - Mouse: 1000+ movement samples
```

### Step 3: Make Your Contribution

Run the contribution tool for each circuit:

```bash
# BattleCircuit contribution
cargo run --release --bin ceremony-contribute -- \
  --circuit battle \
  --input keys/ceremony/battle_params_<N-1>.bin \
  --output keys/ceremony/battle_params_<N>.bin \
  --attestation keys/ceremony/battle_attestation_<N>.json \
  --entropy-sources entropy_sources.txt

# StateCircuit contribution
cargo run --release --bin ceremony-contribute -- \
  --circuit state \
  --input keys/ceremony/state_params_<N-1>.bin \
  --output keys/ceremony/state_params_<N>.bin \
  --attestation keys/ceremony/state_attestation_<N>.json \
  --entropy-sources entropy_sources.txt
```

**During contribution**:
- The tool will prompt you for additional entropy
- Enter random text when prompted
- Move your mouse randomly if requested
- The process will take 2-4 hours per circuit

**Monitor progress**:
- Watch for any errors or warnings
- Note the progress percentage
- Ensure sufficient disk space

### Step 4: Verify Your Contribution

After contribution completes, verify it was successful:

```bash
# Verify BattleCircuit contribution
cargo run --release --bin ceremony-verify -- \
  --circuit battle \
  --prev keys/ceremony/battle_params_<N-1>.bin \
  --current keys/ceremony/battle_params_<N>.bin \
  --attestation keys/ceremony/battle_attestation_<N>.json

# Verify StateCircuit contribution
cargo run --release --bin ceremony-verify -- \
  --circuit state \
  --prev keys/ceremony/state_params_<N-1>.bin \
  --current keys/ceremony/state_params_<N>.bin \
  --attestation keys/ceremony/state_attestation_<N>.json
```

**Expected output**:
```
✓ Previous contribution valid
✓ Contribution properly applied
✓ Output hash matches attestation
✓ All verification checks passed
```

### Step 5: Sign Your Attestation

Sign your attestation with GPG/PGP:

```bash
# Generate GPG key if you don't have one
gpg --full-generate-key

# Sign attestations
gpg --clearsign keys/ceremony/battle_attestation_<N>.json
gpg --clearsign keys/ceremony/state_attestation_<N>.json
```

**Publish your public key**:
```bash
# Export public key
gpg --armor --export your-email@example.com > pubkey.asc

# Upload to keyserver
gpg --keyserver keyserver.ubuntu.com --send-keys <your-key-id>
```

### Step 6: Destroy Toxic Waste

**CRITICAL SECURITY STEP**

You must securely erase all secret values used in your contribution:

1. **Overwrite random data files**:
   ```bash
   shred -vfz -n 10 entropy_sources.txt
   shred -vfz -n 10 dice_rolls.txt
   shred -vfz -n 10 mic_recording.wav
   # ... repeat for all entropy files
   ```

2. **Clear build artifacts**:
   ```bash
   cargo clean
   shred -vfz -n 10 target/release/ceremony-contribute
   ```

3. **Clear system memory** (Linux):
   ```bash
   sudo sync
   echo 3 | sudo tee /proc/sys/vm/drop_caches
   ```

4. **Reboot your machine**:
   ```bash
   sudo reboot
   ```

**Optional but recommended**:
- Use a dedicated machine for the ceremony
- Consider using a live USB OS that doesn't persist data
- Physically destroy storage media after ceremony (extreme security)

### Step 7: Transmit to Next Participant

Send your output files to the next participant:

Files to send:
- `battle_params_<N>.bin`
- `battle_attestation_<N>.json.asc`
- `state_params_<N>.bin`
- `state_attestation_<N>.json.asc`
- `pubkey.asc` (your GPG public key)

**Transfer methods** (in order of preference):
1. **Encrypted transfer**: Use GPG-encrypted email or Signal
2. **SFTP/SCP**: Secure file transfer to coordinator's server
3. **Physical media**: USB drive via courier (for maximum security)

**Notify coordinator**:
```
Subject: Ceremony Contribution Complete - Participant <N>

I have completed my contribution to the BitCell trusted setup ceremony.

Circuits: BattleCircuit, StateCircuit
Participant ID: <N>
Output hashes:
  BattleCircuit: sha256:<hash>
  StateCircuit: sha256:<hash>

Files transmitted to: [next participant name/email]
Method: [transfer method used]

I confirm that I have:
✓ Used multiple independent sources of randomness
✓ Verified my contribution integrity
✓ Signed my attestation with GPG
✓ Securely destroyed all secret values
✓ Transmitted output to next participant

GPG Key Fingerprint: <fingerprint>
```

### Step 8: Publish Your Attestation

Post your signed attestation publicly:

1. **GitHub**: Create a Gist with your attestation
2. **Twitter**: Tweet your attestation hash
3. **Blog/Website**: Publish full attestation if you have one
4. **Forum**: Post to BitCell community forum

**Example tweet**:
```
I just completed my contribution to the @BitCell trusted setup ceremony!

Attestation hash: sha256:abc123...
GPG fingerprint: ABCD 1234 ...

Full attestation: [link to gist]

#BitCell #TrustedSetup #ZKP
```

## Troubleshooting

### Build Errors

**Problem**: Compilation fails
```
Solution:
1. Update Rust: rustup update stable
2. Clean build: cargo clean
3. Check disk space: df -h
4. Check Rust version: rustc --version (need 1.70+)
```

### Contribution Errors

**Problem**: Out of memory during contribution
```
Solution:
1. Close all other applications
2. Increase swap space
3. Use a machine with more RAM
4. Contact coordinator for help
```

**Problem**: Contribution takes too long (>8 hours)
```
Solution:
1. Verify CPU is not throttling (check temperatures)
2. Ensure no other processes consuming CPU
3. Check if using test parameters (should be production size)
```

### Verification Fails

**Problem**: Verification reports errors
```
DO NOT PROCEED. Contact coordinator immediately with:
- Error message
- Log files
- SHA256 hash of input and output files
```

### File Transfer Issues

**Problem**: Files too large for email
```
Solutions:
1. Use file sharing service (Dropbox, Google Drive)
2. Use SFTP to coordinator's server
3. Split files: split -b 100M file.bin file.bin.part
```

## Security Reminders

### DO:
- ✓ Use multiple independent entropy sources
- ✓ Verify all inputs before processing
- ✓ Sign your attestation with GPG
- ✓ Destroy toxic waste thoroughly
- ✓ Publish your attestation publicly
- ✓ Keep your participation confidential until ceremony completes

### DO NOT:
- ✗ Reuse random values from other sources
- ✗ Skip verification steps
- ✗ Share intermediate files publicly
- ✗ Retain any secret values after contribution
- ✗ Run ceremony on shared/cloud infrastructure
- ✗ Multitask during contribution (avoid contamination)

## Post-Ceremony

After the ceremony completes:

1. **Verify final keys**: Check that your contribution is in the chain
2. **Review ceremony report**: Confirm all participants listed
3. **Test final keys**: Try generating and verifying a proof
4. **Celebrate**: You helped secure the BitCell network! 🎉

## Questions?

- **Documentation**: [CEREMONY.md](../docs/CEREMONY.md)
- **Coordinator Email**: ceremony@bitcell.org
- **Emergency Contact**: [coordinator phone]
- **Discord**: #trusted-setup channel

## Acknowledgment

By participating, you agree to:
- Follow these instructions carefully
- Use multiple entropy sources honestly
- Destroy toxic waste completely
- Publish your attestation publicly
- Not collude with other participants

Your honest participation is crucial to the security of BitCell. Thank you!

---

**Version**: 1.0  
**Last Updated**: December 2025  
**Valid For**: BitCell Trusted Setup Ceremony Q1 2026
