import { Hono } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import type { HealthStatus } from '@/services/health.ts';
import { withErrorHandler } from '@/utils/handler.ts';

type HealthData = {
    status: 'healthy' | 'degraded';
    services: {
        mongodb: HealthStatus;
        redis: HealthStatus;
        sui: HealthStatus;
    };
    timestamp: string;
};

const checkHealthHandler = withErrorHandler(async (c) => {
    const healthService = c.get('healthService');
    const services = await healthService.check();

    const isHealthy =
        services.mongodb === 'healthy' &&
        services.redis === 'healthy' &&
        services.sui === 'healthy';

    const data: HealthData = {
        status: isHealthy ? 'healthy' : 'degraded',
        services,
        timestamp: new Date().toISOString(),
    };

    return c.json({ success: true, data }, isHealthy ? 200 : 503);
}, 'Health check failed');

export const healthRoutes = new Hono<AppBindings>().get('/', checkHealthHandler);
