use std::io::{self, Read, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use serde_json::json;
use crate::errors::{AppError, Result};
use crate::session::SessionManager;
use crate::telemetry::activity_aggregator::ActivityAggregator;
use crate::native_messaging::payload::validate_native_payload;

/// Read a native messaging frame: 4-byte little-endian length prefix followed by JSON UTF-8 bytes.
pub fn read_native_frame<R: Read>(reader: &mut R) -> Result<Option<String>> {
    let mut len_bytes = [0u8; 4];
    match reader.read_exact(&mut len_bytes) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(AppError::InternalError(format!("Native read header error: {}", e))),
    }

    let len = u32::from_le_bytes(len_bytes) as usize;
    if len == 0 || len > 1024 * 1024 {
        return Err(AppError::InternalError(format!("Invalid native message size: {}", len)));
    }

    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)
        .map_err(|e| AppError::InternalError(format!("Native read body error: {}", e)))?;

    let s = String::from_utf8(buf)
        .map_err(|e| AppError::InternalError(format!("Native message invalid UTF-8: {}", e)))?;

    Ok(Some(s))
}

/// Write a native messaging frame: 4-byte little-endian length prefix followed by UTF-8 bytes.
pub fn write_native_frame<W: Write>(writer: &mut W, data: &[u8]) -> Result<()> {
    let len = (data.len() as u32).to_le_bytes();
    writer.write_all(&len)
        .map_err(|e| AppError::InternalError(format!("Native write header error: {}", e)))?;
    writer.write_all(data)
        .map_err(|e| AppError::InternalError(format!("Native write body error: {}", e)))?;
    writer.flush()
        .map_err(|e| AppError::InternalError(format!("Native write flush error: {}", e)))?;
    Ok(())
}

/// Process a single incoming native message string and update telemetry
pub async fn process_incoming_native_message(
    raw_json: &str,
    session_manager: &Arc<Mutex<SessionManager>>,
    activity_aggregator: &Arc<Mutex<ActivityAggregator>>,
) -> Result<String> {
    if let Some(payload) = validate_native_payload(raw_json) {
        let (work_id, emp_id) = {
            let sm = session_manager.lock().await;
            if let Some(sess) = sm.active_session() {
                (sess.id.clone(), sess.employee_id.clone())
            } else {
                (String::new(), String::new())
            }
        };

        if !work_id.is_empty() {
            let mut agg = activity_aggregator.lock().await;
            let _ = agg.update_active_website(&work_id, &emp_id, &payload.domain);
        }

        let resp = json!({
            "status": "ok",
            "domain": payload.domain,
            "browser": payload.browser,
        });
        Ok(resp.to_string())
    } else {
        let resp = json!({
            "status": "error",
            "message": "invalid_payload"
        });
        Ok(resp.to_string())
    }
}

/// Run the native host loop over synchronous reader and writer streams.
pub async fn run_native_host_stream<R: Read + Send + 'static, W: Write + Send + 'static>(
    mut reader: R,
    mut writer: W,
    session_manager: Arc<Mutex<SessionManager>>,
    activity_aggregator: Arc<Mutex<ActivityAggregator>>,
) {
    loop {
        // Run blocking read on blocking thread pool
        let maybe_msg = tokio::task::spawn_blocking(move || {
            let res = read_native_frame(&mut reader);
            (res, reader)
        }).await;

        match maybe_msg {
            Ok((Ok(Some(msg)), next_reader)) => {
                reader = next_reader;
                let resp_str = match process_incoming_native_message(
                    &msg,
                    &session_manager,
                    &activity_aggregator,
                ).await {
                    Ok(s) => s,
                    Err(e) => json!({"status": "error", "message": e.to_string()}).to_string(),
                };

                let resp_bytes = resp_str.into_bytes();
                let write_res = tokio::task::spawn_blocking(move || {
                    let res = write_native_frame(&mut writer, &resp_bytes);
                    (res, writer)
                }).await;

                match write_res {
                    Ok((Ok(()), next_writer)) => {
                        writer = next_writer;
                    }
                    _ => break,
                }
            }
            _ => break,
        }
    }
}

/// Spawns a background listener on standard I/O if the app is launched in native host mode.
pub fn spawn_native_messaging_listener(
    session_manager: Arc<Mutex<SessionManager>>,
    activity_aggregator: Arc<Mutex<ActivityAggregator>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let stdin = io::stdin();
        let stdout = io::stdout();
        run_native_host_stream(stdin, stdout, session_manager, activity_aggregator).await;
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use crate::persistence::Database;

    #[tokio::test]
    async fn test_native_frame_roundtrip() {
        let payload = r#"{"type":"tab_active","browser":"chrome","domain":"github.com","timestamp":"2026-09-08T12:00:00Z"}"#;
        let mut buffer = Vec::new();

        write_native_frame(&mut buffer, payload.as_bytes()).unwrap();
        assert_eq!(buffer.len(), 4 + payload.len());

        let mut cursor = Cursor::new(buffer);
        let read_result = read_native_frame(&mut cursor).unwrap();
        assert_eq!(read_result.unwrap(), payload);
    }

    #[tokio::test]
    async fn test_process_native_message() {
        let db = Database::new_in_memory().unwrap();
        let sm = Arc::new(Mutex::new(SessionManager::new(db.clone())));
        let agg = Arc::new(Mutex::new(ActivityAggregator::new(db)));

        let msg = r#"{"type":"tab_active","browser":"chrome","domain":"linear.app","timestamp":"2026-09-08T12:00:00Z"}"#;
        let resp = process_incoming_native_message(msg, &sm, &agg).await.unwrap();
        assert!(resp.contains("\"status\":\"ok\""));
        assert!(resp.contains("linear.app"));
    }
}
