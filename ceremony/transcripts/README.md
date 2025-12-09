# Ceremony Transcripts

This directory contains complete transcripts of BitCell trusted setup ceremonies.

## Purpose

The ceremony transcript provides a complete, verifiable record of:
- The random beacon used to initialize the ceremony
- Every participant contribution
- All verification proofs
- Final key hashes
- Timestamps and participant information

This enables anyone to independently verify the ceremony was conducted correctly.

## Transcript Format

Each ceremony produces a JSON transcript:

```json
{
  "ceremony_id": "bitcell-battle-circuit-2025",
  "circuit": "BattleCircuit",
  "circuit_constraints": 6700000,
  "start_time": "2025-XX-XX HH:MM:SS UTC",
  "end_time": "2025-XX-XX HH:MM:SS UTC",
  "duration_days": 21,
  
  "random_beacon": {
    "source": "Bitcoin",
    "block_number": 850000,
    "block_hash": "000000000000000000012345...",
    "timestamp": "2025-XX-XX HH:MM:SS UTC"
  },
  
  "contributions": [
    {
      "round": 1,
      "participant": {
        "name": "Alice",
        "contact": "alice@example.com",
        "pgp_fingerprint": "ABCD 1234 5678...",
        "location": "United States",
        "affiliation": "Independent Developer"
      },
      "timestamp": "2025-XX-XX HH:MM:SS UTC",
      "input_hash": "sha256:abc123...",
      "output_hash": "sha256:def456...",
      "contribution_proof": {
        "challenge": "...",
        "response": "..."
      },
      "attestation_file": "attestations/battle_round_01_alice.txt",
      "verified": true
    },
    {
      "round": 2,
      "participant": {
        "name": "Bob",
        "contact": "bob@university.edu",
        "pgp_fingerprint": "EFGH 5678 9012...",
        "location": "Germany",
        "affiliation": "Academic Researcher"
      },
      "timestamp": "2025-XX-XX HH:MM:SS UTC",
      "input_hash": "sha256:def456...",
      "output_hash": "sha256:ghi789...",
      "contribution_proof": {
        "challenge": "...",
        "response": "..."
      },
      "attestation_file": "attestations/battle_round_02_bob.txt",
      "verified": true
    }
    // ... more contributions
  ],
  
  "statistics": {
    "total_participants": 25,
    "total_rounds": 25,
    "average_contribution_time_hours": 1.5,
    "countries_represented": 8,
    "independent_participants": 25
  },
  
  "final_keys": {
    "proving_key": {
      "file": "keys/battle/proving_key.bin",
      "sha256": "abc123...",
      "size_bytes": 2147483648
    },
    "verification_key": {
      "file": "keys/battle/verification_key.bin",
      "sha256": "def456...",
      "size_bytes": 1024
    }
  },
  
  "verification": {
    "all_contributions_verified": true,
    "key_derivation_verified": true,
    "independent_auditors": [
      {
        "name": "Audit Firm XYZ",
        "date": "2025-XX-XX",
        "report": "audits/xyz_report.pdf"
      }
    ]
  }
}
```

## Files

After ceremonies complete, this directory will contain:

- `battle_transcript.json` - Full transcript for BattleCircuit ceremony
- `state_transcript.json` - Full transcript for StateCircuit ceremony
- `README.md` - This file

## Verification

To verify a ceremony using the transcript:

```bash
# Using ceremony verification tool
cd ceremony/tools
cargo run --release --bin ceremony-verify \
  --transcript ../transcripts/battle_transcript.json \
  --keys ../../keys/battle/

# Manual verification
# 1. Verify random beacon (check Bitcoin blockchain)
# 2. Verify each contribution proof
# 3. Verify final keys match the chain of contributions
# 4. Cross-reference with attestations
```

### Verification Checklist

- [ ] Random beacon is valid and unpredictable
- [ ] Random beacon was announced before ceremony started
- [ ] Each contribution has valid proof
- [ ] Input/output hashes form a chain
- [ ] All participants have attestations
- [ ] Participants are independent (different people/organizations)
- [ ] Final keys hash matches transcript
- [ ] Keys can generate and verify proofs

## Distribution

Transcripts are distributed via:
- GitHub repository (primary)
- IPFS (content-addressed backup)
- BitTorrent (decentralized distribution)
- Official website

## Status

**Current Status:** Awaiting Ceremony (Q1 2026)

Transcripts will be published immediately after ceremony completion.

Expected file sizes:
- `battle_transcript.json` - ~500 KB (for 25 participants)
- `state_transcript.json` - ~500 KB (for 25 participants)

## Support

**Questions about transcripts?**
- Email: ceremony@bitcell.org
- Discord: #ceremony-verification
- Documentation: [`../../docs/CEREMONY.md`](../../docs/CEREMONY.md)

---

**Last Updated:** December 2025  
**Ceremony Status:** Planning Phase
