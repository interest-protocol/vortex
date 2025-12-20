import { Hono } from 'hono';
import type { AppBindings } from '@/types/index.ts';

type HealthStatus = 'healthy' | 'unhealthy';

type HealthData = {
    status: 'healthy' | 'degraded';
    services: {
        mongodb: HealthStatus;
        redis: HealthStatus;
    };
    timestamp: string;
};

export const healthRoutes = new Hono<AppBindings>().get('/', async (c) => {
    const db = c.get('db');
    const redis = c.get('redis');

    const [mongoStatus, redisStatus] = await Promise.all([
        db
            .command({ ping: 1 })
            .then((): HealthStatus => 'healthy')
            .catch((): HealthStatus => 'unhealthy'),
        redis
            .ping()
            .then((): HealthStatus => 'healthy')
            .catch((): HealthStatus => 'unhealthy'),
    ]);

    const isHealthy = mongoStatus === 'healthy' && redisStatus === 'healthy';

    const data: HealthData = {
        status: isHealthy ? 'healthy' : 'degraded',
        services: {
            mongodb: mongoStatus,
            redis: redisStatus,
        },
        timestamp: new Date().toISOString(),
    };

    return c.json({ success: true, data }, isHealthy ? 200 : 503);
});
