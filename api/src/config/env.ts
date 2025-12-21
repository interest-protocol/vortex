import { z } from 'zod';

const baseSchema = z.object({
    NODE_ENV: z.enum(['development', 'production', 'test']).default('development'),
    PORT: z.coerce.number().default(3000),
    HOST: z.string().default('0.0.0.0'),
    MONGODB_URI: z.string().default('mongodb://localhost:27017/vortex'),
    REDIS_URL: z.string().default('redis://localhost:6379'),
    CORS_ORIGIN: z.string().optional(),
    SUI_PRIVATE_KEY: z.string(),
    SHINAMI_RPC_KEY: z.string(),
});

const envSchema = baseSchema.refine((data) => data.NODE_ENV !== 'production' || data.CORS_ORIGIN, {
    message: 'CORS_ORIGIN is required in production',
    path: ['CORS_ORIGIN'],
});

const parsed = envSchema.safeParse(Bun.env);

if (!parsed.success) {
    console.error('Invalid environment variables:', parsed.error.flatten());
    process.exit(1);
}

export const env = parsed.data;
