import { useState, useEffect } from 'react';
import { api } from './api';

export interface Server {
    id: number;
    server_id: string;
    name: string;
    ip: string;
    rcon_port: number;
    created_at: string;
    token?: string;
}

export function useServers() {
    const [servers, setServers] = useState<Server[]>([]);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        api('/servers')
            .then(r => r.json())
            .then(data => { setServers(data.data || []); setLoading(false); })
            .catch(() => setLoading(false));
    }, []);

    return { servers, loading };
}
