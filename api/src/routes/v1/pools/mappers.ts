import type { Pool, PoolDocument } from './types.js';

export function toPool(doc: PoolDocument): Pool {
    return {
        id: doc._id,
        digest: doc.digest,
        sender: doc.sender,
        checkpoint: doc.checkpoint,
        checkpointTimestampMs: doc.checkpoint_timestamp_ms,
        poolAddress: doc.pool_address,
        coinType: doc.coin_type,
    };
}
