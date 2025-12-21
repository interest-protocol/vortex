import { cors } from 'hono/cors';
import type { MiddlewareHandler } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { env } from '@/config/env.ts';

const parseOrigins = (origin: string): string | string[] => {
    const origins = origin.split(',').map((o) => o.trim());
    return origins.length === 1 ? (origins[0] ?? origin) : origins;
};

const createCorsMiddleware = (): MiddlewareHandler<AppBindings> => {
    if (env.NODE_ENV !== 'production' || !env.CORS_ORIGIN) {
        return cors();
    }

    return cors({
        origin: parseOrigins(env.CORS_ORIGIN),
        allowMethods: ['GET', 'POST', 'OPTIONS'],
        allowHeaders: ['Content-Type', 'Authorization'],
        exposeHeaders: ['Content-Length'],
        maxAge: 86400,
        credentials: true,
    });
};

export const corsMiddleware = createCorsMiddleware();
