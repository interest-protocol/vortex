import { normalizeSuiObjectId, normalizeStructTag } from '@mysten/sui/utils';

import type { PoolDocument } from '@/db/collections/index.ts';
import type { Pool } from './types.ts';

export const toPool = (doc: PoolDocument): Pool => ({
    id: doc._id,
    digest: doc.digest,
    sender: doc.sender,
    checkpoint: doc.checkpoint,
    checkpointTimestampMs: doc.checkpoint_timestamp_ms,
    objectId: normalizeSuiObjectId(doc.pool_address),
    coinType: normalizeStructTag(doc.coin_type),
});
