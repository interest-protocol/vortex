import { z } from 'zod';
import { PAGINATION } from '@/constants/index.ts';
import { coinTypeSchema, indexSchema } from '@/utils/schemas.ts';

const operators = ['gt', 'gte', 'lt', 'lte'] as const;

export const getCommitmentsQuerySchema = z
    .object({
        coin_type: coinTypeSchema,
        index: indexSchema,
        op: z.enum(operators).default('gte'),
        limit: z.coerce
            .number()
            .int()
            .min(1)
            .max(PAGINATION.MAX_LIMIT)
            .default(PAGINATION.DEFAULT_LIMIT),
    })
    .transform((data) => ({
        coinType: data.coin_type,
        index: data.index,
        mongoOp: `$${data.op}` as const,
        limit: data.limit,
    }));
