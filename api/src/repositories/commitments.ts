import type { Db } from 'mongodb';
import { COMMITMENTS_COLLECTION, type CommitmentDocument } from '@/db/collections/index.ts';

export type CommitmentFilter = {
    coin_type: string;
    index: Record<string, number>;
};

export type CommitmentsRepository = {
    find: (params: {
        filter: CommitmentFilter;
        skip: number;
        limit: number;
    }) => Promise<CommitmentDocument[]>;
    count: (filter: CommitmentFilter) => Promise<number>;
    findFromIndex: (coinType: string, fromIndex: number) => Promise<CommitmentDocument[]>;
};

export const createCommitmentsRepository = (db: Db): CommitmentsRepository => {
    const collection = db.collection<CommitmentDocument>(COMMITMENTS_COLLECTION);

    return {
        find: async ({ filter, skip, limit }) =>
            collection.find(filter).sort({ index: 1 }).skip(skip).limit(limit).toArray(),

        count: async (filter) => collection.countDocuments(filter),

        findFromIndex: async (coinType, fromIndex) =>
            collection
                .find({ coin_type: coinType, index: { $gte: fromIndex } })
                .sort({ index: 1 })
                .toArray(),
    };
};
