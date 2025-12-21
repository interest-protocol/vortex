import { z } from 'zod';

export const executeTransactionSchema = z.object({
    txBytes: z.string().min(1),
});
