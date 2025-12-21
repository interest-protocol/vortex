import type { Context } from 'hono';
import type { AppBindings, PaginatedResponse } from '@/types/index.ts';
import { validateQuery } from '@/utils/validation.ts';
import { withErrorHandler } from '@/utils/handler.ts';
import { getCommitmentsQuerySchema } from './schema.ts';
import { toCommitment } from './mappers.ts';
import type { Commitment } from './types.ts';

const getCommitmentsHandler = async (c: Context<AppBindings>) => {
    const validation = validateQuery(c, getCommitmentsQuerySchema);
    if (!validation.success) return validation.response;

    const commitments = c.get('commitments');
    const { coinType, index, mongoOp, page, limit } = validation.data;
    const skip = (page - 1) * limit;
    const filter = { coin_type: coinType, index: { [mongoOp]: index } };

    const [docs, total] = await Promise.all([
        commitments.find({ filter, skip, limit }),
        commitments.count(filter),
    ]);

    const totalPages = Math.ceil(total / limit);

    const data: PaginatedResponse<Commitment> = {
        items: docs.map(toCommitment),
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

export const getCommitments = withErrorHandler(
    getCommitmentsHandler,
    'Failed to fetch commitments'
);
