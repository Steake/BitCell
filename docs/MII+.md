Implement the following:

---

## **1. Primary Tie-Breaker: Mutual Influence Integral (MII)**

After each tournament match, if both patterns end with identical final energy totals, compute the “Mutual Influence Integral” for each participant.

Definition:

For miner **A** affecting miner **B**:

* Track, at each CA timestep `t`, the energy delta in the opponent’s scoring region `R_B`.
* Square each delta before summing to amplify high-impact moves.
* Accumulate across all timesteps.

Formally:

```
MII(A,B) = sum_t( sum_cells_in(R_B)( (delta_energy_cell[t])^2 ) )
MII(B,A) = same but in R_A
```

Winner = side with the larger MII value.

This must be deterministic and ZK-provable. The agent should expose it as a standalone function callable during match finalization.

---

## **2. Secondary Tie-Breaker: Temporal Entropy Differential (TED)**

If MII scores are exactly equal:

Compute Shannon entropy for each opponent region across the entire simulation timeline.

For region R_B affected by A:

```
TED(A,B) = sum_t( entropy(R_B_state[t]) )
```

Winner = side with the higher TED score.

Entropy implementation must be deterministic and compatible with the proving circuit constraints. Use fixed-precision arithmetic already present in the CA engine.

---

## **3. Final Tie-Breaker: Lexicographic Seed Break**

If TED also ties:

* Hash each pattern with the global VRF seed of the current block height.
* Interpret hash outputs as unsigned integers.
* The lower numeric value wins.

Definition:

```
L_A = hash(G_A || global_seed)
L_B = hash(G_B || global_seed)
Winner = min(L_A, L_B)
```

This ensures fully deterministic resolution with no extra signalling channels.

---

## **4. Optional Mechanic: Deterministic Evolving Cell Phenotypes**

> **Note:** This feature is deferred to a future PR. The implementation below describes the planned design.

Add a phenotype field to each cell (2–4 bits). Mutation occurs when cell energy exceeds a threshold `theta`.

Mutation rule:

```
if energy > theta:
    phenotype = hash(x_coord, y_coord, timestep) % P
```

Where:

* `P` = number of phenotype classes (2–4 recommended)
* Hash must be the protocol-standard short hash function

Phenotypes must modify CA rules in deterministic ways. Example rule modifications to implement:

* Phenotype 0 → classic Life
* Phenotype 1 → +1 survival buffer
* Phenotype 2 → spreads death to one extra neighbour
* Phenotype 3 → boosts birth thresholds locally

All phenotype behaviours must be deterministic, easy to constrain within the proving system, and not introduce randomness.

Integrate phenotype transitions into the existing CA step function and include phenotype state in the ZK transition commitments.

---

## **5. Integration Requirements**

* Extend the match-resolution module with the full tie-breaker pipeline:

  1. compute_mii()
  2. compute_ted()
  3. lexicographic_seed_break()
* Ensure spectral invariants (energy accounting, deterministic transitions) remain intact.
* Update tournament harness to call tie-breakers only when final energy scores match.
* Update the proving circuit interface so the tie-breaker logic, including MII and TED contributions, is circuit-verifiable.
* Update tests to:

  * confirm identical initial states always produce identical tie-breaking outcomes,
  * verify no miner can signal across commitments,
  * ensure phenotype mutation cannot be influenced across regions.

---

## **6. Deliverables**

Produce:

* Full implementation of the three-stage tie breaker.
* Integration into CA step logic.
* Updated proving-compatible data structures.
* Unit tests and regression tests.
* Simulation traces demonstrating correct behaviour.

All components must remain deterministic, reproducible from commitments, and safe against collusion.

