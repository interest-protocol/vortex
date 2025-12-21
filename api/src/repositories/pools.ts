import type { Db } from 'mongodb';
import { POOLS_COLLECTION, type PoolDocument } from '@/db/collections/index.ts';

export type PoolFilter = {
    coin_type?: string;
};

export type PoolsRepository = {
    find: (params: { filter: PoolFilter; skip: number; limit: number }) => Promise<PoolDocument[]>;
    count: (filter: PoolFilter) => Promise<number>;
};

export const createPoolsRepository = (db: Db): PoolsRepository => {
    const collection = db.collection<PoolDocument>(POOLS_COLLECTION);

    return {
        find: async ({ filter, skip, limit }) =>
            collection.find(filter).sort({ checkpoint: -1 }).skip(skip).limit(limit).toArray(),

        count: async (filter) => collection.countDocuments(filter),
    };
};
