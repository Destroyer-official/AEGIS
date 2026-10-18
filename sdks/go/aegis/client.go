package aegis

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

type Client struct {
	baseURL    string
	token      string
	httpClient *http.Client
}

func NewClient(config ClientConfig) *Client {
	timeout := config.Timeout
	if timeout == 0 {
		timeout = 30 * time.Second
	}
	return &Client{
		baseURL: config.BaseURL,
		token:   config.Token,
		httpClient: &http.Client{
			Timeout: timeout,
		},
	}
}

func (c *Client) SetToken(token string) {
	c.token = token
}

func (c *Client) request(method, path string, body interface{}) ([]byte, error) {
	var bodyReader io.Reader
	if body != nil {
		data, err := json.Marshal(body)
		if err != nil {
			return nil, fmt.Errorf("marshal body: %w", err)
		}
		bodyReader = bytes.NewReader(data)
	}

	req, err := http.NewRequest(method, c.baseURL+path, bodyReader)
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}

	req.Header.Set("Content-Type", "application/json")
	if c.token != "" {
		req.Header.Set("Authorization", "Bearer "+c.token)
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("execute request: %w", err)
	}
	defer resp.Body.Close()

	data, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read response: %w", err)
	}

	if resp.StatusCode >= 400 {
		var errResp ErrorResponse
		if json.Unmarshal(data, &errResp) == nil {
			return nil, &APIError{
				Message:    errResp.Error,
				Code:       errResp.Code,
				StatusCode: resp.StatusCode,
			}
		}
		return nil, &APIError{
			Message:    resp.Status,
			Code:       resp.StatusCode,
			StatusCode: resp.StatusCode,
		}
	}

	return data, nil
}

func (c *Client) Health() (*HealthResponse, error) {
	data, err := c.request("GET", "/health", nil)
	if err != nil {
		return nil, err
	}
	var result HealthResponse
	if err := json.Unmarshal(data, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

func (c *Client) Login(username, password string) (*TokenResponse, error) {
	data, err := c.request("POST", "/auth/login", &LoginRequest{
		Username: username,
		Password: password,
	})
	if err != nil {
		return nil, err
	}
	var result TokenResponse
	if err := json.Unmarshal(data, &result); err != nil {
		return nil, err
	}
	c.token = result.Token
	return &result, nil
}

func (c *Client) Query(q *TrinoQuery) (*QueryResult, error) {
	data, err := c.request("POST", "/query", q)
	if err != nil {
		return nil, err
	}
	var result QueryResult
	if err := json.Unmarshal(data, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

func (c *Client) RateLimitStatus() (*RateLimitStatus, error) {
	data, err := c.request("GET", "/rate-limit/status", nil)
	if err != nil {
		return nil, err
	}
	var result RateLimitStatus
	if err := json.Unmarshal(data, &result); err != nil {
		return nil, err
	}
	return &result, nil
}
