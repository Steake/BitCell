//! Consensus Layer for BitCell
//!
//! Implements tournament-based consensus with:
//! - Block structures
//! - Tournament commit-reveal protocol
//! - VRF-based randomness
//! - Eligibility and miner set management
//! - Fork choice (heaviest chain)

pub mod block;
pub mod tournament;
pub mod fork_choice;
pub mod orchestrator;
pub mod finality;

pub use block::{Block, BlockHeader, Transaction, BattleProof};
pub use tournament::{Tournament, TournamentPhase, GliderCommitment, GliderReveal, TournamentMatch};
pub use fork_choice::ChainState;
pub use orchestrator::TournamentOrchestrator;
pub use finality::{FinalityGadget, FinalityVote, FinalityStatus, VoteType, EquivocationEvidence};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid block")]
    InvalidBlock,
    
    #[error("Tournament error: {0}")]
    TournamentError(String),
    
    #[error("Fork choice error: {0}")]
    ForkChoiceError(String),
}
