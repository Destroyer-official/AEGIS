import {
  HealthResponse,
  LoginRequest,
  TokenResponse,
  TrinoQuery,
  QueryResult,
  RateLimitStatus,
} from "./types";

export interface ClientConfig {
  baseUrl: string;
  token?: string;
  timeout?: number;
}

export class AegisClient {
  private baseUrl: string;
  private token: string | null;
  private timeout: number;

  constructor(config: ClientConfig) {
    this.baseUrl = config.baseUrl.replace(/\/$/, "");
    this.token = config.token ?? null;
    this.timeout = config.timeout ?? 30000;
  }

  setToken(token: string): void {
    this.token = token;
  }

  private async request<T>(
    method: string,
    path: string,
    body?: unknown
  ): Promise<T> {
    const headers: Record<string, string> = {
      "Content-Type": "application/json",
    };
    if (this.token) {
      headers["Authorization"] = `Bearer ${this.token}`;
    }

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await fetch(`${this.baseUrl}${path}`, {
        method,
        headers,
        body: body ? JSON.stringify(body) : undefined,
        signal: controller.signal,
      });

      if (!response.ok) {
        const error = await response.json().catch(() => ({
          error: response.statusText,
          code: response.status,
        }));
        throw new AegisError(error.error, error.code, response.status);
      }

      return response.json();
    } finally {
      clearTimeout(timeoutId);
    }
  }

  async health(): Promise<HealthResponse> {
    return this.request<HealthResponse>("GET", "/health");
  }

  async login(username: string, password: string): Promise<TokenResponse> {
    const result = await this.request<TokenResponse>("POST", "/auth/login", {
      username,
      password,
    });
    this.token = result.token;
    return result;
  }

  async query(trinoQuery: TrinoQuery): Promise<QueryResult> {
    return this.request<QueryResult>("POST", "/query", trinoQuery);
  }

  async rateLimitStatus(): Promise<RateLimitStatus> {
    return this.request<RateLimitStatus>("GET", "/rate-limit/status");
  }

  async adminAction(): Promise<{ status: string }> {
    return this.request<{ status: string }>("POST", "/admin/action");
  }

  async manageUsers(): Promise<{ status: string }> {
    return this.request<{ status: string }>("POST", "/admin/users");
  }
}

export class AegisError extends Error {
  public code: number;
  public statusCode: number;

  constructor(message: string, code: number, statusCode: number) {
    super(message);
    this.name = "AegisError";
    this.code = code;
    this.statusCode = statusCode;
  }
}

export { AegisClient as default };
