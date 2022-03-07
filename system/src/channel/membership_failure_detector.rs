use tokio::sync::mpsc;

pub type MembershipFailureDetectorReceiver = mpsc::Receiver<MembershipFailureDetectorRequest>;
pub type MembershipFailureDetectorSender = mpsc::Sender<MembershipFailureDetectorRequest>;

#[derive(Clone, Debug)]
pub enum MembershipFailureDetectorRequest {
    Launch,
}

pub async fn build_failure_detector_channel() -> (
    MembershipFailureDetectorSender,
    MembershipFailureDetectorReceiver,
) {
    let (sender, receiver) = mpsc::channel::<MembershipFailureDetectorRequest>(1);

    (sender, receiver)
}

pub async fn launch_failure_detector(
    failure_detector: &MembershipFailureDetectorSender,
) -> Result<(), Box<dyn std::error::Error>> {
    failure_detector
        .send(MembershipFailureDetectorRequest::Launch)
        .await?;

    Ok(())
}
