import { z } from 'zod';

export const createAccountSchema = z.object({
    owner: z.string().min(1),
    hashedSecret: z.string().min(1),
});

export const getAccountsQuerySchema = z.object({
    hashed_secret: z.string().min(1),
});
