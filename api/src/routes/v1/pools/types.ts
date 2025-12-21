export type Pool = {
    id: string;
    digest: string;
    sender: string;
    checkpoint: number;
    checkpointTimestampMs: number;
    poolAddress: string;
    coinType: string;
};
