alter table
    groups drop column group_type;

alter table
    groups
add
    column root bool not null default false;