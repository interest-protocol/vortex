import { z } from 'zod';
import {
    suiAddressSchema,
    poseidonHashSchema,
    coinTypeSchema,
    indexSchema,
} from '@/utils/schemas.ts';

export const getMerklePathBodySchema = z
    .object({
        coin_type: coinTypeSchema,
        index: indexSchema,
        amount: poseidonHashSchema,
        public_key: poseidonHashSchema,
        blinding: poseidonHashSchema,
        vortex_pool: suiAddressSchema,
    })
    .transform((data) => ({
        coinType: data.coin_type,
        index: data.index,
        amount: BigInt(data.amount),
        publicKey: data.public_key,
        blinding: BigInt(data.blinding),
        vortexPool: data.vortex_pool,
    }));
