import { z } from 'zod';

export const getMerklePathBodySchema = z
    .object({
        coin_type: z.string().regex(/^0x[a-fA-F0-9]+::\w+::\w+$/),
        index: z.coerce.number().int().min(0),
        amount: z.string(),
        public_key: z.string(),
        blinding: z.string(),
        vortex_pool: z.string().regex(/^0x[a-fA-F0-9]+$/),
    })
    .transform((data) => ({
        coinType: data.coin_type,
        index: data.index,
        amount: BigInt(data.amount),
        publicKey: data.public_key,
        blinding: BigInt(data.blinding),
        vortexPool: data.vortex_pool,
    }));
