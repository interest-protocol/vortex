import type { Db } from 'mongodb';
import type { Redis } from 'ioredis';
import {
    buildMerkleTree,
    deserializeMerkleTree,
    getMerklePath as sdkGetMerklePath,
    type Utxo,
    type MerkleTree,
    type MerklePath,
} from '@interest-protocol/vortex-sdk';
import type { SerializedTreeState } from 'fixed-merkle-tree';
import { COMMITMENTS_COLLECTION, type CommitmentDocument } from '@/db/collections/index.ts';
import { REDIS_KEYS } from '@/constants/index.ts';

const getTreeKey = (coinType: string): string => `${REDIS_KEYS.MERKLE_TREE_PREFIX}${coinType}`;

const getLastIndexKey = (coinType: string): string =>
    `${REDIS_KEYS.MERKLE_LAST_INDEX_PREFIX}${coinType}`;

type FetchCommitmentsParams = {
    db: Db;
    coinType: string;
    fromIndex: number;
};

const fetchCommitmentsFromIndex = async ({
    db,
    coinType,
    fromIndex,
}: FetchCommitmentsParams): Promise<CommitmentDocument[]> => {
    const collection = db.collection<CommitmentDocument>(COMMITMENTS_COLLECTION);
    return collection
        .find({ coin_type: coinType, index: { $gte: fromIndex } })
        .sort({ index: 1 })
        .toArray();
};

const getCachedTree = async (redis: Redis, coinType: string): Promise<MerkleTree | null> => {
    const data = await redis.get(getTreeKey(coinType));
    if (!data) return null;

    const serialized = JSON.parse(data) as SerializedTreeState;
    return deserializeMerkleTree(serialized);
};

const getCachedLastIndex = async (redis: Redis, coinType: string): Promise<number> => {
    const data = await redis.get(getLastIndexKey(coinType));
    return data ? parseInt(data, 10) : -1;
};

type CacheTreeParams = {
    redis: Redis;
    coinType: string;
    tree: MerkleTree;
    lastIndex: number;
};

const cacheTree = async ({ redis, coinType, tree, lastIndex }: CacheTreeParams): Promise<void> => {
    const serialized = tree.serialize();
    await redis.set(getTreeKey(coinType), JSON.stringify(serialized));
    await redis.set(getLastIndexKey(coinType), lastIndex.toString());
};

type GetOrBuildMerkleTreeParams = {
    db: Db;
    redis: Redis;
    coinType: string;
};

export const getOrBuildMerkleTree = async ({
    db,
    redis,
    coinType,
}: GetOrBuildMerkleTreeParams): Promise<MerkleTree> => {
    let tree = await getCachedTree(redis, coinType);
    let lastIndex = await getCachedLastIndex(redis, coinType);

    const newCommitments = await fetchCommitmentsFromIndex({
        db,
        coinType,
        fromIndex: lastIndex + 1,
    });

    if (newCommitments.length === 0 && tree) {
        return tree;
    }

    if (!tree) {
        const allCommitments = await fetchCommitmentsFromIndex({ db, coinType, fromIndex: 0 });
        const elements = allCommitments.map((c) => c.commitment);
        tree = buildMerkleTree(elements);
        const lastCommitment = allCommitments[allCommitments.length - 1];
        lastIndex = lastCommitment ? lastCommitment.index : -1;
    } else {
        const elements = newCommitments.map((c) => c.commitment);
        tree.bulkInsert(elements);
        const lastNewCommitment = newCommitments[newCommitments.length - 1];
        if (lastNewCommitment) {
            lastIndex = lastNewCommitment.index;
        }
    }

    await cacheTree({ redis, coinType, tree, lastIndex });

    return tree;
};

export type { MerklePath };

export type MerklePathResponse = {
    path: MerklePath;
    root: string;
};

type GetMerklePathParams = {
    db: Db;
    redis: Redis;
    coinType: string;
    utxo: Utxo;
};

export const getMerklePath = async ({
    db,
    redis,
    coinType,
    utxo,
}: GetMerklePathParams): Promise<MerklePathResponse> => {
    const tree = await getOrBuildMerkleTree({ db, redis, coinType });
    const path = sdkGetMerklePath(tree, utxo);

    return {
        path,
        root: tree.root.toString(),
    };
};
