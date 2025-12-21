import { z } from 'zod';
import { PAGINATION } from '@/constants/index.ts';
import { coinTypeSchema } from '@/utils/schemas.ts';

const operators = ['gt', 'gte', 'lt', 'lte'] as const;

const MAX_LIMIT = 500;

const DEFAULT_LIMIT = 100;

export const getCommitmentsQuerySchema = z
    .object({
        coin_type: coinTypeSchema,
        index: z.coerce.number().int().min(0),
        op: z.enum(operators).default('gte'),
        page: z.coerce.number().int().min(PAGINATION.MIN_PAGE).default(PAGINATION.MIN_PAGE),
        limit: z.coerce.number().int().min(1).max(MAX_LIMIT).default(DEFAULT_LIMIT),
    })
    .transform((data) => ({
        coinType: data.coin_type,
        index: data.index,
        mongoOp: `$${data.op}` as const,
        page: data.page,
        limit: data.limit,
    }));
