import type { Context } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { validateBody } from '@/utils/validation.ts';
import { getMerklePathBodySchema } from './schema.ts';

export const getMerklePathHandler = async (c: Context<AppBindings>) => {
    const merkleService = c.get('merkleService');

    const validated = await validateBody(c, getMerklePathBodySchema);
    if (!validated.success) return validated.response;

    const { coinType, index, amount, publicKey, blinding, vortexPool } = validated.data;
    const utxo = { amount, publicKey, blinding, vortexPool };
    const data = await merkleService.getMerklePath({ coinType, index, utxo });

    return c.json({ success: true, data });
};
