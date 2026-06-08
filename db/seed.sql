-- ============================================================
-- Seed Data (Development Only)
-- ============================================================

-- Super Admin user (password: Admin@123456)
-- bcrypt hash generated with cost 12
INSERT INTO users (
    id, email, password_hash, first_name, last_name,
    role, status, email_verified
) VALUES (
    '00000000-0000-0000-0000-000000000001',
    'admin@whatsup.dev',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewKTphISKYRTuoey',
    'Super', 'Admin',
    'super_admin', 'active', TRUE
) ON CONFLICT (email) DO NOTHING;

-- Demo Organization
INSERT INTO organizations (
    id, name, slug, owner_id, plan, status,
    max_contacts, max_campaigns, max_team_members, monthly_msg_quota
) VALUES (
    '00000000-0000-0000-0000-000000000010',
    'Demo Company', 'demo-company',
    '00000000-0000-0000-0000-000000000001',
    'professional', 'active',
    50000, 100, 10, 50000
) ON CONFLICT (slug) DO NOTHING;

-- Add super admin as owner of demo org
INSERT INTO org_members (organization_id, user_id, role)
VALUES (
    '00000000-0000-0000-0000-000000000010',
    '00000000-0000-0000-0000-000000000001',
    'owner'
) ON CONFLICT (organization_id, user_id) DO NOTHING;

-- Demo subscription (professional plan)
INSERT INTO org_subscriptions (
    organization_id, plan_id, status, billing_interval,
    current_period_start, current_period_end
)
SELECT
    '00000000-0000-0000-0000-000000000010',
    id, 'active', 'monthly',
    NOW(), NOW() + INTERVAL '30 days'
FROM subscription_plans WHERE name = 'professional'
ON CONFLICT DO NOTHING;
