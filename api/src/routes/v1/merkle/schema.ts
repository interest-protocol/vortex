import { z } from 'zod';

export const getMerklePathBodySchema = z
    .object({
        coin_type: z.string().regex(/^0x[a-fA-F0-9]+::\w+::\w+$/),
        amount: z.string(),
        blinding: z.string(),
        private_key: z.string(),
        index: z.string(),
        vortex_pool: z.string().regex(/^0x[a-fA-F0-9]+$/),
    })
    .transform((data) => ({
        coinType: data.coin_type,
        amount: BigInt(data.amount),
        blinding: BigInt(data.blinding),
        privateKey: BigInt(data.private_key),
        index: BigInt(data.index),
        vortexPool: data.vortex_pool,
    }));

export type GetMerklePathBody = z.infer<typeof getMerklePathBodySchema>;
