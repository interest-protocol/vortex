export const POOLS_COLLECTION = 'new_pools';

export type PoolDocument = {
    _id: string;
    digest: string;
    sender: string;
    checkpoint: number;
    checkpoint_timestamp_ms: number;
    pool_address: string;
    coin_type: string;
};
