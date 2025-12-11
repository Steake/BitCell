# Ceremony Attestations

This directory contains signed attestations from ceremony participants.

## Purpose

Each participant signs an attestation confirming:
1. They generated genuine randomness
2. They followed the ceremony process
3. **They destroyed their toxic waste**

These attestations create public accountability and transparency.

## Attestation Format

Each attestation is a text file (optionally PGP-signed) containing:

```
BitCell Trusted Setup Ceremony Attestation

Ceremony: [BattleCircuit | StateCircuit]
Round: N
Date: YYYY-MM-DD
Participant: [Name or Pseudonym]

I hereby attest that:

1. I generated the contribution independently using genuine randomness.
   Entropy sources used: [description]

2. I verified the input parameters matched the published hash.
   Input hash: abc123...

3. I ran the contribution tool and verified the output.
   Output hash: def456...

4. I securely destroyed all files containing toxic waste:
   [detailed destruction method]

5. To the best of my knowledge, no copies remain.

6. I acted in good faith to support BitCell security.

Signature: [PGP signature or contact info]
Date: YYYY-MM-DD
```

## Files

After the ceremony completes, this directory will contain:

- `battle_round_01_alice.txt` - Alice's attestation for BattleCircuit round 1
- `battle_round_02_bob.txt.asc` - Bob's PGP-signed attestation
- `state_round_01_alice.txt` - Alice's attestation for StateCircuit round 1
- ... (one file per participant per circuit)

## Verification

To verify a PGP-signed attestation:

```bash
# Import participant's public key (if available)
gpg --import participant_pubkey.asc

# Verify signature
gpg --verify battle_round_01_alice.txt.asc
```

For unsigned attestations, verification relies on:
- Matching to public transcript
- Cross-referencing with coordinator records
- Community recognition of participant identity

## Status

**Current Status:** Awaiting Ceremony (Q1 2026)

This directory will be populated as participants contribute to the ceremony.

---

**Last Updated:** December 2025  
**Ceremony Status:** Planning Phase
