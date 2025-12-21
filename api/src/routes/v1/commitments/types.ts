export type { CommitmentDocument } from '@/db/collections/index.ts';

export type Commitment = {
    id: string;
    digest: string;
    sender: string;
    checkpoint: number;
    checkpointTimestampMs: number;
    coinType: string;
    index: number;
    commitment: string;
    encryptedOutput: number[];
};
