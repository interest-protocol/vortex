import type { MiddlewareHandler } from 'hono';
import { RateLimiterRedis, RateLimiterRes } from 'rate-limiter-flexible';
import { getRedis } from '@/db/redis.ts';
import type { AppBindings } from '@/types/index.ts';

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

const getClientIp = (c: { req: { header: (name: string) => string | undefined } }): string => {
    return (
        c.req.header('cf-connecting-ip') ??
        c.req.header('x-real-ip') ??
        c.req.header('x-forwarded-for')?.split(',')[0]?.trim() ??
        'unknown'
    );
};

const setRateLimitHeaders = (
    c: { header: (name: string, value: string) => void },
    res: RateLimiterRes,
    limit: number
): void => {
    c.header('X-RateLimit-Limit', limit.toString());
    c.header('X-RateLimit-Remaining', res.remainingPoints.toString());
    c.header(
        'X-RateLimit-Reset',
        Math.ceil(Date.now() / 1000 + res.msBeforeNext / 1000).toString()
    );
};

export const createRateLimiter = (
    config: Partial<RateLimitConfig> = {}
): MiddlewareHandler<AppBindings> => {
    const { points, duration, keyPrefix } = { ...DEFAULT_CONFIG, ...config };

    let rateLimiter: RateLimiterRedis | null = null;

    const getRateLimiter = (): RateLimiterRedis => {
        rateLimiter ??= new RateLimiterRedis({
            storeClient: getRedis(),
            points,
            duration,
            keyPrefix,
        });
        return rateLimiter;
    };

    return async (c, next) => {
        const clientIp = getClientIp(c);
        const limiter = getRateLimiter();

        try {
            const res = await limiter.consume(clientIp);
            setRateLimitHeaders(c, res, points);
            await next();
            return;
        } catch (err) {
            if (err instanceof RateLimiterRes) {
                setRateLimitHeaders(c, err, points);
                c.header('Retry-After', Math.ceil(err.msBeforeNext / 1000).toString());
                return c.json({ success: false, error: 'Too many requests' }, 429);
            }
            throw err;
        }
    };
};

export const rateLimitMiddleware = createRateLimiter();
