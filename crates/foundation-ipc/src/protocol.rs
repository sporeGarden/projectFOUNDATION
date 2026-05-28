// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC 2.0 protocol types — request/response wire format.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcRequest {
    /// Protocol version — always "2.0".
    pub jsonrpc: &'static str,
    /// Semantic method name (`domain.verb`).
    pub method: String,
    /// Parameters object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    /// Request ID for correlation.
    pub id: u64,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC 2.0 request.
    #[must_use]
    pub fn new(method: impl Into<String>, params: Option<Value>, id: u64) -> Self {
        Self {
            jsonrpc: "2.0",
            method: method.into(),
            params,
            id,
        }
    }

    /// Serialize to newline-delimited JSON (wire format).
    ///
    /// # Errors
    ///
    /// Returns serialization error if params contain non-serializable values.
    pub fn to_wire(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut bytes = serde_json::to_vec(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}

/// A JSON-RPC 2.0 response.
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcResponse {
    /// Protocol version.
    #[allow(dead_code)]
    pub jsonrpc: String,
    /// Successful result (mutually exclusive with `error`).
    pub result: Option<Value>,
    /// Error object (mutually exclusive with `result`).
    pub error: Option<JsonRpcError>,
    /// Correlation ID matching the request.
    pub id: Option<u64>,
}

/// JSON-RPC error object.
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcError {
    /// Standard error code.
    pub code: i64,
    /// Human-readable error message.
    pub message: String,
    /// Optional structured error data.
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    /// Extract the result value, or return an error if the response indicates failure.
    ///
    /// # Errors
    ///
    /// Returns the RPC error code and message if the response has an error field.
    pub fn into_result(self) -> Result<Value, (i64, String)> {
        if let Some(err) = self.error {
            return Err((err.code, err.message));
        }
        Ok(self.result.unwrap_or(Value::Null))
    }

    /// Parse a response from a newline-delimited JSON byte slice.
    ///
    /// # Errors
    ///
    /// Returns deserialization error on malformed input.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

/// Standard JSON-RPC error codes.
pub mod error_codes {
    /// Parse error — invalid JSON.
    pub const PARSE_ERROR: i64 = -32700;
    /// Invalid request object.
    pub const INVALID_REQUEST: i64 = -32600;
    /// Method not found.
    pub const METHOD_NOT_FOUND: i64 = -32601;
    /// Invalid method parameters.
    pub const INVALID_PARAMS: i64 = -32602;
    /// Internal error.
    pub const INTERNAL_ERROR: i64 = -32603;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_serialization() {
        let req = JsonRpcRequest::new("health.liveness", Some(serde_json::json!({})), 1);
        let wire = req.to_wire().unwrap();
        let s = String::from_utf8(wire).unwrap();
        assert!(s.ends_with('\n'));
        assert!(s.contains("\"jsonrpc\":\"2.0\""));
        assert!(s.contains("\"method\":\"health.liveness\""));
    }

    #[test]
    fn response_success() {
        let json = r#"{"jsonrpc":"2.0","result":{"status":"alive"},"id":1}"#;
        let resp: JsonRpcResponse = serde_json::from_str(json).unwrap();
        let result = resp.into_result().unwrap();
        assert_eq!(result["status"], "alive");
    }

    #[test]
    fn response_error() {
        let json =
            r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#;
        let resp: JsonRpcResponse = serde_json::from_str(json).unwrap();
        let err = resp.into_result().unwrap_err();
        assert_eq!(err.0, -32601);
    }
}
