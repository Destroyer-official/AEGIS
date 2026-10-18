use std::time::Instant;
use tonic::{Request, Response, Status};

use crate::LzrEngine;
use aegis_proto::types::{
    ClassifyBatchRequest, ClassifyBatchResponse, ClassifyRequest, ClassifyResponse, Empty,
    HealthCheckResponse,
};

/// gRPC service wrapping the LZR classifier engine.
///
/// **Stability**: Unstable — internal API for Dev 9 gateway integration.
/// This is a manual implementation that doesn't require generated tonic code.
pub struct LzrService {
    engine: LzrEngine,
}

impl LzrService {
    /// Create a new LZR gRPC service with the default embedded signature set.
    pub fn new() -> Self {
        Self {
            engine: LzrEngine::new(),
        }
    }

    /// Create service with a custom engine (for testing or custom signatures).
    pub fn with_engine(engine: LzrEngine) -> Self {
        Self { engine }
    }

    /// Classify a single payload against the LZR signature database.
    ///
    /// Target p99 latency < 1ms.
    #[allow(clippy::result_large_err)]
    pub async fn classify(
        &self,
        request: Request<ClassifyRequest>,
    ) -> Result<Response<ClassifyResponse>, Status> {
        let req = request.into_inner();
        let start = Instant::now();

        if req.payload.len() > 64 * 1024 {
            return Err(Status::invalid_argument("payload exceeds 64KB limit"));
        }

        let result = self.engine.classify(&req.payload);
        let latency_us = start.elapsed().as_micros() as u64;

        Ok(Response::new(ClassifyResponse {
            protocol: result.protocol,
            category: result.category,
            priority: result.priority,
            matched: result.matched,
            latency_us,
            signature_count: self.engine.signature_count() as u32,
        }))
    }

    /// Classify multiple payloads in a single RPC call.
    #[allow(clippy::result_large_err)]
    pub async fn classify_batch(
        &self,
        request: Request<ClassifyBatchRequest>,
    ) -> Result<Response<ClassifyBatchResponse>, Status> {
        let req = request.into_inner();

        if req.requests.len() > 1000 {
            return Err(Status::invalid_argument("batch size exceeds 1000 limit"));
        }

        let responses: Vec<ClassifyResponse> = req
            .requests
            .iter()
            .map(|r| {
                let start = Instant::now();
                let result = self.engine.classify(&r.payload);
                let latency_us = start.elapsed().as_micros() as u64;

                ClassifyResponse {
                    protocol: result.protocol,
                    category: result.category,
                    priority: result.priority,
                    matched: result.matched,
                    latency_us,
                    signature_count: self.engine.signature_count() as u32,
                }
            })
            .collect();

        Ok(Response::new(ClassifyBatchResponse { responses }))
    }

    /// Health check — returns engine status.
    #[allow(clippy::result_large_err)]
    pub async fn health_check(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        Ok(Response::new(HealthCheckResponse {
            ready: true,
            signature_count: self.engine.signature_count() as u32,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }))
    }
}

impl Default for LzrService {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for the LZR classifier gRPC service (manual, no generated code needed).
#[tonic::async_trait]
pub trait LzrClassifier {
    async fn classify(
        &self,
        request: Request<ClassifyRequest>,
    ) -> Result<Response<ClassifyResponse>, Status>;

    async fn classify_batch(
        &self,
        request: Request<ClassifyBatchRequest>,
    ) -> Result<Response<ClassifyBatchResponse>, Status>;

    async fn health_check(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<HealthCheckResponse>, Status>;
}

#[tonic::async_trait]
impl LzrClassifier for LzrService {
    async fn classify(
        &self,
        request: Request<ClassifyRequest>,
    ) -> Result<Response<ClassifyResponse>, Status> {
        self.classify(request).await
    }

    async fn classify_batch(
        &self,
        request: Request<ClassifyBatchRequest>,
    ) -> Result<Response<ClassifyBatchResponse>, Status> {
        self.classify_batch(request).await
    }

    async fn health_check(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        self.health_check(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_classify_grpc() {
        let service = LzrService::new();

        let request = Request::new(ClassifyRequest {
            payload: b"HTTP/1.1 200 OK\r\n\r\n".to_vec(),
            tls_hint: String::new(),
            source_ip: vec![],
        });

        let response = service.classify(request).await.unwrap();
        let result = response.into_inner();

        assert!(result.matched);
        assert_eq!(result.protocol, "HTTP/1.1");
        assert_eq!(result.category, "web");
        assert!(result.latency_us > 0);
        assert!(result.signature_count >= 400);
    }

    #[tokio::test]
    async fn test_classify_batch_grpc() {
        let service = LzrService::new();

        let payloads: Vec<Vec<u8>> = vec![
            b"HTTP/1.1 200 OK\r\n".to_vec(),
            b"SSH-2.0-OpenSSH_8.9\r\n".to_vec(),
            b"220 mail.example.com ESMTP\r\n".to_vec(),
        ];

        let requests: Vec<ClassifyRequest> = payloads
            .into_iter()
            .map(|p| ClassifyRequest {
                payload: p,
                tls_hint: String::new(),
                source_ip: vec![],
            })
            .collect();

        let request = Request::new(ClassifyBatchRequest { requests });
        let response = service.classify_batch(request).await.unwrap();
        let result = response.into_inner();

        assert_eq!(result.responses.len(), 3);
        assert_eq!(result.responses[0].protocol, "HTTP/1.1");
        assert_eq!(result.responses[1].protocol, "SSH");
        assert_eq!(result.responses[2].protocol, "SMTP");
    }

    #[tokio::test]
    async fn test_health_check_grpc() {
        let service = LzrService::new();

        let request = Request::new(Empty {});
        let response = service.health_check(request).await.unwrap();
        let result = response.into_inner();

        assert!(result.ready);
        assert!(result.signature_count >= 400);
        assert!(!result.version.is_empty());
    }

    #[tokio::test]
    async fn test_classify_unknown_payload() {
        let service = LzrService::new();

        let request = Request::new(ClassifyRequest {
            payload: b"RANDOM UNKNOWN DATA".to_vec(),
            tls_hint: String::new(),
            source_ip: vec![],
        });

        let response = service.classify(request).await.unwrap();
        let result = response.into_inner();

        assert!(!result.matched);
        assert_eq!(result.protocol, "unknown");
    }

    #[tokio::test]
    async fn test_classify_payload_too_large() {
        let service = LzrService::new();

        let request = Request::new(ClassifyRequest {
            payload: vec![0u8; 65 * 1024],
            tls_hint: String::new(),
            source_ip: vec![],
        });

        let result = service.classify(request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn test_classify_batch_too_large() {
        let service = LzrService::new();

        let requests: Vec<ClassifyRequest> = (0..1001)
            .map(|_| ClassifyRequest {
                payload: b"test".to_vec(),
                tls_hint: String::new(),
                source_ip: vec![],
            })
            .collect();

        let request = Request::new(ClassifyBatchRequest { requests });
        let result = service.classify_batch(request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
    }
}
