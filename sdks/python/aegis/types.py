from __future__ import annotations

from dataclasses import dataclass, field
from typing import Optional


@dataclass
class HealthResponse:
    status: str
    version: str
    timestamp: str


@dataclass
class ErrorResponse:
    error: str
    code: int


@dataclass
class LoginRequest:
    username: str
    password: str


@dataclass
class TokenResponse:
    token: str
    expires_in: int


@dataclass
class TrinoQuery:
    query: str
    catalog: Optional[str] = None
    schema: Optional[str] = None


@dataclass
class ColumnDef:
    name: str
    data_type: str


@dataclass
class TrinoStatementStats:
    state: Optional[str] = None
    queued: Optional[bool] = None
    rows: Optional[int] = None
    processed_rows: Optional[int] = None


@dataclass
class QueryResult:
    columns: list[ColumnDef] = field(default_factory=list)
    rows: list[list[str]] = field(default_factory=list)
    total_rows: int = 0
    query_id: str = ""
    stats: Optional[TrinoStatementStats] = None


@dataclass
class RateLimitStatus:
    key: str = ""
    allowed: bool = False
    remaining: int = 0


@dataclass
class Claims:
    sub: str = ""
    role: str = ""
    tenant_id: str = ""
    exp: int = 0
    iat: int = 0
