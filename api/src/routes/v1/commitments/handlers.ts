import type { Context } from 'hono';
import type { AppBindings, PaginatedResponse } from '@/types/index.ts';
import { COMMITMENTS_COLLECTION, type CommitmentDocument } from '@/db/collections/index.ts';
import { validateQuery } from '@/utils/validation.ts';
import { getCommitmentsQuerySchema } from './schema.ts';
import { toCommitment } from './mappers.ts';
import type { Commitment } from './types.ts';

export async function getCommitments(c: Context<AppBindings>) {
    const db = c.get('db');

    const validation = validateQuery(c, getCommitmentsQuerySchema, {
        coin_type: c.req.query('coin_type'),
        index: c.req.query('index'),
        op: c.req.query('op'),
        page: c.req.query('page'),
        limit: c.req.query('limit'),
    });

    if (!validation.success) {
        return validation.response;
    }

    const { coinType, index, mongoOp, page, limit } = validation.data;
    const skip = (page - 1) * limit;

    const collection = db.collection<CommitmentDocument>(COMMITMENTS_COLLECTION);
    const filter = { coin_type: coinType, index: { [mongoOp]: index } };

    const [commitments, total] = await Promise.all([
        collection.find(filter).sort({ index: 1 }).skip(skip).limit(limit).toArray(),
        collection.countDocuments(filter),
    ]);

    const totalPages = Math.ceil(total / limit);

    const data: PaginatedResponse<Commitment> = {
        items: commitments.map(toCommitment),
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
