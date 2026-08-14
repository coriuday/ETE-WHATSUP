import { api } from "./client";

export async function getOverview() {
  const res = await api.get("/analytics/overview");
  return res.data?.data ?? res.data;
}

export async function getCampaignAnalytics() {
  const res = await api.get("/analytics/campaigns");
  return res.data?.data ?? res.data;
}

export async function getMessageAnalytics() {
  const res = await api.get("/analytics/messages");
  return res.data?.data ?? res.data;
}
