-- Seed initial activity categories
INSERT INTO public.activity_categories (name, type, identifier, display_name) VALUES
    ('Microsoft Word', 'desktop_application', 'WINWORD.EXE', 'Microsoft Word'),
    ('Microsoft Excel', 'desktop_application', 'EXCEL.EXE', 'Microsoft Excel'),
    ('Visual Studio Code', 'desktop_application', 'Code.exe', 'Visual Studio Code'),
    ('Slack', 'desktop_application', 'slack.exe', 'Slack'),
    ('Gmail', 'browser', 'mail.google.com', 'Gmail'),
    ('Google Meet', 'browser', 'meet.google.com', 'Google Meet'),
    ('GitHub', 'browser', 'github.com', 'GitHub')
ON CONFLICT (type, identifier) DO NOTHING;
