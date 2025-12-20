import { Hono } from 'hono';
import { logger as honoLogger } from 'hono/logger';
import { env } from '@/config/env.ts';
import { connectMongoDB, disconnectMongoDB } from '@/db/mongodb.ts';
import { connectRedis, disconnectRedis } from '@/db/redis.ts';
import { corsMiddleware, databaseMiddleware, errorHandler } from '@/middleware/index.ts';
import { routes } from '@/routes/index.ts';
import type { AppBindings } from '@/types/index.ts';
import { logger } from '@/utils/logger.ts';

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
        },
    })
);

app.route('/api', routes);

async function main() {
    await connectMongoDB();
    connectRedis();

    logger.info({ host: env.HOST, port: env.PORT }, 'Server started');

    return {
        port: env.PORT,
        hostname: env.HOST,
        fetch: app.fetch,
    };
}

async function shutdown() {
    logger.info('Shutting down...');
    await disconnectMongoDB();
    await disconnectRedis();
    process.exit(0);
}

process.on('SIGINT', () => {
    shutdown().catch((err: unknown) => {
        logger.error({ err }, 'Shutdown error');
    });
});
process.on('SIGTERM', () => {
    shutdown().catch((err: unknown) => {
        logger.error({ err }, 'Shutdown error');
    });
});

export default await main();
