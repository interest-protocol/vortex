import type { Context } from 'hono';
import type { z } from 'zod';

type ValidationResult<T> = { success: true; data: T } | { success: false; response: Response };

const validate = <T extends z.ZodSchema>(
    c: Context,
    schema: T,
    data: unknown
): ValidationResult<z.infer<T>> => {
    const parsed = schema.safeParse(data);

    if (!parsed.success) {
        return {
            success: false,
            response: c.json({ success: false, error: parsed.error.flatten().fieldErrors }, 400),
        };
    }

    return { success: true, data: parsed.data as z.infer<T> };
};

export const validateBody = async <T extends z.ZodSchema>(
    c: Context,
    schema: T
): Promise<ValidationResult<z.infer<T>>> => {
    const body: unknown = await c.req.json().catch(() => ({}));
    return validate(c, schema, body);
};

export const validateQuery = <T extends z.ZodSchema>(
    c: Context,
    schema: T
): ValidationResult<z.infer<T>> => validate(c, schema, c.req.query());
