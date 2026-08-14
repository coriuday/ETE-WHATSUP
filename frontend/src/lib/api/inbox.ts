import { api } from "./client";

export async function listConversations(params: Record<string, unknown> = {}) {
  const res = await api.get("/conversations", { params });
  return res.data?.data ?? res.data;
}

export async function getConversation(id: string) {
  const res = await api.get(`/conversations/${id}`);
  return res.data?.data ?? res.data;
}

export async function getConversationMessages(id: string) {
  const res = await api.get(`/conversations/${id}/messages`);
  return res.data?.data ?? res.data;
}

export async function sendConversationMessage(id: string, body: Record<string, unknown>) {
  const res = await api.post(`/conversations/${id}/messages`, body);
  return res.data?.data ?? res.data;
}

export async function resolveConversation(id: string) {
  const res = await api.post(`/conversations/${id}/resolve`);
  return res.data;
}

export async function reopenConversation(id: string) {
  const res = await api.post(`/conversations/${id}/reopen`);
  return res.data;
}

export async function assignConversation(id: string, userId: string | null) {
  const res = await api.put(`/conversations/${id}/assign`, { user_id: userId });
  return res.data;
}

export async function listNotes(id: string) {
  const res = await api.get(`/conversations/${id}/notes`);
  return res.data?.data ?? res.data;
}

export async function addNote(id: string, body: string) {
  const res = await api.post(`/conversations/${id}/notes`, { body });
  return res.data;
}
