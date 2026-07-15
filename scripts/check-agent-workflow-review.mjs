import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

const baselinePath = fileURLToPath(
  new URL("../docs/agent-workflow/BASELINES.md", import.meta.url),
);

function readReviewMetadata() {
  const lines = readFileSync(baselinePath, "utf8").split(/\r?\n/u);
  const sectionStart = lines.findIndex((line) => line.trim() === "## 复审状态");

  if (sectionStart === -1) {
    throw new Error("BASELINES.md 缺少“复审状态”章节。");
  }

  const metadata = new Map();

  for (const line of lines.slice(sectionStart + 1)) {
    if (line.startsWith("## ")) {
      break;
    }

    const match = line.match(/^\|\s*`([^`]+)`\s*\|\s*`([^`]+)`\s*\|\s*$/u);
    if (match === null) {
      continue;
    }

    const [, field, value] = match;
    if (metadata.has(field)) {
      throw new Error(`复审状态包含重复字段：${field}`);
    }

    metadata.set(field, value);
  }

  const requiredFields = [
    "last-reviewed",
    "review-interval-days",
    "review-timezone",
    "next-review",
  ];

  for (const field of requiredFields) {
    if (!metadata.has(field)) {
      throw new Error(`复审状态缺少字段：${field}`);
    }
  }

  return metadata;
}

function parseIsoDate(value, field) {
  if (!/^\d{4}-\d{2}-\d{2}$/u.test(value)) {
    throw new Error(`${field} 必须使用 YYYY-MM-DD 格式：${value}`);
  }

  const date = new Date(`${value}T00:00:00.000Z`);
  if (Number.isNaN(date.valueOf()) || date.toISOString().slice(0, 10) !== value) {
    throw new Error(`${field} 不是有效日期：${value}`);
  }

  return date;
}

function addDays(date, days) {
  const result = new Date(date);
  result.setUTCDate(result.getUTCDate() + days);

  if (Number.isNaN(result.valueOf())) {
    throw new Error("复审日期超出支持范围。");
  }

  return result.toISOString().slice(0, 10);
}

function currentDateInTimeZone(timeZone) {
  let formatter;

  try {
    formatter = new Intl.DateTimeFormat("en-US", {
      day: "2-digit",
      month: "2-digit",
      timeZone,
      year: "numeric",
    });
  } catch {
    throw new Error(`review-timezone 不是有效的 IANA 时区：${timeZone}`);
  }

  const parts = Object.fromEntries(
    formatter
      .formatToParts(new Date())
      .filter(({ type }) => type !== "literal")
      .map(({ type, value }) => [type, value]),
  );

  return `${parts.year}-${parts.month}-${parts.day}`;
}

try {
  const { values: arguments_ } = parseArgs({
    allowPositionals: false,
    args: process.argv.slice(2),
    options: {
      today: { type: "string" },
      validate: { default: false, type: "boolean" },
    },
    strict: true,
  });
  const todayArgument = arguments_.today;
  const validateOnly = arguments_.validate;
  const metadata = readReviewMetadata();
  const lastReviewed = metadata.get("last-reviewed");
  const intervalText = metadata.get("review-interval-days");
  const timeZone = metadata.get("review-timezone");
  const recordedNextReview = metadata.get("next-review");

  const lastReviewedDate = parseIsoDate(lastReviewed, "last-reviewed");
  parseIsoDate(recordedNextReview, "next-review");

  if (!/^[1-9]\d*$/u.test(intervalText)) {
    throw new Error(`review-interval-days 必须是正整数：${intervalText}`);
  }

  const intervalDays = Number(intervalText);
  if (!Number.isSafeInteger(intervalDays)) {
    throw new Error(`review-interval-days 超出安全整数范围：${intervalText}`);
  }

  const expectedNextReview = addDays(lastReviewedDate, intervalDays);
  if (recordedNextReview !== expectedNextReview) {
    throw new Error(
      `next-review 应为 ${expectedNextReview}，当前为 ${recordedNextReview}。`,
    );
  }

  const currentDate = currentDateInTimeZone(timeZone);
  const today = todayArgument ?? currentDate;
  parseIsoDate(today, "today");

  const due = today >= expectedNextReview;
  const state = due ? "已到期" : "未到期";
  console.log(
    `Agent 工作流复审${state}：today=${today}，next-review=${expectedNextReview}，timezone=${timeZone}`,
  );

  if (due && !validateOnly) {
    process.exitCode = 2;
  }
} catch (error) {
  const message = error instanceof Error ? error.message : String(error);
  console.error(`Agent 工作流复审检查失败：${message}`);
  process.exitCode = 1;
}
