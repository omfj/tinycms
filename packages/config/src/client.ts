type NamedParams = Record<string, string | number | boolean>;
type PositionalParams = Array<string | number | boolean>;

export interface QueryOptions {
  params?: NamedParams | PositionalParams;
}

export interface TinyCmsClientOptions {
  baseUrl: string;
  token?: string;
}

export class TinyCmsClient {
  private baseUrl: string;
  private token?: string;

  constructor(options: TinyCmsClientOptions) {
    this.baseUrl = options.baseUrl.replace(/\/$/, "");
    this.token = options.token;
  }

  async query<TData = Record<string, unknown>>(
    sql: string,
    options: QueryOptions = {},
  ): Promise<TData[]> {
    const res = await fetch(`${this.baseUrl}/api/query`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer " + this.token,
      },
      body: JSON.stringify({ q: sql, params: options.params }),
    });

    if (!res.ok) {
      const body = await res.json().catch(() => ({}));
      throw new Error(
        (body as { error?: string }).error ?? `query failed: ${res.status}`,
      );
    }

    return res.json() as Promise<TData[]>;
  }
}
