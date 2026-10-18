from __future__ import annotations

from dataclasses import asdict
from typing import Any, Optional

import httpx

from .types import (
    Claims,
    ColumnDef,
    ErrorResponse,
    HealthResponse,
    LoginRequest,
    QueryResult,
    RateLimitStatus,
    TokenResponse,
    TrinoQuery,
    TrinoStatementStats,
)


class AegisError(Exception):
    def __init__(self, message: str, code: int, status_code: int):
        super().__init__(message)
        self.code = code
        self.status_code = status_code


class AegisClient:
    def __init__(
        self,
        base_url: str,
        token: Optional[str] = None,
        timeout: float = 30.0,
    ):
        self.base_url = base_url.rstrip("/")
        self._token = token
        self._client = httpx.Client(timeout=timeout)

    def set_token(self, token: str) -> None:
        self._token = token

    def _request(
        self, method: str, path: str, body: Optional[Any] = None
    ) -> Any:
        headers = {"Content-Type": "application/json"}
        if self._token:
            headers["Authorization"] = f"Bearer {self._token}"

        response = self._client.request(
            method,
            f"{self.base_url}{path}",
            headers=headers,
            json=body,
        )

        if response.status_code >= 400:
            try:
                err = ErrorResponse(**response.json())
                raise AegisError(err.error, err.code, response.status_code)
            except (KeyError, ValueError):
                raise AegisError(
                    response.text, response.status_code, response.status_code
                )

        return response.json()

    def health(self) -> HealthResponse:
        data = self._request("GET", "/health")
        return HealthResponse(**data)

    def login(self, username: str, password: str) -> TokenResponse:
        data = self._request("POST", "/auth/login", {"username": username, "password": password})
        result = TokenResponse(**data)
        self._token = result.token
        return result

    def query(self, q: TrinoQuery) -> QueryResult:
        data = self._request("POST", "/query", asdict(q))
        stats = None
        if data.get("stats"):
            stats = TrinoStatementStats(**data["stats"])
        return QueryResult(
            columns=[ColumnDef(**c) for c in data["columns"]],
            rows=data["rows"],
            total_rows=data["total_rows"],
            query_id=data["query_id"],
            stats=stats,
        )

    def rate_limit_status(self) -> RateLimitStatus:
        data = self._request("GET", "/rate-limit/status")
        return RateLimitStatus(**data)

    def close(self) -> None:
        self._client.close()
