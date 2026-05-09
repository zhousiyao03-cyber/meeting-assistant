import { Hono } from "hono";
import { cors } from "hono/cors";
import { Env } from "./env";
import { getLicense, getKeyByEmail, planInfoFromLicense } from "./license";
import { recordUsage } from "./usage";
import { handleWebhook } from "./webhook";
import { licenseEmail, sendEmail } from "./emails";

const app = new Hono<{ Bindings: Env }>();

app.use(
  "/*",
  cors({
    origin: ["tauri://localhost", "http://localhost:1420", "https://confide.knosi.xyz"],
    allowMethods: ["GET", "POST"],
    allowHeaders: ["Content-Type", "Authorization"],
  }),
);

app.get("/", (c) => c.text("Confide API"));

app.get("/plan/:key", async (c) => {
  const license = await getLicense(c.env, c.req.param("key"));
  if (!license) return c.json({ error: "not_found" }, 404);
  if (license.revoked) return c.json({ error: "revoked" }, 403);
  return c.json(planInfoFromLicense(license));
});

app.post("/usage", async (c) => {
  const { key, events } = await c.req.json<{ key: string; events: any[] }>();
  if (typeof key !== "string" || !Array.isArray(events)) {
    return c.json({ error: "bad_request" }, 400);
  }
  try {
    const r = await recordUsage(c.env, key, events);
    const license = await getLicense(c.env, key);
    return c.json({ ...r, plan: license ? planInfoFromLicense(license) : null });
  } catch (e: any) {
    return c.json({ error: e.message }, 500);
  }
});

app.post("/lemonsqueezy-webhook", async (c) => {
  const rawBody = await c.req.text();
  const signature = c.req.header("X-Signature") ?? null;
  return handleWebhook(c.env, rawBody, signature);
});

app.post("/recover-license", async (c) => {
  const { email } = await c.req.json<{ email: string }>();
  if (typeof email !== "string") return c.json({ error: "bad_request" }, 400);
  const key = await getKeyByEmail(c.env, email);
  if (!key) return c.json({ error: "not_found" }, 404);
  const license = await getLicense(c.env, key);
  if (!license) return c.json({ error: "not_found" }, 404);
  const tmpl = licenseEmail(license.locale ?? "en-US", key, license.tier);
  await sendEmail(c.env.RESEND_API_KEY, email, tmpl);
  return c.json({ ok: true });
});

export default app;
