import { isValidSuiAddress } from '@mysten/sui/utils';
import { z } from 'zod';

export const suiAddressSchema = z.string().refine(isValidSuiAddress, 'Invalid Sui address');

export const poseidonHashSchema = z
    .string()
    .regex(/^[0-9]+$/)
    .max(80);

export const coinTypeSchema = z.string().regex(/^0x[a-fA-F0-9]+::\w+::\w+$/);

export const indexSchema = z.coerce.number().int().min(0);
