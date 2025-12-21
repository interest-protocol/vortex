import type { Context } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { buildPaginatedResponse } from '@/types/index.ts';
import { validateQuery } from '@/utils/validation.ts';
import { withErrorHandler } from '@/utils/handler.ts';
import { poolsQuerySchema } from './schema.ts';
import { toPool } from './mappers.ts';

const getPoolsHandler = async (c: Context<AppBindings>) => {
    const validation = validateQuery(c, poolsQuerySchema);
    if (!validation.success) return validation.response;

    const pools = c.get('pools');
    const { page, limit, coin_type } = validation.data;
    const skip = (page - 1) * limit;
    const filter = coin_type ? { coin_type } : {};

    const [docs, total] = await Promise.all([
        pools.find({ filter, skip, limit }),
        pools.count(filter),
    ]);

    const data = buildPaginatedResponse(docs, toPool, { page, limit, total });

    return c.json({ success: true, data });
};

export const getPools = withErrorHandler(getPoolsHandler, 'Failed to fetch pools');
