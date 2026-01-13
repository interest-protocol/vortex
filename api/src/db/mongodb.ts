import { MongoClient, type Db } from 'mongodb';
import invariant from 'tiny-invariant';
import { env } from '@/config/env.ts';
import { logger } from '@/utils/logger.ts';
import { ACCOUNTS_COLLECTION } from './collections/index.ts';

let client: MongoClient | null = null;

let db: Db | null = null;

const ensureIndexes = async (database: Db): Promise<void> => {
    await database.collection(ACCOUNTS_COLLECTION).createIndex({ hidden: 1 });
};

export const connectMongoDB = async (): Promise<Db> => {
    if (db) return db;

    client = new MongoClient(env.MONGODB_URI);
    await client.connect();
    db = client.db();

    await ensureIndexes(db);

    logger.info('Connected to MongoDB');
    return db;
};

export const disconnectMongoDB = async (): Promise<void> => {
    if (!client) return;

    await client.close();
    client = null;
    db = null;
    logger.info('Disconnected from MongoDB');
};

export const getDb = (): Db => {
    invariant(db, 'MongoDB not connected. Call connectMongoDB first.');
    return db;
};
