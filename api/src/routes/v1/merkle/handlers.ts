import type { Context } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { validateBody } from '@/utils/validation.ts';
import { withErrorHandler } from '@/utils/handler.ts';
import { getMerklePathBodySchema } from './schema.ts';

const getMerklePathHandlerInternal = async (c: Context<AppBindings>) => {
    const validated = await validateBody(c, getMerklePathBodySchema);
    if (!validated.success) return validated.response;

    const merkleService = c.get('merkleService');
    const { coinType, index, amount, publicKey, blinding, vortexPool } = validated.data;
    const utxo = { amount, publicKey, blinding, vortexPool };
    const data = await merkleService.getMerklePath({ coinType, index, utxo });

    return c.json({ success: true, data });
};

export const getMerklePathHandler = withErrorHandler(
    getMerklePathHandlerInternal,
    'Failed to get merkle path'
);
