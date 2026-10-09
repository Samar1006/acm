-- Public examples shown in the description and run by the Tests button.
create table problem_examples (
    id integer primary key not null,
    problem_id integer not null,
    example_number integer not null,
    input text not null,
    expected_output text not null,
    explanation text not null default '',

    unique (problem_id, example_number),
    foreign key (problem_id) references problems(id)
);

-- NULL uses the default: required, except for competition problems.
alter table problems add require_tests boolean;

-- The latest code that passed a problem's examples, per user and language.
create table test_passes (
    user_id integer not null,
    problem_id integer not null,
    language text not null,
    code_hash text not null,
    time datetime not null default (datetime('now')),

    primary key (user_id, problem_id, language),
    foreign key (user_id) references users(id),
    foreign key (problem_id) references problems(id)
);

create table drafts (
    user_id integer not null,
    problem_id integer not null,
    language text not null,
    code text not null,
    update_dt datetime not null default (datetime('now')),

    primary key (user_id, problem_id, language),
    foreign key (user_id) references users(id),
    foreign key (problem_id) references problems(id)
);

create table bookmarks (
    user_id integer not null,
    problem_id integer not null,
    create_dt datetime not null default (datetime('now')),

    primary key (user_id, problem_id),
    foreign key (user_id) references users(id),
    foreign key (problem_id) references problems(id)
);

create index problem_tags_problem on problem_tags (problem_id);
create index problem_tags_name on problem_tags (name);

-- One of Accepted, WrongAnswer, RuntimeError, TimeLimitExceeded,
-- CompilationError, InternalError. Compilation errors stay visible to their
-- author but are not counted as submissions anywhere.
alter table submissions add verdict text;

update submissions set verdict = case
    when success then 'Accepted'
    when error like '[%' then 'CompilationError'
    when error like 'Manually overridden%' then 'WrongAnswer'
    when error like 'Process took too long%' then 'TimeLimitExceeded'
    when error like 'encountered a runtime error%' then 'RuntimeError'
    when error is not null then 'InternalError'
    when exists (
        select 1 from test_results
        where test_results.submission_id = submissions.id
          and not test_results.success
          and test_results.error like '%fuel%'
    ) then 'TimeLimitExceeded'
    when exists (
        select 1 from test_results
        where test_results.submission_id = submissions.id
          and not test_results.success
          and test_results.error is not null
    ) then 'RuntimeError'
    else 'WrongAnswer'
end;
