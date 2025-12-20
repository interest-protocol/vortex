import { hexToDecimal } from '@/utils/hex.ts';
import type { CommitmentDocument, Commitment } from './types.ts';

export const toCommitment = (doc: CommitmentDocument): Commitment => ({
    id: doc._id,
    digest: doc.digest,
    sender: doc.sender,
    checkpoint: doc.checkpoint,
    checkpointTimestampMs: doc.checkpoint_timestamp_ms,
    coinType: doc.coin_type,
    index: doc.index,
    commitment: hexToDecimal(doc.commitment),
    encryptedOutput: doc.encrypted_output,
});
