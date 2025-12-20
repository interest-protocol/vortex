import type { Context } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { getMerklePath } from '@/services/merkle-tree.ts';
import { getMerklePathBodySchema } from './schema.ts';
import type { MerklePathResponse } from './types.ts';

export const getMerklePathHandler = async (c: Context<AppBindings>) => {
    const db = c.get('db');
    const redis = c.get('redis');

    const body: unknown = await c.req.json();
    const parsed = getMerklePathBodySchema.safeParse(body);

    if (!parsed.success) {
        return c.json({ success: false, error: parsed.error.flatten() }, 400);
    }

    const { coinType, index, amount, publicKey, blinding, vortexPool } = parsed.data;

    const utxo = { amount, publicKey, blinding, vortexPool };

    const result = await getMerklePath({ db, redis, coinType, index, utxo });

    const data: MerklePathResponse = result;

    return c.json({ success: true, data });
};
