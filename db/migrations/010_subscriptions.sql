-- ============================================================
-- Migration 010: Subscriptions & Billing
-- ============================================================

CREATE TYPE subscription_plan AS ENUM (
    'free',
    'starter',
    'professional',
    'enterprise'
);

CREATE TYPE subscription_status AS ENUM (
    'trialing',
    'active',
    'past_due',
    'cancelled',
    'expired'
);

CREATE TYPE billing_interval AS ENUM ('monthly', 'yearly');

-- ── Plan Definitions (configurable by super admin) ───────────
CREATE TABLE IF NOT EXISTS subscription_plans (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name                subscription_plan NOT NULL UNIQUE,
    display_name        VARCHAR(100) NOT NULL,
    description         TEXT,

    -- Pricing
    monthly_price_inr   DECIMAL(10, 2) NOT NULL DEFAULT 0,
    yearly_price_inr    DECIMAL(10, 2) NOT NULL DEFAULT 0,

    -- Limits
    max_contacts        INTEGER NOT NULL DEFAULT 1000,
    max_campaigns       INTEGER NOT NULL DEFAULT 10,
    max_team_members    INTEGER NOT NULL DEFAULT 3,
    max_wa_accounts     INTEGER NOT NULL DEFAULT 1,
    monthly_msg_quota   INTEGER NOT NULL DEFAULT 1000,

    -- Features
    features            TEXT[] NOT NULL DEFAULT '{}',
    is_active           BOOLEAN NOT NULL DEFAULT TRUE,

    -- Stripe
    stripe_product_id   VARCHAR(255),
    stripe_price_monthly_id VARCHAR(255),
    stripe_price_yearly_id  VARCHAR(255),

    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER update_subscription_plans_updated_at
    BEFORE UPDATE ON subscription_plans
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ── Organization Subscriptions ───────────────────────────────
CREATE TABLE IF NOT EXISTS org_subscriptions (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id     UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    plan_id             UUID NOT NULL REFERENCES subscription_plans(id),

    status              subscription_status NOT NULL DEFAULT 'trialing',
    billing_interval    billing_interval NOT NULL DEFAULT 'monthly',

    -- Stripe
    stripe_customer_id      VARCHAR(255),
    stripe_subscription_id  VARCHAR(255),

    -- Dates
    trial_ends_at       TIMESTAMPTZ,
    current_period_start TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    current_period_end  TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '30 days',
    cancelled_at        TIMESTAMPTZ,
    cancel_at_period_end BOOLEAN NOT NULL DEFAULT FALSE,

    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_org_subscriptions_org_id ON org_subscriptions(organization_id);
CREATE INDEX idx_org_subscriptions_status ON org_subscriptions(status);
CREATE INDEX idx_org_subscriptions_stripe_customer ON org_subscriptions(stripe_customer_id);

CREATE TRIGGER update_org_subscriptions_updated_at
    BEFORE UPDATE ON org_subscriptions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ── Billing Invoices ─────────────────────────────────────────
CREATE TABLE IF NOT EXISTS billing_invoices (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id     UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    subscription_id     UUID REFERENCES org_subscriptions(id) ON DELETE SET NULL,
    stripe_invoice_id   VARCHAR(255) UNIQUE,
    amount_inr          DECIMAL(10, 2) NOT NULL,
    status              VARCHAR(50) NOT NULL DEFAULT 'pending',   -- pending, paid, failed, refunded
    invoice_url         TEXT,
    pdf_url             TEXT,
    period_start        TIMESTAMPTZ,
    period_end          TIMESTAMPTZ,
    paid_at             TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_invoices_org_id ON billing_invoices(organization_id);

-- ── Usage Overage Tracking ───────────────────────────────────
CREATE TABLE IF NOT EXISTS usage_records (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    period_start    TIMESTAMPTZ NOT NULL,
    period_end      TIMESTAMPTZ NOT NULL,
    msgs_sent       INTEGER NOT NULL DEFAULT 0,
    contacts_count  INTEGER NOT NULL DEFAULT 0,
    campaigns_run   INTEGER NOT NULL DEFAULT 0,
    recorded_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_usage_records_org ON usage_records(organization_id);

-- ── Seed default plans ───────────────────────────────────────
INSERT INTO subscription_plans (name, display_name, description, monthly_price_inr, yearly_price_inr,
    max_contacts, max_campaigns, max_team_members, max_wa_accounts, monthly_msg_quota, features)
VALUES
    ('free', 'Free', 'Get started at no cost', 0, 0,
     500, 3, 1, 1, 500,
     ARRAY['basic_campaigns', 'contact_import', 'basic_analytics']),

    ('starter', 'Starter', 'Perfect for small businesses', 1499, 14990,
     5000, 20, 3, 1, 5000,
     ARRAY['basic_campaigns', 'contact_import', 'templates', 'scheduling', 'inbox', 'basic_analytics']),

    ('professional', 'Professional', 'For growing teams', 3999, 39990,
     50000, 100, 10, 3, 50000,
     ARRAY['all_starter', 'automation', 'ab_testing', 'advanced_analytics', 'api_access', 'drip_campaigns']),

    ('enterprise', 'Enterprise', 'Unlimited scale', 9999, 99990,
     -1, -1, -1, 10, -1,   -- -1 = unlimited
     ARRAY['all_professional', 'custom_integrations', 'dedicated_support', 'sla', 'white_label']);
