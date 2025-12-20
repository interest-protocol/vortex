import { Hono } from 'hono';
import type { AppBindings, PaginatedResponse } from '@/types/index.js';
import { COLLECTIONS } from '@/constants/index.js';
import { poolsQuerySchema } from './schema.js';
import type { Pool, PoolFilter } from './types.js';

export const poolsRoutes = new Hono<AppBindings>().get('/', async (c) => {
    const db = c.get('db');

    const queryParams = poolsQuerySchema.safeParse({
        page: c.req.query('page'),
        limit: c.req.query('limit'),
        coin_type: c.req.query('coin_type'),
    });

    if (!queryParams.success) {
        return c.json(
            {
                success: false,
                error: queryParams.error.flatten().fieldErrors,
            },
            400
        );
    }

    const { page, limit, coin_type } = queryParams.data;
    const skip = (page - 1) * limit;

    const filter: PoolFilter = {};
    if (coin_type) {
        filter.coin_type = coin_type;
    }

    const collection = db.collection<Pool>(COLLECTIONS.NEW_POOLS);

    const [pools, total] = await Promise.all([
        collection.find(filter).sort({ checkpoint: -1 }).skip(skip).limit(limit).toArray(),
        collection.countDocuments(filter),
    ]);

    const totalPages = Math.ceil(total / limit);

    const data: PaginatedResponse<Pool> = {
        items: pools,
        pagination: {
            page,
            limit,
            total,
            totalPages,
            hasNext: page < totalPages,
            hasPrev: page > 1,
        },
    };

    return c.json({ success: true, data });
});
