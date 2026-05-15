import { createContext, useCallback, useContext, useEffect, useState } from "react";

export type User = {
  id: string;
  email: string;
  name: string | null;
  role: "admin" | "editor" | "viewer" | string;
  status: "active" | "pending" | "suspended" | string;
};

type AuthState = {
  user: User | null;
  loading: boolean;
  refresh: () => Promise<void>;
  logout: () => Promise<void>;
};

const AuthContext = createContext<AuthState>({
  user: null,
  loading: true,
  refresh: async () => {},
  logout: async () => {},
});

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const res = await fetch("/api/auth/me");
      setUser(res.ok ? ((await res.json()) as User) : null);
    } catch {
      setUser(null);
    }
  }, []);

  useEffect(() => {
    void refresh().then(() => setLoading(false));
  }, [refresh]);

  async function logout() {
    await fetch("/api/auth/logout", { method: "POST" });
    setUser(null);
  }

  return (
    <AuthContext.Provider value={{ user, loading, refresh, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  return useContext(AuthContext);
}
