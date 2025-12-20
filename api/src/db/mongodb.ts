import { MongoClient, type Db } from 'mongodb';
import { env } from '@/config/env.ts';
import { logger } from '@/utils/logger.ts';

let client: MongoClient | null = null;
let db: Db | null = null;

export async function connectMongoDB(): Promise<Db> {
    if (db) {
        return db;
    }

    client = new MongoClient(env.MONGODB_URI);
    await client.connect();
    db = client.db();

    logger.info('Connected to MongoDB');
    return db;
}

export async function disconnectMongoDB(): Promise<void> {
    if (client) {
        await client.close();
        client = null;
        db = null;
        logger.info('Disconnected from MongoDB');
    }
}

export function getDb(): Db {
    if (!db) {
        throw new Error('MongoDB not connected. Call connectMongoDB first.');
    }
    return db;
}
