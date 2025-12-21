import type { Db } from 'mongodb';
import { ACCOUNTS_COLLECTION, type AccountDocument } from '@/db/collections/index.ts';

export type AccountFilter = {
    hashed_secret: string;
};

export type AccountsRepository = {
    findByHashedSecret: (hashedSecret: string) => Promise<AccountDocument[]>;
    insert: (doc: AccountDocument) => Promise<void>;
};

export const createAccountsRepository = (db: Db): AccountsRepository => {
    const collection = db.collection<AccountDocument>(ACCOUNTS_COLLECTION);

    return {
        findByHashedSecret: async (hashedSecret) =>
            collection.find({ hashed_secret: hashedSecret }).toArray(),

        insert: async (doc) => {
            await collection.insertOne(doc);
        },
    };
};
