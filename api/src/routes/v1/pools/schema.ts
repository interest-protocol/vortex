import { z } from 'zod';
import { PAGINATION } from '@/constants/index.js';

export const poolsQuerySchema = z.object({
    page: z.coerce.number().int().min(PAGINATION.MIN_PAGE).default(PAGINATION.MIN_PAGE),
    limit: z.coerce
        .number()
        .int()
        .min(1)
        .max(PAGINATION.MAX_LIMIT)
        .default(PAGINATION.DEFAULT_LIMIT),
    coin_type: z.string().optional(),
});

export type PoolsQuery = z.infer<typeof poolsQuerySchema>;
