import type { Db } from 'mongodb';
import type { Redis } from 'ioredis';
import { nodeClient } from './sui.ts';

export type HealthStatus = 'healthy' | 'unhealthy';

export type IndexerStatus = 'synced' | 'behind' | 'unknown';

export type HealthCheckResult = {
    mongodb: HealthStatus;
    redis: HealthStatus;
    sui: HealthStatus;
    indexer: IndexerStatus;
};

export type IndexerSyncInfo = {
    status: IndexerStatus;
    checkpointsBehind: number | null;
    indexerCheckpoint: number | null;
    suiCheckpoint: number | null;
};

export type HealthService = {
    check: () => Promise<HealthCheckResult>;
    getIndexerSyncInfo: () => Promise<IndexerSyncInfo>;
};

const MAX_CHECKPOINT_LAG = 1000;

const toHealthStatus = (promise: Promise<unknown>): Promise<HealthStatus> =>
    promise.then((): HealthStatus => 'healthy').catch((): HealthStatus => 'unhealthy');

type Watermark = {
    _id: string;
    checkpoint_hi_inclusive: number | bigint;
};

export const createHealthService = (db: Db, redis: Redis): HealthService => {
    const getIndexerSyncInfo = async (): Promise<IndexerSyncInfo> => {
        try {
            const [watermarks, suiCheckpoint] = await Promise.all([
                db.collection<Watermark>('watermarks').find({}).toArray(),
                nodeClient.getLatestCheckpointSequenceNumber(),
            ]);

            if (watermarks.length === 0) {
                return {
                    status: 'unknown',
                    checkpointsBehind: null,
                    indexerCheckpoint: null,
                    suiCheckpoint: null,
                };
            }

            const minCheckpoint = Math.min(
                ...watermarks.map((w) => Number(w.checkpoint_hi_inclusive))
            );
            const suiCheckpointNum = Number(suiCheckpoint);
            const checkpointsBehind = suiCheckpointNum - minCheckpoint;

            const status: IndexerStatus =
                checkpointsBehind <= MAX_CHECKPOINT_LAG ? 'synced' : 'behind';

            return {
                status,
                checkpointsBehind,
                indexerCheckpoint: minCheckpoint,
                suiCheckpoint: suiCheckpointNum,
            };
        } catch {
            return {
                status: 'unknown',
                checkpointsBehind: null,
                indexerCheckpoint: null,
                suiCheckpoint: null,
            };
        }
    };

    return {
        check: async () => {
            const [mongodb, redisStatus, sui, indexerInfo] = await Promise.all([
                toHealthStatus(db.command({ ping: 1 })),
                toHealthStatus(redis.ping()),
                toHealthStatus(nodeClient.getLatestCheckpointSequenceNumber()),
                getIndexerSyncInfo(),
            ]);

            return { mongodb, redis: redisStatus, sui, indexer: indexerInfo.status };
        },
        getIndexerSyncInfo,
    };
};
