use crate::models::AuditLog;

/// Log an API request to the audit trail (for future audit integration)
#[allow(dead_code)]
pub fn log_request(audit: &AuditLog) {
    tracing::info!(
        agent_id = %audit.agent_id,
        service = %audit.service_id,
        endpoint = %audit.endpoint,
        method = %audit.method,
        status = audit.status_code,
        duration_ms = audit.response_time_ms,
        "API request"
    );
}
