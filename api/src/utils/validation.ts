import type { Context } from 'hono';
import type { z } from 'zod';

type ValidationResult<T> = { success: true; data: T } | { success: false; response: Response };

export function validateBody<T extends z.ZodSchema>(
    c: Context,
    schema: T,
    body: unknown
): ValidationResult<z.infer<T>> {
    const parsed = schema.safeParse(body);

    if (!parsed.success) {
        return {
            success: false,
            response: c.json(
                {
                    success: false,
                    error: parsed.error.flatten().fieldErrors,
                },
                400
            ),
        };
    }

    return { success: true, data: parsed.data as z.infer<T> };
}

export function validateQuery<T extends z.ZodSchema>(
    c: Context,
    schema: T,
    query: unknown
): ValidationResult<z.infer<T>> {
    const parsed = schema.safeParse(query);

    if (!parsed.success) {
        return {
            success: false,
            response: c.json(
                {
                    success: false,
                    error: parsed.error.flatten().fieldErrors,
                },
                400
            ),
        };
    }

    return { success: true, data: parsed.data as z.infer<T> };
}
