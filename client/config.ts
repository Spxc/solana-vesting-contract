import "dotenv/config";
import { z } from "zod";

/**
 * App running mode
 */
export enum NodeEnv {
  PROD = "prd",
  DEV = "dev",
  TEST = "test",
}

const configSchema = z.object({
  NODE_ENV: z.nativeEnum(NodeEnv).default(NodeEnv.PROD),
  IDL: z.string(),
});

export const config = configSchema.parse(process.env);
