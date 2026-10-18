use axum::extract::{
    ws::{Message, WebSocket},
    Extension,
};
use futures_util::StreamExt;
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info};

use crate::auth::AuthUser;
use crate::grpc_handler::{gateway_proto, GrpcQueryService};
use crate::query::TrinoRouter;

pub fn ws_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    user: AuthUser,
    trino: Arc<TrinoRouter>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_socket(socket, user, trino))
}

pub async fn ws_stream_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    user: AuthUser,
    Extension(grpc_service): Extension<Arc<GrpcQueryService>>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_stream_socket(socket, user, grpc_service))
}

async fn handle_socket(mut socket: WebSocket, user: AuthUser, trino: Arc<TrinoRouter>) {
    info!("WebSocket connected: user={}", user.claims.sub);

    while let Some(msg) = socket.next().await {
        match msg {
            Ok(Message::Text(text)) => match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(val) => {
                    let sql = val.get("sql").and_then(|s| s.as_str()).unwrap_or("");
                    if sql.is_empty() {
                        let _ = socket
                            .send(Message::Text(
                                json!({"error": "missing 'sql' field"}).to_string(),
                            ))
                            .await;
                        continue;
                    }

                    match trino.execute_query(sql, &user).await {
                        Ok(result) => {
                            let _ = socket
                                .send(Message::Text(
                                    serde_json::to_string(&result).unwrap_or_default(),
                                ))
                                .await;
                        }
                        Err(status) => {
                            let _ = socket
                                .send(Message::Text(
                                    json!({"error": format!("query failed: {}", status)})
                                        .to_string(),
                                ))
                                .await;
                        }
                    }
                }
                Err(e) => {
                    let _ = socket
                        .send(Message::Text(
                            json!({"error": format!("invalid JSON: {}", e)}).to_string(),
                        ))
                        .await;
                }
            },
            Ok(Message::Close(_)) => {
                info!("WebSocket closed by client");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

async fn handle_stream_socket(
    mut socket: WebSocket,
    user: AuthUser,
    grpc_service: Arc<GrpcQueryService>,
) {
    info!("Stream WebSocket connected: user={}", user.claims.sub);

    while let Some(msg) = socket.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(val) => {
                        let sql = val.get("sql").and_then(|s| s.as_str()).unwrap_or("");
                        let batch_size = val
                            .get("batch_size")
                            .and_then(|s| s.as_u64())
                            .unwrap_or(100) as i32;

                        if sql.is_empty() {
                            let _ = socket
                                .send(Message::Text(
                                    json!({"error": "missing 'sql' field"}).to_string(),
                                ))
                                .await;
                            continue;
                        }

                        let request = tonic::Request::new(gateway_proto::StreamRequest {
                            sql: sql.to_string(),
                            batch_size,
                        });

                        match grpc_service.execute_stream_query(request).await {
                            Ok(response) => {
                                let mut stream = response.into_inner();
                                while let Some(chunk) = stream.next().await {
                                    match chunk {
                                        Ok(stream_chunk) => {
                                            let msg = json!({
                                                "rows": stream_chunk.rows.iter().map(|r| &r.values).collect::<Vec<_>>(),
                                                "is_last": stream_chunk.is_last,
                                                "query_id": stream_chunk.query_id,
                                            });
                                            if socket
                                                .send(Message::Text(msg.to_string()))
                                                .await
                                                .is_err()
                                            {
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            let _ = socket.send(Message::Text(
                                                json!({"error": format!("stream error: {}", e)}).to_string(),
                                            )).await;
                                            break;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = socket
                                    .send(Message::Text(
                                        json!({"error": format!("gRPC error: {}", e)}).to_string(),
                                    ))
                                    .await;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = socket
                            .send(Message::Text(
                                json!({"error": format!("invalid JSON: {}", e)}).to_string(),
                            ))
                            .await;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("Stream WebSocket closed by client");
                break;
            }
            Err(e) => {
                error!("Stream WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}
