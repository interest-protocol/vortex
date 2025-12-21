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
import { REDIS_KEYS } from '@/constants/index.ts';
import type { CommitmentsRepository } from '@/repositories/index.ts';
import { hexToDecimal } from '@/utils/hex.ts';

export type MerklePath = [string, string][];

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

export type MerkleService = {
    getMerklePath: (params: {
        coinType: string;
        index: number;
        utxo: UtxoData;
    }) => Promise<MerklePathResponse>;
};

const getTreeKey = (coinType: string): string => `${REDIS_KEYS.MERKLE_TREE_PREFIX}${coinType}`;
const getLastIndexKey = (coinType: string): string =>
    `${REDIS_KEYS.MERKLE_LAST_INDEX_PREFIX}${coinType}`;

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

const cacheTree = async (
    redis: Redis,
    coinType: string,
    tree: MerkleTree,
    lastIndex: number
): Promise<void> => {
    const serialized = tree.serialize();
    await redis.set(getTreeKey(coinType), JSON.stringify(serialized));
    await redis.set(getLastIndexKey(coinType), lastIndex.toString());
};

const getOrBuildMerkleTree = async (
    redis: Redis,
    commitmentsRepo: CommitmentsRepository,
    coinType: string
): Promise<MerkleTree> => {
    const cachedTree = await getCachedTree(redis, coinType);
    const lastIndex = await getCachedLastIndex(redis, coinType);

    const commitments = await commitmentsRepo.findFromIndex(
        coinType,
        cachedTree ? lastIndex + 1 : 0
    );

    if (commitments.length === 0 && cachedTree) {
        return cachedTree;
    }

    const elements = commitments.map((c) => hexToDecimal(c.commitment));
    const tree = cachedTree ?? buildMerkleTree([]);

    if (elements.length > 0) {
        tree.bulkInsert(elements);
    }

    const newLastIndex = commitments.at(-1)?.index ?? lastIndex;
    await cacheTree(redis, coinType, tree, newLastIndex);

    return tree;
};

export const createMerkleService = (
    redis: Redis,
    commitmentsRepo: CommitmentsRepository
): MerkleService => ({
    getMerklePath: async ({ coinType, index, utxo }) => {
        const tree = await getOrBuildMerkleTree(redis, commitmentsRepo, coinType);
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
    },
});
