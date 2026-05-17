CREATE TABLE IF NOT EXISTS daily_active_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip INET NOT NULL,
    date DATE NOT NULL DEFAULT CURRENT_DATE,
    UNIQUE(ip, date)
);

CREATE TABLE IF NOT EXISTS page_views (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip INET NOT NULL,
    page TEXT NOT NULL,
    viewed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS feedback (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    submitter_ip INET,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
