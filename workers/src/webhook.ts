import { Env } from "./env";
import {
  License,
  newFreeLicense,
  generateLicenseKey,
  PLAN_CATALOG,
  Tier,
} from "./plans";
import { getLicense, getKeyByEmail, putLicense, setKeyForEmail } from "./license";
import { licenseEmail, sendEmail } from "./emails";

async function verifySignature(
  rawBody: string,
  signatureHeader: string | null,
  secret: string,
): Promise<boolean> {
  if (!signatureHeader || !secret) return false;
  const enc = new TextEncoder();
  const key = await crypto.subtle.importKey(
    "raw",
    enc.encode(secret),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"],
  );
  const sig = await crypto.subtle.sign("HMAC", key, enc.encode(rawBody));
  const hex = Array.from(new Uint8Array(sig))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
  if (hex.length !== signatureHeader.length) return false;
  let diff = 0;
  for (let i = 0; i < hex.length; i++) {
    diff |= hex.charCodeAt(i) ^ signatureHeader.charCodeAt(i);
  }
  return diff === 0;
}

function tierFromVariantId(variantId: string): Tier | null {
  for (const [tier, cfg] of Object.entries(PLAN_CATALOG)) {
    if (cfg.lemonVariantId === variantId) return tier as Tier;
  }
  return null;
}

export async function handleWebhook(
  env: Env,
  rawBody: string,
  signatureHeader: string | null,
): Promise<Response> {
  if (!(await verifySignature(rawBody, signatureHeader, env.LEMONSQUEEZY_WEBHOOK_SECRET))) {
    return new Response("invalid signature", { status: 401 });
  }

  const event = JSON.parse(rawBody);
  const eventName = event.meta?.event_name as string;
  const data = event.data;

  const email = data?.attributes?.user_email as string | undefined;
  const variantId = String(data?.attributes?.variant_id ?? "");
  const eventId = String(event.meta?.event_id ?? crypto.randomUUID());

  await env.DB.prepare(
    "INSERT INTO lemonsqueezy_events (id, license_key, event_type, amount_cents, ts) VALUES (?, ?, ?, ?, ?)",
  )
    .bind(eventId, "", eventName, data?.attributes?.total ?? null, Date.now())
    .run()
    .catch(() => {});

  if (!email) {
    return new Response("ok (no email)", { status: 200 });
  }

  let key = await getKeyByEmail(env, email);
  let license: License | null = key ? await getLicense(env, key) : null;
  if (!license) {
    key = generateLicenseKey();
    license = newFreeLicense(email, "en-US");
    await setKeyForEmail(env, email, key);
  }

  switch (eventName) {
    case "subscription_created":
    case "subscription_payment_success": {
      const tier = tierFromVariantId(variantId);
      if (tier) {
        license.tier = tier;
        license.used_this_month_seconds = 0;
        license.resume_optimization_credits_used = 0;
        license.cancelled_at = null;
        license.revoked = false;
        license.renews_at = data.attributes.renews_at
          ? new Date(data.attributes.renews_at).getTime()
          : null;
      }
      break;
    }
    case "subscription_cancelled":
      license.cancelled_at = Date.now();
      break;
    case "subscription_expired":
    case "subscription_payment_failed":
      license.tier = "free";
      license.cancelled_at = Date.now();
      license.renews_at = null;
      break;
    case "order_refunded":
      license.revoked = true;
      break;
  }

  await putLicense(env, key!, license);

  if (eventName === "subscription_created" && license.tier !== "free") {
    const tmpl = licenseEmail(license.locale ?? "en-US", key!, license.tier);
    await sendEmail(env.RESEND_API_KEY, email, tmpl);
  }

  return new Response("ok", { status: 200 });
}
