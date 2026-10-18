package aegis

import "time"

type HealthResponse struct {
	Status    string `json:"status"`
	Version   string `json:"version"`
	Timestamp string `json:"timestamp"`
}

type ErrorResponse struct {
	Error string `json:"error"`
	Code  int    `json:"code"`
}

type LoginRequest struct {
	Username string `json:"username"`
	Password string `json:"password"`
}

type TokenResponse struct {
	Token     string `json:"token"`
	ExpiresIn uint64 `json:"expires_in"`
}

type TrinoQuery struct {
	Query   string  `json:"query"`
	Catalog *string `json:"catalog,omitempty"`
	Schema  *string `json:"schema,omitempty"`
}

type ColumnDef struct {
	Name     string `json:"name"`
	DataType string `json:"data_type"`
}

type TrinoStatementStats struct {
	State        *string `json:"state,omitempty"`
	Queued       *bool   `json:"queued,omitempty"`
	Rows         *uint64 `json:"rows,omitempty"`
	ProcessedRows *uint64 `json:"processed_rows,omitempty"`
}

type QueryResult struct {
	Columns   []ColumnDef          `json:"columns"`
	Rows      [][]string           `json:"rows"`
	TotalRows uint64               `json:"total_rows"`
	QueryID   string               `json:"query_id"`
	Stats     *TrinoStatementStats `json:"stats,omitempty"`
}

type RateLimitStatus struct {
	Key       string `json:"key"`
	Allowed   bool   `json:"allowed"`
	Remaining uint64 `json:"remaining"`
}

type Claims struct {
	Sub      string `json:"sub"`
	Role     string `json:"role"`
	TenantID string `json:"tenant_id"`
	Exp      int64  `json:"exp"`
	Iat      int64  `json:"iat"`
}

type ClientConfig struct {
	BaseURL string
	Token   string
	Timeout time.Duration
}
