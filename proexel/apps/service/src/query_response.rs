use appcore_bin::application::{ApiResponse, RuntimeResult};
use serde_json::Value;

pub(crate) fn json_response(status_code: u16, value: Value) -> RuntimeResult<ApiResponse> {
    match serde_json::to_vec(&value) {
        Ok(payload) => Ok(ApiResponse {
            status_code,
            payload,
        }),
        Err(error) => {
            eprintln!("proexel response encoding failed reason={error}");
            Ok(ApiResponse {
                status_code: 500,
                payload: br#"{"error":"response_encode_failed"}"#.to_vec(),
            })
        }
    }
}
