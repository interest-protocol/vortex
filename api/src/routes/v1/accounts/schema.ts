import { z } from 'zod';
import { suiAddressSchema, poseidonHashSchema } from '@/utils/schemas.ts';

export const createAccountSchema = z.object({
    owner: suiAddressSchema,
    hashedSecret: poseidonHashSchema,
});

export const getAccountsQuerySchema = z.object({
    hashed_secret: poseidonHashSchema,
});
