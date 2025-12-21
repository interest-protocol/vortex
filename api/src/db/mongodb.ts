import { MongoClient, type Db } from 'mongodb';
import { env } from '@/config/env.ts';
import { logger } from '@/utils/logger.ts';

let client: MongoClient | null = null;
let db: Db | null = null;

export const connectMongoDB = async (): Promise<Db> => {
    if (db) return db;

    client = new MongoClient(env.MONGODB_URI);
    await client.connect();
    db = client.db();

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
    if (!db) throw new Error('MongoDB not connected. Call connectMongoDB first.');
    return db;
};
