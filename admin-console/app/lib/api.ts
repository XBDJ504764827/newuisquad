const API_BASE = '/api/v1';
export const AUTH_EXPIRED_EVENT = 'auth:expired';

export function clearAuthStorage() {
    try { localStorage.removeItem('token'); } catch {}
    try { localStorage.removeItem('username'); } catch {}
    try { localStorage.removeItem('role'); } catch {}
    try { localStorage.removeItem('permissions'); } catch {}
}

function getAuthHeaders(): Record<string, string> {
    try {
        const token = localStorage.getItem('token');
        if (token) return { Authorization: `Bearer ${token}` };
    } catch {}
    return {};
}

// fetch 封装：自动添加 Authorization 头和 API_BASE 前缀
export function api(path: string, init?: RequestInit, timeoutMs = 15000): Promise<Response> {
    const headers = new Headers(init?.headers);
    const authHeaders = getAuthHeaders();
    for (const [k, v] of Object.entries(authHeaders)) {
        headers.set(k, v);
    }
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), timeoutMs);
    const signal = init?.signal || controller.signal;
    return fetch(`${API_BASE}${path}`, { ...init, headers, signal }).then(res => {
        clearTimeout(timer);
        if (res.status === 401) {
            clearAuthStorage();
            try { window.dispatchEvent(new CustomEvent(AUTH_EXPIRED_EVENT)); } catch {}
        }
        return res;
    }).catch(err => {
        clearTimeout(timer);
        if (err.name === 'AbortError') {
            throw new Error('请求超时');
        }
        throw err;
    });
}
