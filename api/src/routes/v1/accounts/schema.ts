import { isValidSuiAddress } from '@mysten/sui/utils';
import { z } from 'zod';

const suiAddressSchema = z.string().refine(isValidSuiAddress, 'Invalid Sui address');

const poseidonHashSchema = z
    .string()
    .regex(/^[0-9]+$/)
    .max(80);

export const createAccountSchema = z.object({
    owner: suiAddressSchema,
    hashedSecret: poseidonHashSchema,
});

export const getAccountsQuerySchema = z.object({
    hashed_secret: poseidonHashSchema,
});
