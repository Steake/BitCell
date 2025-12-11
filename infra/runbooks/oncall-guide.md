# BitCell On-Call Rotation Guide

## Overview

This guide outlines the on-call rotation process, responsibilities, and best practices for BitCell production infrastructure support.

---

## On-Call Schedule

### Rotation Structure

**Primary On-Call:**
- Duration: 1 week (Monday 9:00 AM to Monday 9:00 AM)
- Responsibilities: First responder to all incidents
- Expected response time: See incident severity levels
- Coverage: 24/7

**Secondary On-Call:**
- Duration: 1 week (same schedule as primary)
- Responsibilities: Backup for escalations, support for complex issues
- Expected response time: Within 30 minutes of escalation
- Coverage: 24/7

**Tertiary (Leadership):**
- Platform Lead and Engineering Manager
- Escalation point for P0 incidents
- Strategic decision making

### Schedule Management

**Tool:** PagerDuty (https://bitcell.pagerduty.com)

**Calendar:**
- View current schedule: `/schedules/production`
- Request swap: `/overrides/create`
- Update availability: `/user/settings`

**Swapping Shifts:**
1. Find coverage in team Slack channel (#oncall-swaps)
2. Agree on swap dates
3. Create override in PagerDuty
4. Confirm with team lead
5. Update team calendar

---

## Pre-On-Call Preparation

### Before Your Shift Starts

**48 Hours Before:**
- [ ] Review previous week's incidents
- [ ] Check system status and ongoing issues
- [ ] Verify access to all systems
- [ ] Update contact information
- [ ] Review any scheduled maintenance

**24 Hours Before:**
- [ ] Test PagerDuty notifications
- [ ] Ensure laptop and phone are charged
- [ ] Review runbooks for common issues
- [ ] Join #oncall-primary Slack channel
- [ ] Check internet connectivity backup (mobile hotspot)

**Handoff Meeting (Monday 9:00 AM):**
- Attend 15-minute handoff meeting
- Review previous week:
  - Incidents handled
  - Ongoing issues
  - Upcoming changes
  - Known problems
- Ask questions
- Acknowledge in PagerDuty

### Required Access

**Verify you have access to:**
- [ ] PagerDuty (alert receiving)
- [ ] Grafana (monitoring dashboard)
- [ ] Prometheus (metrics)
- [ ] Alertmanager (alert management)
- [ ] Docker/Kubernetes (infrastructure)
- [ ] GitHub (code repository)
- [ ] Slack (communication)
- [ ] VPN (if applicable)
- [ ] Production servers (SSH/kubectl)
- [ ] Cloud provider console (AWS/GCP/Azure)

---

## During Your Shift

### Daily Routine

**Morning Check (9:00 AM):**
```bash
# Run daily health check script
./scripts/daily-health-check.sh

# Review overnight alerts
# Check Alertmanager: http://localhost:9093

# Review metrics dashboard
# Check Grafana: http://localhost:3000
```

**Items to check:**
- [ ] All nodes are operational
- [ ] Chain is progressing normally
- [ ] No unacknowledged alerts
- [ ] System resources are within normal ranges
- [ ] No performance degradation

**Evening Check (6:00 PM):**
- Quick review of metrics
- Acknowledge any minor alerts
- Update team on any ongoing issues

### Alert Handling

**When Alert Fires:**

1. **Acknowledge** alert in PagerDuty (within SLA)
2. **Assess** severity using runbook
3. **Investigate** using diagnostic steps
4. **Act** to resolve or mitigate
5. **Communicate** status updates
6. **Document** actions taken
7. **Escalate** if needed

**Communication Protocol:**

**For P0/P1 incidents:**
- Post in #incidents Slack channel
- Create incident thread
- Update every 30 minutes minimum
- Tag relevant team members
- Update status page

**Example status update:**
```
[UPDATE] 10:15 AM - Node Down Incident
Status: Investigating
Impact: Single node in US-East region down
Actions Taken:
- Reviewed logs, found OOM error
- Restarting node with increased memory
Next Update: 10:30 AM or when resolved
```

### Incident Management

**Step-by-Step Process:**

1. **Triage (0-5 minutes)**
   - Determine severity
   - Assess impact
   - Check if known issue
   - Decide if war room needed

2. **Investigation (5-30 minutes)**
   - Follow runbook procedures
   - Check monitoring dashboards
   - Review logs
   - Identify root cause

3. **Mitigation (10-60 minutes)**
   - Apply fix or workaround
   - Monitor for recovery
   - Verify service restoration
   - Document temporary measures

4. **Resolution (varies)**
   - Confirm permanent fix
   - Monitor for recurrence
   - Update documentation
   - Schedule follow-up if needed

5. **Post-Incident (within 48 hours)**
   - Write incident report
   - Identify action items
   - Update runbooks
   - Share learnings with team

---

## Incident Response

### Severity Levels & Response Times

| Severity | Response Time | Examples | Actions |
|----------|---------------|----------|---------|
| **P0 - Critical** | <15 minutes | Complete outage, data loss, security breach | Page entire team, start war room, notify leadership |
| **P1 - High** | <30 minutes | Regional outage, >30% nodes down, chain stopped | Acknowledge, investigate, escalate if not resolved in 1 hour |
| **P2 - Medium** | <2 hours | Single node down, high latency, minor degradation | Acknowledge, investigate during business hours |
| **P3 - Low** | Next business day | Warnings, non-critical issues | Document in backlog, address in maintenance window |

### Escalation Criteria

**Escalate to Secondary On-Call when:**
- Unable to resolve P1 within 1 hour
- Need second opinion on resolution approach
- Issue requires specialized knowledge
- Need assistance during complex recovery
- Multiple simultaneous incidents

**Escalate to Leadership when:**
- P0 incident occurs
- Security incident detected
- Need authorization for major changes
- Require external communication approval
- Issue may affect SLAs/customers significantly

### War Room Protocol

**For P0/P1 incidents lasting >1 hour:**

**Setup:**
1. Create Zoom/Slack huddle
2. Designate incident commander (usually primary on-call)
3. Assign roles:
   - Commander: Coordinates response
   - Investigator: Diagnoses issue
   - Communicator: Provides updates
   - Scribe: Documents timeline

**During War Room:**
- 10-minute update cycles
- Clear action items with owners
- Document all decisions
- Keep communication channel updated
- Coordinate with other teams if needed

---

## Handoff Procedures

### End of Shift Handoff

**Monday 9:00 AM Meeting:**

**Outgoing On-Call Prepares:**
- Summary document of week's incidents
- List of ongoing issues
- Pending action items
- Known upcoming events/changes
- Tips and observations

**Handoff Meeting Agenda (15 minutes):**
1. Review incident summary (5 min)
2. Discuss ongoing issues (5 min)
3. Highlight upcoming events (2 min)
4. Q&A (3 min)

**After Handoff:**
- Outgoing: Update PagerDuty schedule
- Incoming: Acknowledge in PagerDuty
- Both: Update #oncall-handoff Slack thread

**Handoff Template:**

```
## On-Call Handoff: [Date Range]

### Incidents Summary
- Total incidents: X
- P0: X | P1: X | P2: X | P3: X
- Notable incidents:
  1. [Description, resolution, follow-up needed]
  2. [Description, resolution, follow-up needed]

### Ongoing Issues
- [Issue 1]: Status, next steps, owner
- [Issue 2]: Status, next steps, owner

### Upcoming Events
- [Date]: Planned maintenance window
- [Date]: Expected traffic increase
- [Date]: Deployment scheduled

### Notes & Tips
- [Any observations or tips for next on-call]
- [Known workarounds or quirks]

### Action Items
- [ ] [Task 1] - Owner: @person - Due: Date
- [ ] [Task 2] - Owner: @person - Due: Date
```

---

## Best Practices

### Do's

✅ **Do** acknowledge alerts promptly
✅ **Do** document all actions taken
✅ **Do** communicate proactively
✅ **Do** escalate when unsure
✅ **Do** ask questions
✅ **Do** update runbooks when you learn something
✅ **Do** test fixes in staging first (when possible)
✅ **Do** take breaks during long incidents
✅ **Do** write good incident reports
✅ **Do** share learnings with team

### Don'ts

❌ **Don't** ignore alerts hoping they resolve
❌ **Don't** make changes without documentation
❌ **Don't** hesitate to escalate
❌ **Don't** test fixes directly in production (when avoidable)
❌ **Don't** forget to update status
❌ **Don't** work alone on critical issues
❌ **Don't** skip post-incident reviews
❌ **Don't** leave shift without handoff

### Communication Tips

**Be Clear:**
```
❌ "There's an issue with the thing"
✅ "Node us-east-1 is down due to OOM error. Restarting now."
```

**Be Timely:**
```
❌ (2 hours later) "Oh yeah, that's fixed now"
✅ (Every 30 min) "Update: Still investigating. Trying approach X."
```

**Be Specific:**
```
❌ "Nodes having problems"
✅ "3 out of 7 nodes (us-east-1, us-east-2, us-west-1) unresponsive"
```

---

## Tools & Resources

### Monitoring & Alerting

**Grafana Dashboards:**
- Production Overview: http://localhost:3000/d/production
- Node Details: http://localhost:3000/d/node-details
- Network Health: http://localhost:3000/d/network

**Prometheus:**
- Metrics: http://localhost:9999
- Alerts: http://localhost:9999/alerts

**Alertmanager:**
- Dashboard: http://localhost:9093
- Silence alerts: http://localhost:9093/#/silences

### Commands Cheat Sheet

```bash
# Check all nodes status
docker-compose ps

# View logs
docker-compose logs -f node-us-east-1

# Restart node
docker-compose restart node-us-east-1

# Check metrics
curl http://localhost:9090/metrics | grep bitcell

# Check health
curl http://localhost:9090/health

# Run chaos test
python3 infra/chaos/chaos_test.py --scenario node_failure

# Database backup
./scripts/backup-database.sh
```

### Runbooks

- [Incident Response](./incident-response.md)
- [Deployment Guide](./deployment-guide.md)
- [Common Issues](./incident-response.md#common-incidents)

### Contacts

**Team Slack Channels:**
- #oncall-primary - Current on-call engineer
- #oncall-swaps - Request shift swaps
- #incidents - Active incident coordination
- #oncall-handoff - Weekly handoff summaries
- #platform-team - General team channel

**Escalation:**
- Secondary On-Call: See PagerDuty
- Platform Lead: @platform-lead
- Engineering Manager: @eng-manager
- Security Team: security@bitcell.network

---

## Post-Incident Activities

### Required Documentation

**For P0/P1 Incidents:**

Within 48 hours, create incident report including:

1. **Summary**: What happened, impact, duration
2. **Timeline**: Detailed sequence of events
3. **Root Cause**: Why it happened
4. **Resolution**: How it was fixed
5. **Impact**: Users/services affected, downtime
6. **Action Items**: Preventive measures
7. **Learnings**: What we learned

**Template:** [incident-report-template.md](./incident-report-template.md)

### Blameless Post-Mortems

**Principles:**
- Focus on systems, not people
- Assume good intentions
- Seek to understand, not judge
- Identify systemic improvements
- Share learnings widely

**Meeting Format:**
- 60 minutes
- All relevant parties invited
- Incident commander presents
- Open discussion
- Action items assigned
- Follow-up scheduled

---

## Self-Care

### Managing Stress

**During Long Incidents:**
- Take regular breaks (every 2 hours)
- Stay hydrated
- Eat proper meals
- Ask for help when needed
- Rotate roles if possible

**After Difficult Shifts:**
- Debrief with team
- Take comp time if needed
- Don't carry stress to next week
- Speak up if feeling burned out

**Work-Life Balance:**
- Set boundaries when off-call
- Keep devices accessible but not obsessive
- Plan activities considering on-call duties
- Communicate availability clearly

### Compensation

**On-Call Pay:**
- On-call stipend per week
- Incident response compensation
- Comp time for after-hours work
- Weekend/holiday multipliers

**Time Off:**
- Extra PTO for on-call rotation
- Flexibility during on-call week
- Mandatory breaks between rotations

---

## Continuous Improvement

### Feedback

**After Each Shift:**
- What went well?
- What could be improved?
- What tools/documentation would help?
- Submit feedback to team lead

**Quarterly Review:**
- Review incident trends
- Update runbooks
- Improve alerting
- Optimize response procedures

### Training

**Recommended Learning:**
- Shadow experienced on-call
- Practice with chaos tests
- Review past incident reports
- Attend platform architecture sessions
- Learn infrastructure components deeply

---

## FAQ

**Q: What if I miss an alert while sleeping?**
A: Escalation automatically goes to secondary after 15 minutes. Document missed alert and learn from it.

**Q: Can I work on other tasks while on-call?**
A: Yes, but maintain ability to respond quickly. Avoid deep focus work that can't be interrupted.

**Q: What if I'm not confident handling an issue?**
A: Escalate immediately. Better to get help early than make wrong changes.

**Q: How do I handle multiple simultaneous incidents?**
A: Prioritize by severity, call in secondary for help, delegate if possible.

**Q: Can I go on vacation during my on-call week?**
A: Yes, but arrange swap in advance. Minimum 2 weeks notice required.

**Q: What if I need to escalate at 3 AM?**
A: Do it. That's what the rotation is for. Better safe than sorry.

---

## Appendix

### Example Incident Log

```
2024-12-09 14:23:15 - Alert received: NodeDown (us-east-1)
2024-12-09 14:23:45 - Acknowledged in PagerDuty
2024-12-09 14:24:00 - Checked logs: OOM error detected
2024-12-09 14:25:30 - Increased memory limit to 16GB
2024-12-09 14:26:00 - Restarting node
2024-12-09 14:29:00 - Node back online
2024-12-09 14:32:00 - Verified peer connectivity restored
2024-12-09 14:35:00 - Incident resolved
2024-12-09 14:40:00 - Created ticket for memory tuning
```

### Useful Links

- [Architecture Docs](../../docs/ARCHITECTURE.md)
- [Runbooks](./incident-response.md)
- [Monitoring Dashboard](http://localhost:3000)
- [PagerDuty](https://bitcell.pagerduty.com)
- [Status Page](https://status.bitcell.network)

---

**Last Updated:** 2024-12-09
**Maintained By:** Platform Team
**Questions:** #platform-team on Slack
