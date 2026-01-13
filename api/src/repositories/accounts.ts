import type { Db, Filter } from 'mongodb';
import { ACCOUNTS_COLLECTION, type AccountDocument } from '@/db/collections/index.ts';

export type FindAccountsParams = {
    hashedSecret: string;
    excludeHidden?: boolean | undefined;
};

export type HideAccountsParams = {
    accountObjectIds?: string[] | undefined;
    hashedSecret?: string | undefined;
};

export type AccountsRepository = {
    findByHashedSecret: (params: FindAccountsParams) => Promise<AccountDocument[]>;
    insert: (doc: AccountDocument) => Promise<void>;
    hideMany: (params: HideAccountsParams) => Promise<number>;
};

export const createAccountsRepository = (db: Db): AccountsRepository => {
    const collection = db.collection<AccountDocument>(ACCOUNTS_COLLECTION);

    return {
        findByHashedSecret: async ({ hashedSecret, excludeHidden }) => {
            const filter: Filter<AccountDocument> = { hashed_secret: hashedSecret };
            if (excludeHidden) {
                filter.hidden = { $ne: true };
            }
            return collection.find(filter).toArray();
        },

        insert: async (doc) => {
            await collection.insertOne(doc);
        },

        hideMany: async ({ accountObjectIds, hashedSecret }) => {
            const filter: Filter<AccountDocument> = {};

            if (accountObjectIds?.length) {
                filter.account_object_id = { $in: accountObjectIds };
            }

            if (hashedSecret) {
                filter.hashed_secret = hashedSecret;
            }

            if (Object.keys(filter).length === 0) {
                return 0;
            }

            const result = await collection.updateMany(filter, { $set: { hidden: true } });

            return result.modifiedCount;
        },
    };
};
