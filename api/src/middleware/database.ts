import type { MiddlewareHandler } from 'hono';
import type { AppBindings } from '../types/index.js';
import { getDb } from '../db/mongodb.js';
import { getRedis } from '../db/redis.js';

export const databaseMiddleware: MiddlewareHandler<AppBindings> = async (c, next) => {
    c.set('db', getDb());
    c.set('redis', getRedis());
    await next();
};
