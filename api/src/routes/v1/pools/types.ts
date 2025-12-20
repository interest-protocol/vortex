export type Pool = {
    _id: string;
    digest: string;
    sender: string;
    checkpoint: number;
    checkpoint_timestamp_ms: number;
    pool_address: string;
    coin_type: string;
};

export type PoolFilter = {
    coin_type?: string;
};
