import { Tier } from "./plans";

type Locale = "zh-CN" | "en-US";

export function licenseEmail(locale: Locale, key: string, tier: Tier) {
  const tierLabel = tier.toUpperCase();
  if (locale === "zh-CN") {
    return {
      subject: `你的 Confide ${tierLabel} license`,
      html: `
        <div style="font-family: -apple-system, sans-serif; max-width: 480px;">
          <h1 style="font-size: 22px;">感谢订阅 Confide ${tierLabel}</h1>
          <p>你的 license key:</p>
          <code style="display: block; font-size: 16px; padding: 14px; background: #f5f5f5; border-radius: 6px; word-break: break-all;">${key}</code>
          <p style="margin-top: 18px; font-size: 14px; color: #666;">
            打开 Confide → 设置 → License → 输入此 key 激活。
          </p>
          <p style="font-size: 12px; color: #999; margin-top: 24px;">
            遇到问题：hello@confide.knosi.xyz
          </p>
        </div>
      `,
    };
  }
  return {
    subject: `Your Confide ${tierLabel} license`,
    html: `
      <div style="font-family: -apple-system, sans-serif; max-width: 480px;">
        <h1 style="font-size: 22px;">Thanks for subscribing to Confide ${tierLabel}</h1>
        <p>Your license key:</p>
        <code style="display: block; font-size: 16px; padding: 14px; background: #f5f5f5; border-radius: 6px; word-break: break-all;">${key}</code>
        <p style="margin-top: 18px; font-size: 14px; color: #666;">
          Open Confide → Settings → License → enter this key to activate.
        </p>
        <p style="font-size: 12px; color: #999; margin-top: 24px;">
          Need help? hello@confide.knosi.xyz
        </p>
      </div>
    `,
  };
}

export async function sendEmail(
  resendApiKey: string,
  to: string,
  email: { subject: string; html: string },
): Promise<void> {
  if (!resendApiKey) {
    console.warn("[email] RESEND_API_KEY not set; skipping send");
    return;
  }
  const r = await fetch("https://api.resend.com/emails", {
    method: "POST",
    headers: {
      Authorization: `Bearer ${resendApiKey}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      from: "Confide <hello@confide.knosi.xyz>",
      to: [to],
      subject: email.subject,
      html: email.html,
    }),
  });
  if (!r.ok) {
    console.error("Resend send failed:", await r.text());
  }
}
