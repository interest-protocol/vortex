import type { Context } from 'hono';
import type { AppBindings, PaginatedResponse } from '@/types/index.ts';
import { POOLS_COLLECTION, type PoolDocument } from '@/db/collections/index.ts';
import { validateQuery } from '@/utils/validation.ts';
import { poolsQuerySchema } from './schema.ts';
import { toPool } from './mappers.ts';
import type { Pool, PoolFilter } from './types.ts';

export async function getPools(c: Context<AppBindings>) {
    const db = c.get('db');

    const validation = validateQuery(c, poolsQuerySchema, {
        page: c.req.query('page'),
        limit: c.req.query('limit'),
        coin_type: c.req.query('coin_type'),
    });

    if (!validation.success) {
        return validation.response;
    }

    const { page, limit, coin_type } = validation.data;
    const skip = (page - 1) * limit;

    const filter: PoolFilter = {};
    if (coin_type) {
        filter.coin_type = coin_type;
    }

    const collection = db.collection<PoolDocument>(POOLS_COLLECTION);

    const [poolDocs, total] = await Promise.all([
        collection.find(filter).sort({ checkpoint: -1 }).skip(skip).limit(limit).toArray(),
        collection.countDocuments(filter),
    ]);

    const totalPages = Math.ceil(total / limit);

    const data: PaginatedResponse<Pool> = {
        items: poolDocs.map(toPool),
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
}
