use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Clone)]
pub struct PermissionRequest {
    pub command: String,
    pub args: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PermissionResponse {
    Approved,
    ApprovedSimilar,
    Denied,
}

pub type PermissionRequestSender = mpsc::Sender<(PermissionRequest, oneshot::Sender<PermissionResponse>)>;
pub type PermissionRequestReceiver = mpsc::Receiver<(PermissionRequest, oneshot::Sender<PermissionResponse>)>;

/// Create a permission request channel
pub fn create_permission_channel() -> (PermissionRequestSender, PermissionRequestReceiver) {
    mpsc::channel(10)
}
