# Participant Instructions for BitCell Trusted Setup Ceremony

**Version:** 1.0  
**Date:** December 2025  
**Estimated Time:** 2-4 hours

Thank you for participating in the BitCell trusted setup ceremony! Your contribution is **critical** to the security of the BitCell blockchain.

---

## Table of Contents

1. [Before You Start](#before-you-start)
2. [Environment Setup](#environment-setup)
3. [Generating Randomness](#generating-randomness)
4. [Making Your Contribution](#making-your-contribution)
5. [Verification](#verification)
6. [Destroying Secrets](#destroying-secrets)
7. [Attestation](#attestation)
8. [Troubleshooting](#troubleshooting)

---

## Before You Start

### Understanding Your Role

As a ceremony participant, you will:
1. Download parameters from the previous participant (or initial beacon)
2. Mix in your own randomness
3. Generate updated parameters
4. Upload your contribution to the coordinator
5. **Destroy all secrets** from your machine
6. Attest that you destroyed your secrets

**Critical:** As long as you destroy your secrets, the final keys will be secure - even if all other participants are compromised!

### Prerequisites

- **Time:** Block out 2-4 hours (contribution ~30 min, but allow time for downloads/uploads)
- **Hardware:** Computer with 16GB+ RAM, 20GB+ free disk space
- **OS:** Linux, macOS, or Windows (Linux/macOS recommended)
- **Internet:** Stable connection for downloading parameters (~2-5 GB)
- **Software:** Rust toolchain (we'll install this)

### Security Recommendations

**HIGHLY RECOMMENDED:**
- ✅ Use a dedicated VM or fresh machine
- ✅ Disconnect from the internet during contribution (after downloading params)
- ✅ Use physical entropy sources (dice, coins) for randomness
- ✅ Wipe the machine completely after contribution

**ACCEPTABLE:**
- ⚠️ Use your regular machine but follow cleanup steps carefully
- ⚠️ Stay online if necessary, but be aware of attack surface

**NOT RECOMMENDED:**
- ❌ Using a shared machine where others have access
- ❌ Skipping cleanup steps
- ❌ Reusing the machine for ceremony-related work afterwards

---

## Environment Setup

### Step 1: Prepare Your Machine

If using a VM (recommended):
```bash
# Create a fresh Ubuntu VM with at least:
# - 4 CPU cores
# - 16 GB RAM
# - 30 GB disk
```

If using your regular machine:
```bash
# Create a dedicated directory
mkdir -p ~/bitcell-ceremony
cd ~/bitcell-ceremony
```

### Step 2: Install Rust

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Step 3: Clone Repository

```bash
# Clone BitCell repository
git clone https://github.com/Steake/BitCell.git
cd BitCell

# Check out ceremony branch (if applicable)
git checkout ceremony-2025

# Build ceremony tools
cd ceremony/tools
cargo build --release
```

### Step 4: Contact Coordinator

Before proceeding, contact the ceremony coordinator to:
1. Confirm your participation slot
2. Get download credentials/URLs
3. Verify the expected hash of input parameters
4. Receive any last-minute instructions

**Coordinator Contact:** [Provided separately via secure channel]

---

## Generating Randomness

High-quality randomness is **essential** for ceremony security. We use multiple sources:

### Physical Entropy Sources

**Method 1: Dice Rolls (Recommended)**

You'll need at least **100 dice rolls** (d6 is fine):

```bash
# Run the entropy collector
cargo run --release --bin entropy-collector

# It will prompt you:
> Roll a d6 and enter the result (1-6): _
```

Enter each die roll. The tool will guide you through collecting sufficient entropy.

**Method 2: Coin Flips**

Flip a coin **256 times** and record heads (H) or tails (T):

```bash
# Example:
HHTHTTHHTHHHTTHT...  (continue for 256 flips)
```

**Method 3: Keyboard Timing**

The tool can collect entropy from keyboard timing:

```bash
cargo run --release --bin entropy-collector --method keyboard

# Type random text for ~2 minutes
# The tool measures timing between keypresses
```

**Method 4: Camera/Microphone (Advanced)**

If you have a webcam or microphone:

```bash
# Capture visual noise from camera
cargo run --release --bin entropy-collector --method camera

# Or capture audio noise
cargo run --release --bin entropy-collector --method audio
```

### System Randomness

The tool will also collect from:
- `/dev/urandom` (OS entropy pool)
- CPU timing variations
- Memory allocation patterns

**Note:** Physical sources are preferred as they're harder to manipulate.

---

## Making Your Contribution

### Step 1: Download Input Parameters

The coordinator will provide a secure download link:

```bash
# Download parameters from previous round
curl -o input_params.bin https://ceremony.bitcell.org/download/round_N.bin

# Verify hash matches coordinator's announcement
sha256sum input_params.bin
# Should match: <hash provided by coordinator>
```

### Step 2: Run Contribution Tool

```bash
# Navigate to ceremony tools
cd ceremony/tools

# Run contribution (this takes 20-45 minutes)
cargo run --release --bin ceremony-contribute \
  --input ../../downloads/input_params.bin \
  --output my_contribution.bin \
  --name "Your Name or Pseudonym"
```

The tool will:
1. ✅ Load and verify input parameters
2. ✅ Collect entropy from you (dice/coins/keyboard)
3. ✅ Mix your randomness with the parameters
4. ✅ Compute updated parameters
5. ✅ Generate proof of contribution
6. ✅ Create output files

**Expected Output:**
```
[1/6] Loading input parameters...
      - Input hash: abc123...
      - Verified ✓

[2/6] Collecting entropy...
      - Roll dice and enter results
      > Roll 1: 4
      > Roll 2: 6
      ... (continue for 100 rolls)
      - Entropy collected: 256 bits ✓

[3/6] Computing contribution...
      - This may take 20-45 minutes
      - Progress: ████████░░░░ 67%

[4/6] Generating proof...
      - Proof generated ✓

[5/6] Writing output...
      - my_contribution.bin (2.3 GB)
      - my_contribution_proof.json
      - Output hash: def456...

[6/6] Done! 
      - Next: Upload my_contribution.bin to coordinator
      - Keep my_contribution_proof.json for records
```

### Step 3: Upload Your Contribution

```bash
# The coordinator will provide upload instructions
# This might be via secure SFTP, AWS S3, or other method

# Example (coordinator will provide exact command):
scp my_contribution.bin ceremony@upload.bitcell.org:/contributions/round_N/
```

**IMPORTANT:** 
- Keep `my_contribution_proof.json` - you'll need it for attestation
- Do NOT share `my_contribution.bin` with anyone except the coordinator
- The upload is large (2-5 GB), be patient

---

## Verification

### Step 1: Wait for Coordinator Verification

The coordinator will:
1. Download your contribution
2. Verify it's correctly formed
3. Verify the proof of contribution
4. Publish verification results

This typically takes 30-60 minutes.

### Step 2: Check Public Announcement

The coordinator will publish:
```
Round N Contribution Accepted
Participant: [Your Name]
Input Hash: abc123...
Output Hash: def456...
Verified: ✓
Timestamp: 2025-XX-XX HH:MM:SS UTC
```

Verify the output hash matches your local `my_contribution.bin`:
```bash
sha256sum my_contribution.bin
```

### Step 3: Verify Your Contribution

Run the verification tool yourself:
```bash
cargo run --release --bin ceremony-verify \
  --input input_params.bin \
  --output my_contribution.bin \
  --proof my_contribution_proof.json

# Should output:
# ✓ Contribution verified successfully
```

---

## Destroying Secrets

**This is the most important step!** Your contribution is only secure if you properly destroy your secrets.

### What to Destroy

All files containing:
- ❌ Input parameters (`input_params.bin`)
- ❌ Output parameters (`my_contribution.bin`)
- ❌ Any temporary files created during contribution
- ❌ Any entropy sources you collected
- ❌ Your shell history that might contain sensitive info

**Keep only:**
- ✅ `my_contribution_proof.json` (this is safe, no secrets)
- ✅ Your attestation document

### Destruction Methods

**Method 1: Secure Wipe (Linux/macOS)**

```bash
# Wipe all ceremony files
shred -vfz -n 10 input_params.bin
shred -vfz -n 10 my_contribution.bin
shred -vfz -n 10 entropy_*.bin

# Wipe the entire ceremony directory
find ~/bitcell-ceremony -type f -exec shred -vfz -n 10 {} \;

# Clear shell history
history -c
rm ~/.bash_history
```

**Method 2: Full Disk Wipe (Recommended if using VM)**

```bash
# If you used a dedicated VM, just delete it
# This ensures everything is destroyed

# Before deletion, copy out your proof and attestation:
scp my_contribution_proof.json your_machine:~/
scp attestation.txt your_machine:~/

# Then delete the VM through your hypervisor
```

**Method 3: Windows**

```powershell
# Use SDelete
sdelete -p 10 input_params.bin
sdelete -p 10 my_contribution.bin

# Or use Cipher
cipher /w:C:\bitcell-ceremony
```

### Verification

After wiping, try to recover files:
```bash
# Should find nothing
ls -la input_params.bin  # No such file
ls -la my_contribution.bin  # No such file
```

If using a VM, verify it's deleted from your hypervisor.

---

## Attestation

### Why Attest?

An attestation is your public statement that:
1. You generated genuine randomness
2. You followed the ceremony process correctly
3. **You destroyed your toxic waste**

This creates public accountability and transparency.

### Creating Your Attestation

```bash
# Use the attestation template
cp ceremony/attestation_template.txt my_attestation.txt

# Edit with your details
nano my_attestation.txt
```

**Attestation Template:**

```
BitCell Trusted Setup Ceremony Attestation

Ceremony: [BattleCircuit | StateCircuit]
Round: N
Date: YYYY-MM-DD
Participant: [Your Name or Pseudonym]

I hereby attest that:

1. I generated the contribution independently using genuine randomness.
   Entropy sources used: [e.g., "100 dice rolls with fair d6 dice"]

2. I verified the input parameters matched the published hash.
   Input hash: abc123...

3. I ran the contribution tool and verified the output.
   Output hash: def456...

4. I securely destroyed all files containing toxic waste from my contribution:
   - Input parameters: DESTROYED via [method]
   - Output parameters: DESTROYED via [method]
   - Temporary files: DESTROYED via [method]
   - [If VM] Virtual machine: DELETED from hypervisor
   - [If bare metal] Files wiped: [date/time]

5. To the best of my knowledge, no copies of the toxic waste remain.

6. I acted in good faith to support the security of BitCell.

Signature: [PGP signature or plain text if no PGP key]
Contact: [Email or other contact, optional]
Date: YYYY-MM-DD
```

### Signing Your Attestation

**With PGP (Recommended):**

```bash
# Sign with your PGP key
gpg --clearsign my_attestation.txt

# This creates my_attestation.txt.asc
# The signature proves you wrote it
```

**Without PGP:**

If you don't have a PGP key, you can still attest:

```bash
# Add identifying information
# - Your GitHub username
# - Your Twitter handle
# - Your LinkedIn profile
# - Or other verifiable identity

# This allows others to verify you're a distinct person
```

### Submitting Your Attestation

Send to the coordinator:

```bash
# Email
email my_attestation.txt.asc to ceremony@bitcell.org

# Or create a GitHub Gist (public)
# And share the link
```

The coordinator will publish all attestations in `ceremony/attestations/`.

---

## Troubleshooting

### Build Errors

**Error:** `cargo: command not found`
```bash
# Re-run Rust installer
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Error:** `linking with cc failed`
```bash
# Install build tools
# Ubuntu/Debian:
sudo apt-get install build-essential
# macOS:
xcode-select --install
```

### Memory Errors

**Error:** `Out of memory` or `SIGKILL`

- Increase VM RAM to 16GB or more
- Close other applications
- Enable swap space:
  ```bash
  sudo fallocate -l 8G /swapfile
  sudo chmod 600 /swapfile
  sudo mkswap /swapfile
  sudo swapon /swapfile
  ```

### Download Issues

**Error:** `Connection timeout` or slow download

- Use a wired connection if possible
- Download during off-peak hours
- Contact coordinator for alternative mirror

### Verification Failures

**Error:** `Contribution verification failed`

- Double-check input parameters hash
- Ensure you used the latest ceremony tools
- Contact coordinator - they may need to investigate

### Cleanup Questions

**Q:** Can I keep a copy of `my_contribution.bin` for records?

**A:** NO! This contains toxic waste. Keep only `my_contribution_proof.json`.

**Q:** What if I accidentally kept a copy?

**A:** Destroy it immediately using the secure wipe methods above. Then inform the coordinator.

**Q:** Can I re-use my machine after the ceremony?

**A:** Yes, if you've properly wiped all files. VM deletion is safer to be certain.

---

## Support

### Coordinator Contact

**Primary:** ceremony@bitcell.org  
**Backup:** [Provided during ceremony]  
**Response Time:** Usually within 4 hours

### Community Support

**Discord:** #ceremony-support channel  
**Telegram:** @BitCellCeremony  
**Forum:** https://forum.bitcell.org/c/ceremony

### FAQ

**Q:** Do I need to be a developer?

**A:** No! Anyone can participate. The tools are designed to be user-friendly.

**Q:** Can I contribute more than once?

**A:** No - we need independent participants. One contribution per person/entity.

**Q:** What if I make a mistake?

**A:** That's okay! As long as you destroy your secrets afterward, your contribution still helps. Imperfect randomness is fine.

**Q:** How long until my contribution is used?

**A:** After all participants contribute, we'll generate the final keys. This is typically 2-3 weeks after the ceremony starts.

**Q:** Is my identity public?

**A:** Your name/pseudonym and attestation are public. Your contact info is private (known only to coordinator).

---

## Thank You!

Your participation makes BitCell more secure. Every contribution adds another layer of security through decentralized trust.

After the ceremony, you'll be publicly acknowledged in:
- Ceremony transcript
- BitCell website
- Technical documentation
- (Optional) NFT commemorating participation

**We appreciate your time and commitment to building a secure blockchain ecosystem.**

---

**Questions?** Contact the coordinator or ask in community channels.

**Last Updated:** December 2025  
**Version:** 1.0  
**Ceremony Status:** Planning Phase
