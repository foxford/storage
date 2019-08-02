create table set_tag (
    id uuid default gen_random_uuid(),

    tag set not null,
    set set not null,

    created_at timestamptz not null default now(),

    check(((tag).label) is not null),
    check(((tag).bucket).label is not null),
    check(((tag).bucket).audience is not null),
    check(((set).label) is not null),
    check(((set).bucket).label is not null),
    check(((set).bucket).audience is not null),
    unique(tag, set),
    primary key (id)
)