export interface Env {
  CONFIDE_LICENSES: KVNamespace;
  DB: D1Database;
  ENVIRONMENT: string;

  // Secrets (set via `wrangler secret put` after first deploy)
  LEMONSQUEEZY_WEBHOOK_SECRET: string;
  LEMONSQUEEZY_API_KEY: string;
  ANTHROPIC_API_KEY: string;
  OPENAI_API_KEY: string;
  RESEND_API_KEY: string;
}
