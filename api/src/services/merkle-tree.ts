import type { Db } from 'mongodb';
import type { Redis } from 'ioredis';
import {
    buildMerkleTree,
    deserializeMerkleTree,
    ZERO_VALUE,
    MERKLE_TREE_HEIGHT,
    poseidon2,
    Utxo,
    type MerkleTree,
} from '@interest-protocol/vortex-sdk';
import type { SerializedTreeState } from 'fixed-merkle-tree';
import { COMMITMENTS_COLLECTION, type CommitmentDocument } from '@/db/collections/index.ts';
import { REDIS_KEYS } from '@/constants/index.ts';
import { hexToDecimal } from '@/utils/hex.ts';

export type MerklePath = [string, string][];

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
        const elements = allCommitments.map((c) => hexToDecimal(c.commitment));
        tree = buildMerkleTree(elements);
        const lastCommitment = allCommitments[allCommitments.length - 1];
        lastIndex = lastCommitment ? lastCommitment.index : -1;
    } else {
        const elements = newCommitments.map((c) => hexToDecimal(c.commitment));
        tree.bulkInsert(elements);
        const lastNewCommitment = newCommitments[newCommitments.length - 1];
        if (lastNewCommitment) {
            lastIndex = lastNewCommitment.index;
        }
    }

    await cacheTree({ redis, coinType, tree, lastIndex });

    return tree;
};

export type MerklePathResponse = {
    path: MerklePath;
    root: string;
};

export type UtxoData = {
    amount: bigint;
    publicKey: string;
    blinding: bigint;
    vortexPool: string;
};

type GetMerklePathParams = {
    db: Db;
    redis: Redis;
    coinType: string;
    index: number;
    utxo: UtxoData;
};

export const getMerklePath = async ({
    db,
    redis,
    coinType,
    index,
    utxo,
}: GetMerklePathParams): Promise<MerklePathResponse> => {
    const tree = await getOrBuildMerkleTree({ db, redis, coinType });
    const treeSize = tree.elements.length;

    if (index < 0 || index >= treeSize) {
        const zeroPath: MerklePath = Array(MERKLE_TREE_HEIGHT)
            .fill(null)
            .map(() => [ZERO_VALUE.toString(), ZERO_VALUE.toString()]);
        return { path: zeroPath, root: tree.root.toString() };
    }

    const { pathElements, pathIndices } = tree.path(index);
    const commitment = Utxo.makeCommitment(utxo);

    const storedCommitment = BigInt(tree.elements[index] as string);
    if (storedCommitment !== commitment) {
        throw new Error(`Commitment mismatch at index ${String(index)}`);
    }

    const wasmPath: MerklePath = [];
    let currentHash = commitment;

    for (let i = 0; i < MERKLE_TREE_HEIGHT; i++) {
        const sibling = BigInt(pathElements[i] as string);
        const isLeft = pathIndices[i] === 0;

        const leftHash = isLeft ? currentHash : sibling;
        const rightHash = isLeft ? sibling : currentHash;

        wasmPath.push([leftHash.toString(), rightHash.toString()]);

        currentHash = poseidon2(leftHash, rightHash);
    }

    return {
        path: wasmPath,
        root: tree.root.toString(),
    };
};
