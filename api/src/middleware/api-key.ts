import { timingSafeEqual } from 'node:crypto';

import type { MiddlewareHandler } from 'hono';

import type { AppBindings } from '@/types/index.ts';
import { env } from '@/config/env.ts';

const safeCompare = (a: string, b: string): boolean => {
    const bufA = Buffer.from(a);
    const bufB = Buffer.from(b);
    if (bufA.length !== bufB.length) {
        timingSafeEqual(bufA, bufA);
        return false;
    }
    return timingSafeEqual(bufA, bufB);
};

export const apiKeyMiddleware: MiddlewareHandler<AppBindings> = async (c, next) => {
    const expectedKey = env.API_KEY;

    if (env.NODE_ENV !== 'production' && !expectedKey) {
        return next();
    }

    const apiKey = c.req.header('x-api-key');

    if (!apiKey || !expectedKey || !safeCompare(apiKey, expectedKey)) {
        return c.json({ success: false, error: 'Invalid or missing API key' }, 401);
    }

    return next();
};
