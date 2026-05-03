const API_BASE = '/api/v1';

function getAuthHeaders(): Record<string, string> {
    try {
        const token = localStorage.getItem('token');
        if (token) return { Authorization: `Bearer ${token}` };
    } catch {}
    return {};
}

// fetch 封装：自动添加 Authorization 头和 API_BASE 前缀
export function api(path: string, init?: RequestInit): Promise<Response> {
    const headers = new Headers(init?.headers);
    const authHeaders = getAuthHeaders();
    for (const [k, v] of Object.entries(authHeaders)) {
        headers.set(k, v);
    }
    return fetch(`${API_BASE}${path}`, { ...init, headers });
}
