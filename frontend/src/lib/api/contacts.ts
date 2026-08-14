import { api } from "./client";

export interface ContactListParams {
  page?: number;
  limit?: number;
  search?: string;
  tags?: string;
}

export async function listContacts(params: ContactListParams = {}) {
  const res = await api.get("/contacts", { params });
  return res.data?.data ?? res.data;
}
