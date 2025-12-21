export const COMMITMENTS_COLLECTION = 'new_commitments';

export type CommitmentDocument = {
    _id: string;
    digest: string;
    sender: string;
    checkpoint: number;
    checkpoint_timestamp_ms: number;
    coin_type: string;
    index: number;
    commitment: string;
    encrypted_output: number[];
};
