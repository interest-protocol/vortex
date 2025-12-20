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

export type Pagination = {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
};

export type PaginatedResponse<T> = {
    items: T[];
    pagination: Pagination;
};
