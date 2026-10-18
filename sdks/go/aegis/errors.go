package aegis

import "fmt"

type APIError struct {
	Message    string
	Code       int
	StatusCode int
}

func (e *APIError) Error() string {
	return fmt.Sprintf("aegis api error %d: %s", e.StatusCode, e.Message)
}
