# Glossary

Key terms and concepts in BitCell.

## A

**Account**  
An entity on the blockchain with an address and balance. Can be externally-owned (controlled by private key) or a smart contract.

**Address**  
A 20-byte identifier for an account, displayed as a hex string starting with `0x`. Example: `0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb`

## B

**Battle**  
A tournament match where two miners' glider patterns compete in a cellular automaton simulation. The pattern with higher regional energy after 1000 steps wins.

**Block**  
A collection of transactions grouped together and added to the blockchain. In BitCell, blocks are proposed by tournament winners.

**Block Explorer**  
A web interface for viewing blockchain data (blocks, transactions, accounts). Similar to Etherscan but for BitCell.

**Block Height**  
The number of blocks in the chain up to and including a specific block. Genesis block has height 0.

**Bond**  
CELL tokens locked by a miner to participate in tournaments. Minimum bond is 1000 CELL. Slashed if miner misbehaves.

## C

**CA (Cellular Automaton)**  
A grid of cells that evolve according to rules based on neighboring cells. BitCell uses Conway's Game of Life rules for tournament battles.

**CELL**  
BitCell's native token. Used for transaction fees, miner bonds, and block rewards.

**Cellular Automaton**  
See CA.

**Commitment**  
A cryptographic hash that hides data but allows later verification. Miners commit to glider patterns before revealing them.

**Consensus**  
The mechanism by which the network agrees on the blockchain state. BitCell uses tournament consensus instead of PoW or PoS.

**Conway's Game of Life**  
A cellular automaton with simple rules that creates complex patterns. Used in BitCell for tournament battles.

## D

**Deterministic**  
Producing the same output for the same input every time. CA battles are deterministic - same patterns always produce the same outcome.

## E

**EBSL (Evidence-Based Subjective Logic)**  
BitCell's reputation system. Tracks positive evidence (r) and negative evidence (s) for each miner to compute trust scores.

**Energy**  
In CA battles, living cells have energy values. Regional energy determines battle outcomes.

**Epoch**  
A fixed period of blocks (e.g., 1000 blocks). Used for trust score decay and other periodic operations.

**Equivocation**  
Proposing two conflicting blocks at the same height. Results in immediate slashing and permanent ban.

## F

**Finality**  
The point at which a transaction is considered irreversible. BitCell uses probabilistic finality - 6+ confirmations recommended.

**Fork**  
When the blockchain splits into two competing chains. Usually resolves quickly as the network converges on the longest chain.

## G

**Gas**  
A measure of computational work. Each transaction consumes gas, and users pay gas fees to miners.

**Genesis Block**  
The first block in the blockchain (height 0). Contains initial state and configuration.

**Glider**  
A pattern in Conway's Game of Life that moves across the grid. Miners use glider patterns to compete in tournaments.

**Groth16**  
A type of zero-knowledge proof (zkSNARK). Used in BitCell for battle proofs and contract execution proofs.

## H

**Hash**  
A fixed-size output from a cryptographic hash function. Used for commitments, block IDs, transaction IDs, etc.

**HWSS (Heavyweight Spaceship)**  
A large, slow-moving pattern in Conway's Game of Life. High energy but predictable.

## L

**Light Client**  
A node that only downloads and verifies block headers, not full blocks. Requires minimal resources.

**LWSS (Lightweight Spaceship)**  
A small, fast-moving pattern in Conway's Game of Life. Low energy but evasive.

## M

**Mempool**  
The set of unconfirmed transactions waiting to be included in a block.

**Miner**  
A node that participates in tournaments to propose new blocks and earn rewards.

**MWSS (Middleweight Spaceship)**  
A medium-sized pattern in Conway's Game of Life. Balanced energy and speed.

## N

**Node**  
A computer running BitCell software. Can be a validator, miner, or light client.

**Nonce**  
A number used once. Used in commitments and transaction replay protection.

## P

**Pedersen Commitment**  
A cryptographic commitment scheme used for hiding smart contract state while allowing proofs.

**Private Key**  
A secret key that controls an account. Must be kept secure. Loss means permanent loss of funds.

**Proof**  
A zero-knowledge proof that verifies computation without revealing inputs. BitCell uses Groth16 proofs.

## R

**Recovery Phrase**  
A list of 12 or 24 words that can recover a wallet. Also called seed phrase or mnemonic.

**Regional Energy**  
The sum of energy values in a region of the CA grid. Used to determine battle winners.

**Reveal Phase**  
The tournament phase where miners reveal their committed glider patterns.

**Ring Signature**  
A cryptographic signature that proves the signer is one of a group, without revealing which one. Used for tournament anonymity.

## S

**Smart Contract**  
A program that runs on the blockchain. In BitCell, contracts are executed in the ZKVM with zero-knowledge privacy.

**Spaceship**  
A class of patterns in Conway's Game of Life that move across the grid. Includes LWSS, MWSS, HWSS.

**State**  
The current data stored in accounts and contracts. BitCell's state is encrypted using Pedersen commitments.

**State Root**  
A hash of the entire state. Changes with every block. Used to verify state consistency.

## T

**Tournament**  
BitCell's consensus mechanism. Eligible miners compete in bracket-style CA battles, and the winner proposes the next block.

**Transaction**  
An operation that changes blockchain state. Examples: transfer tokens, deploy contract, call contract function.

**Trust Score**  
A value from 0 to 1 representing a miner's reputation. Computed from positive and negative evidence using EBSL.

## V

**Validator**  
A node that verifies blocks and transactions. All validators must verify all proofs (no sampling in consensus).

**VRF (Verifiable Random Function)**  
A cryptographic function that produces randomness with a proof of correctness. Used for tournament pairings.

## W

**Wallet**  
Software that manages private keys and allows sending transactions. Can be CLI or GUI.

## Z

**Zero-Knowledge Proof**  
A cryptographic proof that verifies a statement is true without revealing why it's true. Used for private smart contracts.

**ZK-SNARK**  
Zero-Knowledge Succinct Non-Interactive Argument of Knowledge. A type of zero-knowledge proof. BitCell uses Groth16 SNARKs.

**ZKASM**  
Zero-Knowledge Assembly. BitCell's assembly language for smart contracts. Low-level but efficient.

**ZKVM**  
Zero-Knowledge Virtual Machine. BitCell's execution environment for smart contracts.
