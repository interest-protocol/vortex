import type { MiddlewareHandler } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { env } from '@/config/env.ts';

export const apiKeyMiddleware: MiddlewareHandler<AppBindings> = async (c, next) => {
    if (env.NODE_ENV !== 'production' && !env.API_KEY) {
        return next();
    }

    const apiKey = c.req.header('x-api-key');

    if (!apiKey || apiKey !== env.API_KEY) {
        return c.json({ success: false, error: 'Invalid or missing API key' }, 401);
    }

    return next();
};
