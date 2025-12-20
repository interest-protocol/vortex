export type { PoolDocument } from '@/db/collections/index.ts';

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
