use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct MatterRun {
    pub matter_id: String,
    pub intake_accepted: bool,
    pub signed_document_delivered: bool,
    pub deadline_follow_up_scheduled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatterStage {
    MatterIntake,
    SignedDocumentDelivery,
    DeadlineFollowUp,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentFailure {
    MatterIntake { matter_id: String },
    SignedDocumentDelivery { matter_id: String },
    DeadlineFollowUp { matter_id: String },
}

impl AgentFailure {
    pub fn stage(&self) -> MatterStage {
        match self {
            Self::MatterIntake { .. } => MatterStage::MatterIntake,
            Self::SignedDocumentDelivery { .. } => MatterStage::SignedDocumentDelivery,
            Self::DeadlineFollowUp { .. } => MatterStage::DeadlineFollowUp,
        }
    }

    pub fn matter_id(&self) -> &str {
        match self {
            Self::MatterIntake { matter_id }
            | Self::SignedDocumentDelivery { matter_id }
            | Self::DeadlineFollowUp { matter_id } => matter_id,
        }
    }

    pub fn idempotency_key(&self) -> String {
        format!("matter:{}:{:?}", self.matter_id(), self.stage())
    }
}

impl std::fmt::Display for AgentFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "legal agent failed at {:?} for matter {}", self.stage(), self.matter_id())
    }
}

impl std::error::Error for AgentFailure {}

pub fn evaluate(run: &MatterRun) -> Result<MatterStage, AgentFailure> {
    if !run.intake_accepted {
        return Err(AgentFailure::MatterIntake { matter_id: run.matter_id.clone() });
    }
    if !run.signed_document_delivered {
        return Err(AgentFailure::SignedDocumentDelivery { matter_id: run.matter_id.clone() });
    }
    if !run.deadline_follow_up_scheduled {
        return Err(AgentFailure::DeadlineFollowUp { matter_id: run.matter_id.clone() });
    }
    Ok(MatterStage::Complete)
}

