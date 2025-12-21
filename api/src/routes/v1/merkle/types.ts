import type { MerklePath } from '@/services/merkle.ts';

export type MerklePathResponse = {
    path: MerklePath;
    root: string;
};
