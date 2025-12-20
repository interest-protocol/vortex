import type { Context } from 'hono';
import type { Db } from 'mongodb';
import type { Redis } from 'ioredis';

export type AppBindings = {
    Variables: {
        db: Db;
        redis: Redis;
    };
};

export type AppContext = Context<AppBindings>;

export type ApiResponse<T> = { success: true; data: T } | { success: false; error: string };
