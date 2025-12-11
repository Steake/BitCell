# Ceremony Coordinator Checklist

**Version:** 1.0  
**Target Date:** Q1 2026  
**Status:** Planning Phase

This checklist guides the ceremony coordinator through all phases of the trusted setup.

---

## Pre-Ceremony (4-6 weeks before)

### Week -6 to -4: Infrastructure Setup

- [ ] **Set up ceremony server infrastructure**
  - [ ] Provision secure server with TLS certificates
  - [ ] Set up upload/download endpoints
  - [ ] Configure monitoring and logging
  - [ ] Test bandwidth and reliability
  - [ ] Set up backup mirrors

- [ ] **Build ceremony tools**
  - [ ] Build `ceremony-contribute` binary for Linux/macOS/Windows
  - [ ] Build `ceremony-verify` tool
  - [ ] Build `ceremony-coordinator` tool
  - [ ] Test tools on multiple platforms
  - [ ] Create release packages

- [ ] **Prepare documentation**
  - [ ] Finalize participant instructions
  - [ ] Create FAQ document
  - [ ] Prepare announcement templates
  - [ ] Set up communication channels (Discord, Telegram, email)

- [ ] **Security audit of ceremony code**
  - [ ] Internal code review
  - [ ] External security review (if budget permits)
  - [ ] Penetration testing of server infrastructure
  - [ ] Document security measures

### Week -4 to -2: Participant Recruitment

- [ ] **Create participant list**
  - [ ] Target: 20-30 participants minimum
  - [ ] Aim for geographic diversity (5+ countries)
  - [ ] Aim for background diversity (devs, academics, enterprises)
  - [ ] Document independence verification methods

- [ ] **Outreach campaigns**
  - [ ] Blog post announcing ceremony
  - [ ] Social media announcements (Twitter, Reddit, Discord)
  - [ ] Email to BitCell community mailing list
  - [ ] Reach out to academic institutions
  - [ ] Contact blockchain security firms
  - [ ] Contact partner projects

- [ ] **Collect participant registrations**
  - [ ] Create registration form
  - [ ] Collect names/pseudonyms
  - [ ] Collect contact info (email/Telegram)
  - [ ] Collect PGP keys (optional)
  - [ ] Verify independence of participants
  - [ ] Document verification evidence

### Week -2 to 0: Pre-Ceremony Prep

- [ ] **Schedule contribution slots**
  - [ ] Assign each participant a specific time window (1-2 days)
  - [ ] Build buffer time between contributions
  - [ ] Accommodate timezone differences
  - [ ] Send calendar invites

- [ ] **Participant preparation**
  - [ ] Send participant instructions document
  - [ ] Provide download credentials
  - [ ] Conduct test runs with willing participants
  - [ ] Set up support channels
  - [ ] Answer pre-ceremony questions

- [ ] **Generate initial parameters**
  - [ ] Choose random beacon (recent Bitcoin block hash)
  - [ ] Generate initial parameters from beacon
  - [ ] Compute hash of initial parameters
  - [ ] Publish hash as public commitment
  - [ ] Announce ceremony start date

- [ ] **Final checks**
  - [ ] Test full ceremony flow end-to-end
  - [ ] Verify all tools work correctly
  - [ ] Confirm server capacity
  - [ ] Prepare incident response plan
  - [ ] Brief support team

---

## During Ceremony (2-3 weeks per circuit)

### BattleCircuit Ceremony

**Initialize Round 0:**

- [ ] Select Bitcoin block for random beacon
  - [ ] Use a future block (e.g., "block mined on ceremony start date")
  - [ ] Document block number and hash
  - [ ] Publish on website, social media, and Discord

- [ ] Generate initial parameters
  ```bash
  ./ceremony-coordinator init \
    --circuit battle \
    --beacon <bitcoin_block_hash> \
    --output params_round_0.bin
  ```

- [ ] Publish initial parameters
  - [ ] Upload to ceremony server
  - [ ] Compute and announce hash
  - [ ] Post to IPFS as backup
  - [ ] Update ceremony website

**For Each Contribution (Round 1 to N):**

- [ ] **Before participant contribution**
  - [ ] Notify participant their window is open
  - [ ] Provide download link for `params_round_X.bin`
  - [ ] Provide expected hash for verification
  - [ ] Remind them of timeline (they have 24-48 hours)

- [ ] **During participant contribution**
  - [ ] Monitor upload progress
  - [ ] Provide support via Discord/Telegram if needed
  - [ ] Be available for troubleshooting

- [ ] **After receiving contribution**
  - [ ] Download contribution file
  - [ ] Verify hash matches participant's reported hash
  - [ ] Run verification tool:
    ```bash
    ./ceremony-verify \
      --input params_round_X.bin \
      --output params_round_X+1.bin \
      --proof contribution_proof_X.json
    ```
  - [ ] If verification fails:
    - [ ] Contact participant
    - [ ] Debug issue
    - [ ] Allow re-attempt if needed
  - [ ] If verification succeeds:
    - [ ] Publish updated parameters as `params_round_X+1.bin`
    - [ ] Compute and publish hash
    - [ ] Update transcript with contribution details
    - [ ] Post announcement:
      ```
      Round X accepted
      Participant: [Name]
      Input: sha256:...
      Output: sha256:...
      Verified: ✓
      Timestamp: [UTC]
      ```
    - [ ] Thank participant publicly
    - [ ] Collect participant's attestation

- [ ] **Move to next round**
  - [ ] Notify next participant
  - [ ] Repeat process

**After All Contributions:**

- [ ] **Generate final keys**
  ```bash
  ./ceremony-coordinator finalize \
    --circuit battle \
    --input params_round_N.bin \
    --output-dir keys/battle/
  ```

- [ ] Verify final keys
  - [ ] Compute proving key hash
  - [ ] Compute verification key hash
  - [ ] Test proof generation and verification
  - [ ] Run test cases with final keys

- [ ] **Publish final keys**
  - [ ] Commit to repository: `keys/battle/`
  - [ ] Upload to IPFS
  - [ ] Create BitTorrent
  - [ ] Update website
  - [ ] Publish hashes everywhere

- [ ] **Generate and publish transcript**
  - [ ] Compile full ceremony log
  - [ ] Include all participant attestations
  - [ ] Include all verification proofs
  - [ ] Include random beacon
  - [ ] Commit to repository: `ceremony/transcripts/battle_transcript.json`

### StateCircuit Ceremony

Repeat the same process as BattleCircuit ceremony for StateCircuit.

---

## Post-Ceremony (1-2 weeks after)

### Verification and Announcement

- [ ] **Independent verification**
  - [ ] Invite external auditors to verify transcript
  - [ ] Publish verification tools and data
  - [ ] Document any verification findings
  - [ ] Address any concerns raised

- [ ] **Public announcement**
  - [ ] Blog post: "BitCell Trusted Setup Complete"
  - [ ] Social media announcement
  - [ ] Email to community
  - [ ] Press release (if applicable)
  - [ ] Update docs with "Ceremony Complete" status

- [ ] **Repository updates**
  - [ ] Tag release: `ceremony-complete-v1.0`
  - [ ] Update README with key hashes
  - [ ] Update CEREMONY.md with results
  - [ ] Archive ceremony tools

### Participant Recognition

- [ ] **Public acknowledgment**
  - [ ] Update ceremony page with all participant names
  - [ ] Create ceremony Hall of Fame page
  - [ ] Publish list of participants and their contributions
  
- [ ] **Commemorative NFTs (optional)**
  - [ ] Design commemorative NFT
  - [ ] Mint NFTs for participants
  - [ ] Distribute to participant addresses

- [ ] **Thank you communications**
  - [ ] Send personal thank you emails
  - [ ] Public shout-outs on social media
  - [ ] Feature participants in blog posts

### Documentation and Archival

- [ ] **Complete documentation**
  - [ ] Final ceremony report
  - [ ] Statistical analysis (participation rate, timing, etc.)
  - [ ] Lessons learned document
  - [ ] Update RELEASE_REQUIREMENTS.md

- [ ] **Long-term archival**
  - [ ] Archive all ceremony files
  - [ ] Multiple backup locations
  - [ ] IPFS pinning
  - [ ] Cold storage backups

- [ ] **Integration verification**
  - [ ] Test keys work in node software
  - [ ] Update CI/CD to use ceremony keys
  - [ ] Document key loading process for node operators
  - [ ] Monitor initial network usage

---

## Incident Response

### If Participant Drops Out

- [ ] Wait 24 hours past their deadline
- [ ] Attempt to contact participant
- [ ] If no response, move to next participant
- [ ] Document skip in transcript
- [ ] Continue ceremony

### If Contribution Fails Verification

- [ ] Contact participant immediately
- [ ] Debug the issue together
- [ ] Check if input parameters were correct
- [ ] Check if tools were built correctly
- [ ] Allow re-attempt with fresh parameters
- [ ] If repeated failures, may need to skip

### If Security Issue Discovered

- [ ] Pause ceremony immediately
- [ ] Assess the severity
- [ ] Notify all participants
- [ ] Fix the issue
- [ ] Determine if restart is needed
- [ ] Document the incident and resolution

### If Website/Server Goes Down

- [ ] Switch to backup mirror
- [ ] Notify participants of new URL
- [ ] Investigate root cause
- [ ] Restore primary service
- [ ] Document downtime

---

## Tools and Commands Reference

### Coordinator Tool Commands

```bash
# Initialize ceremony with random beacon
./ceremony-coordinator init \
  --circuit [battle|state] \
  --beacon <hex_string> \
  --output params_round_0.bin

# Verify a contribution
./ceremony-verify \
  --input params_round_N.bin \
  --output params_round_N+1.bin \
  --proof contribution_proof.json

# Generate final keys
./ceremony-coordinator finalize \
  --circuit [battle|state] \
  --input params_round_final.bin \
  --output-dir keys/[battle|state]/

# Generate transcript
./ceremony-coordinator transcript \
  --ceremony-dir ceremony_data/ \
  --output transcript.json
```

### Hash Computation

```bash
# Compute SHA-256 hash
sha256sum params_round_N.bin

# Verify hash matches
echo "<expected_hash> params_round_N.bin" | sha256sum -c
```

### Key Distribution

```bash
# Upload to IPFS
ipfs add -r keys/

# Create torrent
transmission-create keys/ -o bitcell-keys.torrent

# Publish hashes
echo "## Key Hashes" > KEY_HASHES.md
sha256sum keys/battle/*.bin >> KEY_HASHES.md
sha256sum keys/state/*.bin >> KEY_HASHES.md
```

---

## Contact Information

**Coordinator:** [Name/Team]  
**Email:** ceremony@bitcell.org  
**Backup:** [Alternative contact]  
**Emergency:** [Phone number for critical issues]

**Support Channels:**
- Discord: #ceremony-support
- Telegram: @BitCellCeremony
- Email: ceremony@bitcell.org

---

## Success Criteria

The ceremony is considered successful when:

- [x] At least 20 independent participants contributed
- [x] All contributions verified successfully
- [x] Final keys generated and tested
- [x] Keys published to multiple distribution channels
- [x] Full transcript published with all attestations
- [x] Independent verification completed
- [x] No security issues discovered
- [x] Community confidence in the ceremony

---

**Last Updated:** December 2025  
**Ceremony Status:** Planning Phase  
**Next Review:** Before ceremony start
