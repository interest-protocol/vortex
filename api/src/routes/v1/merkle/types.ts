import type { MerklePath } from '@interest-protocol/vortex-sdk';

export type MerklePathResponse = {
    path: MerklePath;
    root: string;
};
