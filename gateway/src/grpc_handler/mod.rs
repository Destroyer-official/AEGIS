use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::auth::{AuthUser, Claims};
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
}

impl GrpcQueryService {
    pub fn new(trino: Arc<TrinoRouter>) -> Self {
        Self { trino }
    }

    pub fn into_service(self) -> QueryServiceServer<Self> {
        QueryServiceServer::new(self)
    }

    pub async fn execute_stream_query(
        &self,
        request: Request<StreamRequest>,
    ) -> Result<
        Response<
            tonic::codegen::tokio_stream::wrappers::ReceiverStream<Result<StreamChunk, Status>>,
        >,
        Status,
    > {
        QueryService::stream_query(self, request).await
    }
}

fn create_auth_claims(user: &str, role: &str) -> AuthUser {
    AuthUser {
        claims: Claims {
            sub: user.to_string(),
            role: role.to_string(),
            tenant_id: "default".to_string(),
            exp: 9999999999,
            iat: 1000000000,
        },
    }
}

#[tonic::async_trait]
impl QueryService for GrpcQueryService {
    async fn execute_query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        let req = request.into_inner();
        let user = std::env::var("GRPC_DEFAULT_USER").unwrap_or_else(|_| "grpc-service".into());
        let role = std::env::var("GRPC_DEFAULT_ROLE").unwrap_or_else(|_| "admin".into());
        let auth = create_auth_claims(&user, &role);

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
        let req = request.into_inner();
        let user = std::env::var("GRPC_DEFAULT_USER").unwrap_or_else(|_| "grpc-service".into());
        let role = std::env::var("GRPC_DEFAULT_ROLE").unwrap_or_else(|_| "admin".into());
        let auth = create_auth_claims(&user, &role);

        let batch_size = req.batch_size.max(1) as usize;
        let trino = Arc::clone(&self.trino);

        let (tx, rx) = tokio::sync::mpsc::channel(32);

        tokio::spawn(async move {
            let result = trino.execute_query(&req.sql, &auth).await;
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
