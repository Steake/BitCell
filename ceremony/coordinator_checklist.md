# Ceremony Coordinator Checklist

This checklist guides the ceremony coordinator through all phases of the BitCell trusted setup ceremony.

## Pre-Ceremony (Weeks -4 to -1)

### Week -4: Planning

- [ ] Set ceremony start date (target: Q1 2026)
- [ ] Define timeline (4-6 weeks for full ceremony)
- [ ] Create ceremony announcement draft
- [ ] Set up communication channels:
  - [ ] Signal group for coordinators
  - [ ] Encrypted email list
  - [ ] Discord #trusted-setup channel
- [ ] Prepare ceremony website/page
- [ ] Create ceremony GitHub project board

### Week -3: Participant Recruitment

- [ ] Publish ceremony announcement:
  - [ ] BitCell blog
  - [ ] Twitter/social media
  - [ ] Reddit (r/crypto, r/zkp)
  - [ ] Crypto forums
  - [ ] Academic mailing lists
- [ ] Set up participant application form
- [ ] Define selection criteria:
  - [ ] Geographic diversity (3+ continents)
  - [ ] Organizational diversity
  - [ ] Technical expertise
  - [ ] Community reputation
- [ ] Review applications as they arrive
- [ ] Contact potential participants

### Week -2: Participant Selection

- [ ] Select participants:
  - [ ] Minimum: 5 participants
  - [ ] Target: 10-12 participants
  - [ ] Reserve: 2-3 backup participants
- [ ] Notify selected participants
- [ ] Notify backup participants
- [ ] Publish participant list (with consent)
- [ ] Schedule participant slots (3-4 days per participant)
- [ ] Set up file transfer infrastructure:
  - [ ] SFTP server
  - [ ] Encrypted communication
  - [ ] Backup storage

### Week -1: Technical Preparation

- [ ] Finalize ceremony tools:
  - [ ] Build and test contribution tool
  - [ ] Build and test verification tool
  - [ ] Build and test chain verification
  - [ ] Build and test finalization tool
- [ ] Generate checksums for all tools
- [ ] Create ceremony commit and tag:
  - [ ] Lock code at specific commit
  - [ ] Create signed tag: `ceremony-v1.0`
- [ ] Test full ceremony workflow end-to-end
- [ ] Prepare initial parameters:
  - [ ] Generate `battle_params_0.bin`
  - [ ] Generate `state_params_0.bin`
  - [ ] Compute and publish checksums
- [ ] Distribute materials to participants:
  - [ ] Participant instructions
  - [ ] Software checksums
  - [ ] Coordinator contact info
  - [ ] Security guidelines
- [ ] Set up monitoring:
  - [ ] Progress tracking spreadsheet
  - [ ] Automated hash verification
  - [ ] Backup systems

## Ceremony Phase (Weeks 1-4)

### Daily Coordinator Tasks

For each participant handoff:

- [ ] Verify previous participant completed:
  - [ ] Output files received
  - [ ] Attestation signed and valid
  - [ ] Hashes match expected values
  - [ ] Public announcement made
- [ ] Prepare handoff package for next participant:
  - [ ] Previous participant's output files
  - [ ] Current checksums
  - [ ] Instructions reminder
  - [ ] Support contact
- [ ] Send handoff notification to next participant
- [ ] Update public progress tracker:
  - [ ] Participant N completed
  - [ ] Current phase: Participant N+1
  - [ ] Expected completion: [date]
- [ ] Archive attestations:
  - [ ] Save signed attestations
  - [ ] Verify GPG signatures
  - [ ] Commit to ceremony repository
- [ ] Backup all files:
  - [ ] Parameters files
  - [ ] Attestations
  - [ ] Logs

### Weekly Coordinator Tasks

- [ ] Post weekly progress update:
  - [ ] Blog post
  - [ ] Social media update
  - [ ] Discord announcement
- [ ] Verify contribution chain:
  - [ ] Run chain verification tool
  - [ ] Check all hashes in sequence
  - [ ] Verify no breaks in chain
- [ ] Monitor for issues:
  - [ ] Check participant progress
  - [ ] Respond to questions
  - [ ] Troubleshoot problems
- [ ] Coordinate with backup participants if needed:
  - [ ] Participant dropout
  - [ ] Technical difficulties
  - [ ] Schedule delays

### Per-Participant Checklist

When Participant N starts:

1. **Initial Contact**
   - [ ] Confirm participant ready to start
   - [ ] Verify hardware meets requirements
   - [ ] Verify software built correctly
   - [ ] Send input files
   - [ ] Provide checksums for verification
   - [ ] Confirm time window (24-48 hours)

2. **During Contribution**
   - [ ] Check in after 4 hours
   - [ ] Verify progress being made
   - [ ] Be available for questions
   - [ ] Monitor for technical issues

3. **After Contribution**
   - [ ] Receive output files
   - [ ] Verify output checksums
   - [ ] Verify attestation format
   - [ ] Verify GPG signature
   - [ ] Run verification tool
   - [ ] Confirm toxic waste destroyed
   - [ ] Request public announcement
   - [ ] Add attestation to repository

4. **Handoff to Next Participant**
   - [ ] Package output files
   - [ ] Compute new checksums
   - [ ] Contact Participant N+1
   - [ ] Transfer files securely
   - [ ] Update progress tracker
   - [ ] Post update on social media

### Handling Issues

**Participant Dropout**:
- [ ] Contact backup participant immediately
- [ ] Brief backup on current state
- [ ] Adjust timeline if needed
- [ ] Update public communications

**Technical Failure**:
- [ ] Diagnose issue (hardware/software/network)
- [ ] Attempt recovery if possible
- [ ] Revert to previous valid state if needed
- [ ] Re-run contribution on different hardware

**Suspicious Activity**:
- [ ] Document suspicious behavior
- [ ] Verify contribution independently
- [ ] Consult with security advisors
- [ ] Consider excluding contribution if invalid
- [ ] Communicate transparently with community

## Post-Ceremony (Week 5-6)

### Week 5: Finalization

- [ ] Receive final participant's contribution
- [ ] Verify complete contribution chain:
  - [ ] Run chain verification on all contributions
  - [ ] Verify all attestations present and valid
  - [ ] Check no gaps in sequence
  - [ ] Compute final contribution hash
- [ ] Generate final keys:
  - [ ] Run finalization tool for BattleCircuit
  - [ ] Run finalization tool for StateCircuit
  - [ ] Verify key generation successful
  - [ ] Compute key checksums
- [ ] Test final keys:
  - [ ] Generate test proof with BattleCircuit keys
  - [ ] Generate test proof with StateCircuit keys
  - [ ] Verify proofs validate correctly
  - [ ] Run full integration tests
- [ ] Prepare keys for distribution:
  - [ ] Package proving keys
  - [ ] Package verification keys
  - [ ] Create metadata files
  - [ ] Compute all checksums
  - [ ] Sign with coordinator GPG key

### Week 5-6: Verification and Publication

- [ ] Independent verification:
  - [ ] Contact independent verifiers
  - [ ] Provide contribution chain
  - [ ] Provide final keys
  - [ ] Receive verification reports
  - [ ] Address any concerns raised
- [ ] Prepare ceremony report:
  - [ ] Summary of process
  - [ ] List of all participants
  - [ ] Timeline of events
  - [ ] Technical details
  - [ ] Security analysis
  - [ ] Verification results
  - [ ] Lessons learned
- [ ] Publish final keys:
  - [ ] **GitHub**: Commit to `keys/` directory
    - [ ] Add metadata.json files
    - [ ] Update README.md
    - [ ] Create release with checksums
  - [ ] **IPFS**: Upload and get CIDs
    - [ ] Pin to multiple nodes
    - [ ] Update metadata with CIDs
  - [ ] **Arweave**: Permanent storage
    - [ ] Upload proving keys
    - [ ] Upload verification keys
    - [ ] Record transaction IDs
  - [ ] **Torrents**: Create and seed
    - [ ] Create .torrent files
    - [ ] Set up seeders
    - [ ] Publish magnet links
- [ ] Update documentation:
  - [ ] Update keys/README.md with hashes
  - [ ] Update docs/CEREMONY.md with results
  - [ ] Add ceremony report to docs/
  - [ ] Update installation instructions
- [ ] Public announcement:
  - [ ] Blog post with full report
  - [ ] Social media announcement
  - [ ] Press release (if appropriate)
  - [ ] Community forum post
  - [ ] Email to participants

### Week 6: Post-Ceremony Tasks

- [ ] Archive ceremony artifacts:
  - [ ] All parameter files
  - [ ] All attestations
  - [ ] All logs
  - [ ] Communication records
  - [ ] Create ceremony archive
  - [ ] Upload to permanent storage
- [ ] Thank participants:
  - [ ] Individual thank you emails
  - [ ] Public acknowledgment
  - [ ] Ceremony commemorative NFT/badge (optional)
- [ ] Integration with BitCell:
  - [ ] Update bitcell-zkp to use ceremony keys
  - [ ] Update tests to verify against ceremony keys
  - [ ] Deploy to testnet
  - [ ] Verify all systems work
- [ ] Monitor adoption:
  - [ ] Track key downloads
  - [ ] Monitor for issues
  - [ ] Respond to questions
  - [ ] Provide support
- [ ] Schedule retrospective:
  - [ ] Team debrief
  - [ ] Document learnings
  - [ ] Update ceremony process for future
  - [ ] Share insights with community

## Emergency Procedures

### Critical Issues During Ceremony

**File Corruption Detected**:
1. Stop ceremony immediately
2. Identify point of corruption
3. Revert to last valid state
4. Re-run affected contributions
5. Add additional verification checks

**Security Breach Suspected**:
1. Pause ceremony
2. Investigate thoroughly
3. Consult security advisors
4. Determine if breach occurred
5. If compromised: restart ceremony
6. If safe: resume with additional security

**Coordinator Incapacitation**:
1. Backup coordinator takes over
2. Review current state
3. Contact all participants
4. Continue ceremony if safe
5. Document any changes

## Success Criteria

The ceremony is considered successful when:

- [ ] Minimum 5 participants completed contributions
- [ ] All attestations published and verified
- [ ] Complete contribution chain verified
- [ ] Final keys generated successfully
- [ ] Independent verification completed
- [ ] Keys published to all distribution channels
- [ ] Test proofs generated and verified
- [ ] Ceremony report published
- [ ] No security issues identified
- [ ] Community accepts ceremony as valid

## Ceremony Metrics

Track these metrics throughout:

- **Participants**:
  - Total: ___
  - Geographic diversity: ___
  - Organizational diversity: ___
  
- **Timeline**:
  - Start date: ___
  - End date: ___
  - Total duration: ___ days
  - Average time per participant: ___ hours

- **Technical**:
  - BattleCircuit key size: ___ MB
  - StateCircuit key size: ___ MB
  - Total data transferred: ___ GB
  - Contribution verification time: ___ hours

- **Security**:
  - Attestations published: ___/___
  - Independent verifications: ___
  - Issues encountered: ___
  - Issues resolved: ___

## Post-Ceremony Follow-Up

### 1 Month After

- [ ] Review ceremony report reception
- [ ] Address any community concerns
- [ ] Monitor key distribution metrics
- [ ] Provide support for key users
- [ ] Document any issues found

### 3 Months After

- [ ] Evaluate ceremony process
- [ ] Update ceremony documentation
- [ ] Plan improvements for future ceremonies
- [ ] Consider publishing academic paper
- [ ] Share findings with ZK community

## Contact Information

**Lead Coordinator**:
- Name: ___
- Email: ceremony@bitcell.org
- Phone: ___
- Signal: ___
- GPG Key: ___

**Backup Coordinator**:
- Name: ___
- Email: ___
- Phone: ___

**Security Advisor**:
- Name: ___
- Email: ___

**Technical Lead**:
- Name: ___
- Email: ___

## Resources

- Ceremony Documentation: [../docs/CEREMONY.md](../docs/CEREMONY.md)
- Participant Instructions: [participant_instructions.md](participant_instructions.md)
- GitHub Project: https://github.com/Steake/BitCell/projects/ceremony
- Discord: #trusted-setup
- Email: ceremony@bitcell.org

---

**Version**: 1.0  
**Created**: December 2025  
**For Use**: BitCell Trusted Setup Ceremony Q1 2026
