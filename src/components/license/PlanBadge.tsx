import { useTranslation } from "react-i18next";
import type { UserPlan } from "../../lib/types";

export function PlanBadge({ plan }: { plan: UserPlan }) {
  const { t } = useTranslation();
  const used = Math.floor(plan.used_this_month_seconds / 60);
  const total = Math.floor(plan.monthly_quota_seconds / 60);
  const tierLabel =
    plan.tier === "free"
      ? t("billing.freePlan")
      : plan.tier === "pro"
      ? t("billing.proPlan")
      : t("billing.ultraPlan");

  return (
    <div className="text-xs flex items-center gap-2">
      <span
        className={`px-2 py-0.5 rounded ${
          plan.tier === "free"
            ? "bg-gray-700 text-gray-300"
            : plan.tier === "pro"
            ? "bg-purple-900 text-purple-200"
            : "bg-yellow-900 text-yellow-200"
        }`}
      >
        {tierLabel}
      </span>
      <span className="text-gray-400">
        {t("billing.minRemaining", { used, total })}
      </span>
    </div>
  );
}
