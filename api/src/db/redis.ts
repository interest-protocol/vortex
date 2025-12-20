import Redis from 'ioredis';
import { env } from '../config/env.js';
import { logger } from '../utils/logger.js';

let redis: Redis | null = null;

export function connectRedis(): Redis {
    if (redis) {
        return redis;
    }

    redis = new Redis(env.REDIS_URL, {
        maxRetriesPerRequest: 3,
        retryStrategy(times) {
            const delay = Math.min(times * 50, 2000);
            return delay;
        },
    });

    redis.on('connect', () => {
        logger.info('Connected to Redis');
    });

    redis.on('error', (err) => {
        logger.error({ err }, 'Redis connection error');
    });

    return redis;
}

export async function disconnectRedis(): Promise<void> {
    if (redis) {
        await redis.quit();
        redis = null;
        logger.info('Disconnected from Redis');
    }
}

export function getRedis(): Redis {
    if (!redis) {
        throw new Error('Redis not connected. Call connectRedis first.');
    }
    return redis;
}
