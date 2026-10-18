use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::auth::{AuthUser, JwtConfig};
use crate::query::TrinoRouter;

pub mod gateway_proto {
    tonic::include_proto!("aegis.gateway");
}

use gateway_proto::{
    query_service_server::{QueryService, QueryServiceServer},
    Column, QueryRequest, QueryResponse, Row, StreamChunk, StreamRequest,
};

#[derive(Clone)]
pub struct GrpcQueryService {
    trino: Arc<TrinoRouter>,
    jwt_config: Arc<JwtConfig>,
}

impl GrpcQueryService {
    pub fn new(trino: Arc<TrinoRouter>, jwt_config: Arc<JwtConfig>) -> Self {
        Self { trino, jwt_config }
    }

    pub fn into_service(self) -> QueryServiceServer<Self> {
        QueryServiceServer::new(self)
    }

    /// Authenticate request from gRPC metadata Bearer token
    fn authenticate_request<T>(&self, request: &Request<T>) -> Result<AuthUser, Status> {
        let auth_header = request
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                Status::unauthenticated("Missing 'authorization' header in gRPC metadata")
            })?;

        let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
            Status::unauthenticated("Invalid authorization header, expected 'Bearer <token>'")
        })?;

        let claims = crate::auth::verify_token(token, &self.jwt_config)
            .map_err(|e| Status::unauthenticated(format!("Invalid or expired JWT: {}", e)))?;

        Ok(AuthUser { claims })
    }

    /// Internal stream execution using the caller's pre-authenticated AuthUser context
    pub async fn execute_stream_query_as_user(
        &self,
        request: Request<StreamRequest>,
        auth: &AuthUser,
    ) -> Result<
        Response<
            tonic::codegen::tokio_stream::wrappers::ReceiverStream<Result<StreamChunk, Status>>,
        >,
        Status,
    > {
        let req = request.into_inner();
        let batch_size = req.batch_size.max(1) as usize;
        let trino = Arc::clone(&self.trino);
        let user = auth.clone();

        let (tx, rx) = tokio::sync::mpsc::channel(32);

        tokio::spawn(async move {
            let result = trino.execute_query(&req.sql, &user).await;
            match result {
                Ok(qr) => {
                    for chunk in qr.rows.chunks(batch_size) {
                        let stream_chunk = StreamChunk {
                            rows: chunk.iter().map(|r| Row { values: r.clone() }).collect(),
                            is_last: false,
                            query_id: qr.query_id.clone(),
                        };
                        if tx.send(Ok(stream_chunk)).await.is_err() {
                            break;
                        }
                    }
                    let _ = tx
                        .send(Ok(StreamChunk {
                            rows: vec![],
                            is_last: true,
                            query_id: qr.query_id,
                        }))
                        .await;
                }
                Err(e) => {
                    let _ = tx
                        .send(Err(Status::internal(format!("query failed: {}", e))))
                        .await;
                }
            }
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            rx,
        )))
    }
}

#[tonic::async_trait]
impl QueryService for GrpcQueryService {
    async fn execute_query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        // Enforce verified identity from gRPC authorization metadata
        let auth = self.authenticate_request(&request)?;

        let req = request.into_inner();
        let result = self
            .trino
            .execute_query(&req.sql, &auth)
            .await
            .map_err(|e| Status::internal(format!("query failed: {}", e)))?;

        let response = QueryResponse {
            columns: result
                .columns
                .into_iter()
                .map(|c| Column {
                    name: c.name,
                    data_type: c.data_type,
                })
                .collect(),
            rows: result.rows.into_iter().map(|r| Row { values: r }).collect(),
            total_rows: result.total_rows as i64,
            query_id: result.query_id,
        };

        Ok(Response::new(response))
    }

    type StreamQueryStream = tokio_stream::wrappers::ReceiverStream<Result<StreamChunk, Status>>;

    async fn stream_query(
        &self,
        request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamQueryStream>, Status> {
        // Enforce verified identity from gRPC authorization metadata
        let auth = self.authenticate_request(&request)?;
        self.execute_stream_query_as_user(request, &auth).await
    }
}
