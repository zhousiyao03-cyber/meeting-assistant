import { Env } from "./env";
import { getLicense, putLicense } from "./license";

export interface UsageEvent {
  event_id: string;
  meeting_id: string;
  provider: string;
  seconds_used: number;
  started_at: number;
  ended_at: number;
}

export async function recordUsage(
  env: Env,
  key: string,
  events: UsageEvent[],
): Promise<{ accepted: number; deduped: number }> {
  let accepted = 0;
  let deduped = 0;
  const license = await getLicense(env, key);
  if (!license) throw new Error("license not found");

  for (const evt of events) {
    const seen = await env.CONFIDE_LICENSES.get(`event:${evt.event_id}`);
    if (seen) {
      deduped++;
      continue;
    }

    if (evt.provider === "confide") {
      license.used_this_month_seconds += evt.seconds_used;
    }
    // BYO providers don't count

    await env.DB.prepare(
      "INSERT INTO usage_events (event_id, license_key, provider, seconds, ts) VALUES (?, ?, ?, ?, ?)",
    )
      .bind(evt.event_id, key, evt.provider, evt.seconds_used, evt.ended_at)
      .run();

    await env.CONFIDE_LICENSES.put(`event:${evt.event_id}`, "1", { expirationTtl: 86400 * 30 });
    accepted++;
  }

  await putLicense(env, key, license);
  return { accepted, deduped };
}
