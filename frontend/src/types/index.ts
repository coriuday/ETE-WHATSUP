export type UserRole = "super_admin" | "business_admin" | "team_member";

export interface User {
  id: string;
  email: string;
  fullName: string;
  role: UserRole;
  isEmailVerified: boolean;
  twoFactorEnabled: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface Organization {
  id: string;
  name: string;
  slug: string;
  createdAt: string;
  updatedAt: string;
}

export interface OrganizationMember {
  id: string;
  organizationId: string;
  userId: string;
  role: UserRole;
  user?: User;
  createdAt: string;
}

export interface WhatsAppAccount {
  id: string;
  organizationId: string;
  name: string;
  phoneNumber?: string;
  phoneNumberId?: string;
  wabaId?: string;
  status: "disconnected" | "connected" | "pending";
  profileName?: string;
  profilePictureUrl?: string;
  createdAt: string;
  updatedAt: string;
}

export interface Contact {
  id: string;
  organizationId: string;
  phoneNumber: string;
  firstName?: string;
  lastName?: string;
  email?: string;
  tags: string[];
  customFields: Record<string, any>;
  status: "active" | "unsubscribed" | "blocked";
  createdAt: string;
  updatedAt: string;
}

export type CampaignStatus = "draft" | "scheduled" | "running" | "completed" | "failed";
export type CampaignType = "promotional" | "transactional" | "reminder" | "survey";

export interface Campaign {
  id: string;
  organizationId: string;
  waAccountId: string;
  name: string;
  type: CampaignType;
  status: CampaignStatus;
  templateId?: string;
  messageBody?: string;
  scheduledAt?: string;
  startedAt?: string;
  completedAt?: string;
  totalRecipientCount: number;
  sentCount: number;
  deliveredCount: number;
  readCount: number;
  failedCount: number;
  createdAt: string;
  updatedAt: string;
}

export type MessageDirection = "inbound" | "outbound";
export type MessageStatus = "pending" | "sent" | "delivered" | "read" | "failed";

export interface Message {
  id: string;
  organizationId: string;
  waAccountId: string;
  campaignId?: string;
  contactId: string;
  waMessageId?: string;
  direction: MessageDirection;
  type: "text" | "image" | "document" | "template";
  body?: string;
  mediaUrl?: string;
  status: MessageStatus;
  sentAt?: string;
  deliveredAt?: string;
  readAt?: string;
  failedAt?: string;
  failureReason?: string;
  createdAt: string;
}

export interface MessageTemplate {
  id: string;
  organizationId: string;
  name: string;
  category: string;
  language: string;
  status: "draft" | "pending_approval" | "approved" | "rejected";
  bodyText: string;
  variables: string[];
  metaTemplateId?: string;
  createdAt: string;
  updatedAt: string;
}

export interface Conversation {
  id: string;
  organizationId: string;
  waAccountId: string;
  contactId: string;
  status: "open" | "resolved";
  isInSession: boolean;
  sessionExpiresAt?: string;
  unreadCount: number;
  lastMessageBody?: string;
  lastMessageDir?: MessageDirection;
  lastMessageAt: string;
  firstMessageAt: string;
  assignedTo?: string;
  contact?: Contact;
}

export interface SubscriptionPlan {
  id: string;
  name: string;
  code: string;
  monthlyPrice: number;
  yearlyPrice: number;
  maxContacts: number;
  maxMessagesPerMonth: number;
  maxTeamMembers: number;
  maxWhatsAppNumbers: number;
  features: string[];
}

export interface OrganizationSubscription {
  id: string;
  organizationId: string;
  planId: string;
  status: "active" | "past_due" | "canceled" | "trial";
  currentPeriodStart: string;
  currentPeriodEnd: string;
  plan?: SubscriptionPlan;
}
