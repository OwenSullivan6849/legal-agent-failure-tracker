use legal_agent_failure_tracker::matter_loop::{evaluate, AgentFailure, MatterRun, MatterStage};

#[test]
fn delivery_failure_stops_before_deadline_follow_up() {
    let run = MatterRun {
        matter_id: "MAT-1042".into(),
        intake_accepted: true,
        signed_document_delivered: false,
        deadline_follow_up_scheduled: false,
    };

    let failure = evaluate(&run).expect_err("delivery must block the later stage");
    assert_eq!(failure.stage(), MatterStage::SignedDocumentDelivery);
    assert_eq!(
        failure,
        AgentFailure::SignedDocumentDelivery { matter_id: "MAT-1042".into() }
    );
}

