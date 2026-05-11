import type { NextConfig } from "next";

const backendUrl = process.env.BACKEND_URL || 'http://127.0.0.1:8744';

const nextConfig: NextConfig = {
  output: 'standalone',
  async rewrites() {
    return [
      { source: '/api/:path*', destination: `${backendUrl}/api/:path*` },
      { source: '/agent/:path*', destination: `${backendUrl}/agent/:path*` },
    ];
  },
};

export default nextConfig;
