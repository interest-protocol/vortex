import type { Context } from 'hono';
import type { AppBindings, PaginatedResponse } from '@/types/index.ts';
import { validateQuery } from '@/utils/validation.ts';
import { poolsQuerySchema } from './schema.ts';
import { toPool } from './mappers.ts';
import type { Pool } from './types.ts';

export const getPools = async (c: Context<AppBindings>) => {
    const pools = c.get('pools');

    const validation = validateQuery(c, poolsQuerySchema);
    if (!validation.success) return validation.response;

    const { page, limit, coin_type } = validation.data;
    const skip = (page - 1) * limit;
    const filter = coin_type ? { coin_type } : {};

    const [poolDocs, total] = await Promise.all([
        pools.find({ filter, skip, limit }),
        pools.count(filter),
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
};
