-- Seed default system permissions and roles

-- Insert Core Permissions
INSERT INTO permissions (id, name, description) VALUES
    (gen_random_uuid(), 'user:create', 'Create new users inside organization'),
    (gen_random_uuid(), 'user:read', 'View user profiles and members'),
    (gen_random_uuid(), 'user:update', 'Update user member details'),
    (gen_random_uuid(), 'user:delete', 'Remove users from organization'),
    
    (gen_random_uuid(), 'organization:create', 'Create new organizations'),
    (gen_random_uuid(), 'organization:read', 'View organization details'),
    (gen_random_uuid(), 'organization:update', 'Update organization settings'),
    (gen_random_uuid(), 'organization:delete', 'Delete organization'),
    (gen_random_uuid(), 'organization:invite', 'Invite members to organization'),

    (gen_random_uuid(), 'role:create', 'Create custom organization roles'),
    (gen_random_uuid(), 'role:read', 'View roles and permission assignments'),
    (gen_random_uuid(), 'role:update', 'Modify custom roles'),
    (gen_random_uuid(), 'role:delete', 'Delete custom roles'),

    (gen_random_uuid(), 'permission:create', 'Create new custom permissions'),
    (gen_random_uuid(), 'permission:read', 'View available permissions'),
    (gen_random_uuid(), 'permission:assign', 'Assign permissions to roles'),

    (gen_random_uuid(), 'api_key:create', 'Generate API keys'),
    (gen_random_uuid(), 'api_key:read', 'View API keys metadata'),
    (gen_random_uuid(), 'api_key:delete', 'Revoke API keys'),

    (gen_random_uuid(), 'session:read', 'View active sessions'),
    (gen_random_uuid(), 'session:revoke', 'Revoke active sessions'),

    (gen_random_uuid(), 'audit:read', 'View audit trail logs')
ON CONFLICT (name) DO NOTHING;

-- Create Global System Roles (organization_id IS NULL)
INSERT INTO roles (id, organization_id, name, description, is_system) VALUES
    ('00000000-0000-0000-0000-000000000001', NULL, 'Owner', 'Full administrative access across all organization resources', true),
    ('00000000-0000-0000-0000-000000000002', NULL, 'Admin', 'Administrative access excluding org deletion and owner transfer', true),
    ('00000000-0000-0000-0000-000000000003', NULL, 'Manager', 'Can manage users, API keys, and view reports', true),
    ('00000000-0000-0000-0000-000000000004', NULL, 'Viewer', 'Read-only access to organization resources', true)
ON CONFLICT (organization_id, name) DO NOTHING;

-- Assign all permissions to Owner role
INSERT INTO role_permissions (role_id, permission_id)
SELECT '00000000-0000-0000-0000-000000000001', id FROM permissions
ON CONFLICT DO NOTHING;

-- Assign Admin permissions (everything except organization:delete)
INSERT INTO role_permissions (role_id, permission_id)
SELECT '00000000-0000-0000-0000-000000000002', id FROM permissions
WHERE name != 'organization:delete'
ON CONFLICT DO NOTHING;

-- Assign Manager permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT '00000000-0000-0000-0000-000000000003', id FROM permissions
WHERE name IN ('user:read', 'user:invite', 'role:read', 'permission:read', 'api_key:create', 'api_key:read', 'api_key:delete', 'session:read', 'audit:read', 'organization:read')
ON CONFLICT DO NOTHING;

-- Assign Viewer permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT '00000000-0000-0000-0000-000000000004', id FROM permissions
WHERE name IN ('user:read', 'role:read', 'permission:read', 'api_key:read', 'organization:read')
ON CONFLICT DO NOTHING;
