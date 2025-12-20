import type { CommitmentDocument, Commitment } from './types.ts';

export function toCommitment(doc: CommitmentDocument): Commitment {
    return {
        id: doc._id,
        digest: doc.digest,
        sender: doc.sender,
        checkpoint: doc.checkpoint,
        checkpointTimestampMs: doc.checkpoint_timestamp_ms,
        coinType: doc.coin_type,
        index: doc.index,
        commitment: doc.commitment,
        encryptedOutput: doc.encrypted_output,
    };
}
