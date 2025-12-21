import Redis from 'ioredis';
import invariant from 'tiny-invariant';
import { env } from '@/config/env.ts';
import { logger } from '@/utils/logger.ts';

let redis: Redis | null = null;

export const connectRedis = (): Redis => {
    if (redis) return redis;

    redis = new Redis(env.REDIS_URL, {
        maxRetriesPerRequest: 3,
        retryStrategy: (times) => Math.min(times * 50, 2000),
    });

    redis.on('connect', () => {
        logger.info('Connected to Redis');
    });
    redis.on('error', (err) => {
        logger.error({ err }, 'Redis connection error');
    });

    return redis;
};

export const disconnectRedis = async (): Promise<void> => {
    if (!redis) return;

    await redis.quit();
    redis = null;
    logger.info('Disconnected from Redis');
};

export const getRedis = (): Redis => {
    invariant(redis, 'Redis not connected. Call connectRedis first.');
    return redis;
};
