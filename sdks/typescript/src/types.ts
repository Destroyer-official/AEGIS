export interface HealthResponse {
  status: string;
  version: string;
  timestamp: string;
}

export interface ErrorResponse {
  error: string;
  code: number;
}

export interface LoginRequest {
  username: string;
  password: string;
}

export interface TokenResponse {
  token: string;
  expires_in: number;
}

export interface TrinoQuery {
  query: string;
  catalog?: string;
  schema?: string;
}

export interface ColumnDef {
  name: string;
  data_type: string;
}

export interface TrinoStatementStats {
  state?: string;
  queued?: boolean;
  rows?: number;
  processed_rows?: number;
}

export interface QueryResult {
  columns: ColumnDef[];
  rows: string[][];
  total_rows: number;
  query_id: string;
  stats?: TrinoStatementStats;
}

export interface RateLimitStatus {
  key: string;
  allowed: boolean;
  remaining: number;
}

export interface Claims {
  sub: string;
  role: "admin" | "analyst" | "viewer";
  tenant_id: string;
  exp: number;
  iat: number;
}
