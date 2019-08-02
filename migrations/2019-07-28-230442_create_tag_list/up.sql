create or replace function tag_list(_type bucket, _in set[], _ex set[], _offset int8, _limit int8)
returns set[] as $$
    with q as (
        select t.set
            from set_tag as t
            where array[t.tag] <@ _in
            group by t.set
        except
        select t.set
            from set_tag as t
            where array[t.tag] <@ _ex
            group by t.set
    ), f as (
        select t.tag
            from set_tag as t
            where array[t.set] <@ (select array_agg(q.set) ::set[] from q) and (tag).bucket = _type
            group by t.tag
            limit _limit
            offset _offset
    )
    select coalesce(array_agg(f.tag), array[] ::set[]) from f;

$$ language sql stable;