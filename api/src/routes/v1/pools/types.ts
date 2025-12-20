export type PoolDocument = {
    _id: string;
    digest: string;
    sender: string;
    checkpoint: number;
    checkpoint_timestamp_ms: number;
    pool_address: string;
    coin_type: string;
};

export type Pool = {
    id: string;
    digest: string;
    sender: string;
    checkpoint: number;
    checkpointTimestampMs: number;
    poolAddress: string;
    coinType: string;
};

export type PoolFilter = {
    coin_type?: string;
};
