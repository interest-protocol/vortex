import type { Db } from 'mongodb';
import type { Redis } from 'ioredis';
import { nodeClient } from './sui.ts';

export type HealthStatus = 'healthy' | 'unhealthy';

export type HealthCheckResult = {
    mongodb: HealthStatus;
    redis: HealthStatus;
    sui: HealthStatus;
};

export type HealthService = {
    check: () => Promise<HealthCheckResult>;
};

const toHealthStatus = (promise: Promise<unknown>): Promise<HealthStatus> =>
    promise.then((): HealthStatus => 'healthy').catch((): HealthStatus => 'unhealthy');

export const createHealthService = (db: Db, redis: Redis): HealthService => ({
    check: async () => {
        const [mongodb, redisStatus, sui] = await Promise.all([
            toHealthStatus(db.command({ ping: 1 })),
            toHealthStatus(redis.ping()),
            toHealthStatus(nodeClient.getLatestCheckpointSequenceNumber()),
        ]);

        return { mongodb, redis: redisStatus, sui };
    },
});
