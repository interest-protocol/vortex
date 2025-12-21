import { Hono } from 'hono';
import { logger as honoLogger } from 'hono/logger';
import { Scalar } from '@scalar/hono-api-reference';
import { env } from '@/config/env.ts';
import { connectMongoDB, disconnectMongoDB } from '@/db/mongodb.ts';
import { connectRedis, disconnectRedis } from '@/db/redis.ts';
import { openApiSpec } from '@/docs/openapi.ts';
import { corsMiddleware, databaseMiddleware, errorHandler } from '@/middleware/index.ts';
import { routes } from '@/routes/index.ts';
import type { AppBindings } from '@/types/index.ts';
import { logger } from '@/utils/logger.ts';

const createApp = () => {
    const app = new Hono<AppBindings>();

    app.use(honoLogger());
    app.use(corsMiddleware);
    app.use(databaseMiddleware);
    app.onError(errorHandler);

    app.get('/', (c) =>
        c.json({
            success: true,
            data: {
                name: 'Vortex API',
                version: '1.0.0',
                docs: '/docs',
            },
        })
    );

    app.get('/openapi.json', (c) => c.json(openApiSpec));

    app.get(
        '/docs',
        Scalar({
            url: '/openapi.json',
            pageTitle: 'Vortex API Documentation',
        })
    );

    app.route('/api', routes);

    return app;
};

const main = async () => {
    await connectMongoDB();
    connectRedis();

    const app = createApp();

    logger.info({ host: env.HOST, port: env.PORT }, 'Server started');

    return {
        port: env.PORT,
        hostname: env.HOST,
        fetch: app.fetch,
    };
};

const shutdown = async () => {
    logger.info('Shutting down...');
    await disconnectMongoDB();
    await disconnectRedis();
    process.exit(0);
};

const handleShutdown = () => {
    shutdown().catch((err: unknown) => {
        logger.error({ err }, 'Shutdown error');
    });
};

process.on('SIGINT', handleShutdown);
process.on('SIGTERM', handleShutdown);

export default await main();
