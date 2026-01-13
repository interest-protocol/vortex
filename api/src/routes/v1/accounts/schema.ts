import { z } from 'zod';
import { suiAddressSchema, suiObjectIdSchema, poseidonHashSchema } from '@/utils/schemas.ts';

export const createAccountSchema = z.object({
    owner: suiAddressSchema,
    hashedSecret: poseidonHashSchema,
});

export const getAccountsQuerySchema = z.object({
    hashed_secret: poseidonHashSchema,
    exclude_hidden: z
        .enum(['true', 'false'])
        .transform((v) => v === 'true')
        .optional(),
});

export const hideAccountsSchema = z
    .object({
        accountObjectIds: z.array(suiObjectIdSchema).optional(),
        hashedSecret: poseidonHashSchema.optional(),
    })
    .refine((data) => (data.accountObjectIds?.length ?? 0) > 0 || data.hashedSecret, {
        message: 'At least one of accountObjectIds or hashedSecret must be provided',
    });
