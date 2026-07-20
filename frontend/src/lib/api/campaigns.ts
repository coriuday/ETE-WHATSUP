import { api } from "./client";

export interface CampaignListParams {
  page?: number;
  limit?: number;
  status?: string;
}

export async function listCampaigns(params: CampaignListParams = {}) {
  const res = await api.get("/campaigns", { params });
  return res.data?.data ?? res.data;
}
