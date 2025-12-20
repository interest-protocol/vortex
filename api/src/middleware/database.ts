import type { MiddlewareHandler } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { getDb } from '@/db/mongodb.ts';
import { getRedis } from '@/db/redis.ts';

export const databaseMiddleware: MiddlewareHandler<AppBindings> = async (c, next) => {
    c.set('db', getDb());
    c.set('redis', getRedis());
    await next();
};
