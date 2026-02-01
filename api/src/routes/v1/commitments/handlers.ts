import type { Context } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { validateQuery } from '@/utils/validation.ts';
import { withErrorHandler } from '@/utils/handler.ts';
import { getCommitmentsQuerySchema } from './schema.ts';
import { toCommitment } from './mappers.ts';

const getCommitmentsHandler = async (c: Context<AppBindings>) => {
    const validation = validateQuery(c, getCommitmentsQuerySchema);
    if (!validation.success) return validation.response;

    const commitments = c.get('commitments');
    const { coinType, index, mongoOp, limit } = validation.data;
    const filter = { coin_type: coinType, index: { [mongoOp]: index } };

    const docs = await commitments.find({ filter, skip: 0, limit: limit + 1 });
    const hasNext = docs.length > limit;
    const items = (hasNext ? docs.slice(0, limit) : docs).map(toCommitment);

    return c.json({ success: true, data: { items, hasNext } });
};

export const getCommitments = withErrorHandler(
    getCommitmentsHandler,
    'Failed to fetch commitments'
);
