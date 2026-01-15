import type { MiddlewareHandler } from 'hono';
import {
    RateLimiterMemory,
    RateLimiterRedis,
    RateLimiterRes,
    type RateLimiterAbstract,
} from 'rate-limiter-flexible';
import { env } from '@/config/env.ts';
import { getRedis } from '@/db/redis.ts';
import type { AppBindings } from '@/types/index.ts';
import { logger } from '@/utils/logger.ts';

type RateLimitConfig = {
    points: number;
    duration: number;
    keyPrefix: string;
};

const DEFAULT_CONFIG: RateLimitConfig = {
    points: 100,
    duration: 60,
    keyPrefix: 'rl',
};

const getClientIp = (c: {
    req: { header: (name: string) => string | undefined };
}): string | null => {
    const ip =
        c.req.header('cf-connecting-ip') ??
        c.req.header('x-real-ip') ??
        c.req.header('x-forwarded-for')?.split(',')[0]?.trim();

    return ip ?? null;
};

const setRateLimitHeaders = (
    c: { header: (name: string, value: string) => void },
    res: RateLimiterRes,
    limit: number,
    duration: number
): void => {
    const resetSeconds = Math.ceil(res.msBeforeNext / 1000);

    c.header(
        'RateLimit',
        `limit=${String(limit)}, remaining=${String(res.remainingPoints)}, reset=${String(resetSeconds)}`
    );
    c.header('RateLimit-Policy', `${String(limit)};w=${String(duration)}`);
};

const UNKNOWN_IP_KEY = 'unknown';
const UNKNOWN_IP_POINTS_DIVISOR = 10;

export const createRateLimiter = (
    config: Partial<RateLimitConfig> = {}
): MiddlewareHandler<AppBindings> => {
    const { points, duration, keyPrefix } = { ...DEFAULT_CONFIG, ...config };

    const memoryLimiter = new RateLimiterMemory({ points, duration, keyPrefix });

    const unknownIpPoints = Math.max(1, Math.floor(points / UNKNOWN_IP_POINTS_DIVISOR));
    const unknownIpMemoryLimiter = new RateLimiterMemory({
        points: unknownIpPoints,
        duration,
        keyPrefix: `${keyPrefix}:unknown`,
    });

    let redisLimiter: RateLimiterRedis | null = null;
    let unknownIpRedisLimiter: RateLimiterRedis | null = null;
    let lastRedisInstance: ReturnType<typeof getRedis> | null = null;

    const getLimiters = (): { main: RateLimiterAbstract; unknownIp: RateLimiterAbstract } => {
        const currentRedis = getRedis();

        if (currentRedis !== lastRedisInstance) {
            lastRedisInstance = currentRedis;
            redisLimiter = new RateLimiterRedis({
                storeClient: currentRedis,
                points,
                duration,
                keyPrefix,
                insuranceLimiter: memoryLimiter,
            });
            unknownIpRedisLimiter = new RateLimiterRedis({
                storeClient: currentRedis,
                points: unknownIpPoints,
                duration,
                keyPrefix: `${keyPrefix}:unknown`,
                insuranceLimiter: unknownIpMemoryLimiter,
            });
        }

        return {
            main: redisLimiter ?? memoryLimiter,
            unknownIp: unknownIpRedisLimiter ?? unknownIpMemoryLimiter,
        };
    };

    return async (c, next) => {
        const apiKey = c.req.header('x-api-key');
        if (apiKey && env.API_KEY && apiKey === env.API_KEY) {
            await next();
            return;
        }

        const clientIp = getClientIp(c);
        const limiters = getLimiters();

        const isUnknownIp = !clientIp;
        const limiter = isUnknownIp ? limiters.unknownIp : limiters.main;
        const key = clientIp ?? UNKNOWN_IP_KEY;
        const effectivePoints = isUnknownIp ? unknownIpPoints : points;

        if (isUnknownIp) {
            logger.warn('Could not determine client IP, applying stricter rate limit');
        }

        try {
            const res = await limiter.consume(key);
            setRateLimitHeaders(c, res, effectivePoints, duration);
            await next();
            return;
        } catch (err) {
            if (err instanceof RateLimiterRes) {
                setRateLimitHeaders(c, err, effectivePoints, duration);
                c.header('Retry-After', Math.ceil(err.msBeforeNext / 1000).toString());
                return c.json({ success: false, error: 'Too many requests' }, 429);
            }
            throw err;
        }
    };
};

export const rateLimitMiddleware = createRateLimiter();
