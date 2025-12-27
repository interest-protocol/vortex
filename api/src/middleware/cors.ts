import { cors } from 'hono/cors';
import type { MiddlewareHandler } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { env } from '@/config/env.ts';

const parseOrigins = (origin: string): string | string[] => {
    const origins = origin.split(',').map((o) => o.trim());
    return origins.length === 1 ? (origins[0] ?? origin) : origins;
};

const createCorsMiddleware = (): MiddlewareHandler<AppBindings> => {
    const origin =
        env.NODE_ENV === 'production' && env.CORS_ORIGIN ? parseOrigins(env.CORS_ORIGIN) : '*';

    return cors({
        origin,
        allowMethods: ['GET', 'POST', 'OPTIONS'],
        allowHeaders: ['Content-Type', 'Authorization', 'x-api-key'],
        exposeHeaders: ['Content-Length'],
        maxAge: 86400,
        credentials: origin !== '*',
    });
};

export const corsMiddleware = createCorsMiddleware();
