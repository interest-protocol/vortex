import type { MerklePath } from '@/services/merkle-tree.ts';

export type MerklePathResponse = {
    path: MerklePath;
    root: string;
};
