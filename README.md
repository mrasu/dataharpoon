# DataHarpoon

An MCP-ready database that connects to your data — wherever it lives

# Examples

Refer [example](./example)

## Query files

```sql
SELECT 
  *
FROM
  'org.json' AS orgs 
  INNER JOIN 'user.csv' AS users ON orgs.id = users.organization_id
WHERE role = 'member';

#=>
+------------+------+------------------------+-------------------+-----------------+----+-----------------+------------+------------------------+------------+--------+
| created_at | id   | industry               | location          | name            | id | organization_id | user_name  | email                  | join_date  | role   |
+------------+------+------------------------+-------------------+-----------------+----+-----------------+------------+------------------------+------------+--------+
| 2022-04-18 | 1005 | Environmental Services | Seattle, WA       | GreenFuture     | 9  | 1005            | raj_kumar  | raj.kumar@example.com  | 2025-03-12 | member |
| 2020-11-10 | 1003 | Education              | Boston, MA        | EduCore         | 8  | 1003            | lisa_chan  | lisa.chan@example.com  | 2025-03-09 | member |
| 2018-06-12 | 1001 | Software               | San Francisco, CA | Tech Innovators | 1  | 1001            | john_doe   | john.doe@example.com   | 2025-01-15 | member |
| 2018-06-12 | 1001 | Software               | San Francisco, CA | Tech Innovators | 6  | 1001            | nina_patel | nina.patel@example.com | 2025-03-05 | member |
| 2019-03-21 | 1002 | Healthcare             | New York, NY      | HealthPlus      | 3  | 1002            | alice_wong | alice.wong@example.com | 2025-02-02 | member |
| 2019-03-21 | 1002 | Healthcare             | New York, NY      | HealthPlus      | 5  | 1002            | chris_lee  | chris.lee@example.com  | 2025-03-01 | member |
+------------+------+------------------------+-------------------+-----------------+----+-----------------+------------+------------------------+------------+--------+

```

## Call MCP

```sql
-- call mcp/time (https://github.com/modelcontextprotocol/servers/tree/main/src/time)
SELECT * FROM call_mcp('time', 'get_current_time', {'timezone': 'UTC'});

#=>
+---------------------------+--------+----------+
| datetime                  | is_dst | timezone |
+---------------------------+--------+----------+
| 2025-05-25T16:31:38+00:00 | false  | UTC      |
+---------------------------+--------+----------+
```

```sql
-- call GitHub's MCP (https://github.com/github/github-mcp-server/)
SELECT html_url, created_at, title, "user"['login'] FROM call_mcp('github', 'list_issues', {'owner': 'github', 'repo': 'github-mcp-server', 'perPage': 10});

#=>
+--------------------------------------------------------+----------------------+---------------------------------------------------------------------------------------+-----------------------+
| html_url                                               | created_at           | title                                                                                 | tmp_table.user[login] |
+--------------------------------------------------------+----------------------+---------------------------------------------------------------------------------------+-----------------------+
| https://github.com/github/github-mcp-server/pull/433   | 2025-05-24T20:08:41Z | github-mcp-server with Claude Code not working                                        | aryasoni98            |
| https://github.com/github/github-mcp-server/issues/432 | 2025-05-24T18:34:33Z | Repository Access Management                                                          | dschmag               |
| https://github.com/github/github-mcp-server/issues/430 | 2025-05-23T21:29:40Z | Pagination not working well                                                           | SamMorrowDrums        |
| https://github.com/github/github-mcp-server/pull/428   | 2025-05-23T11:35:31Z | Add opt-in filtering for content from users without push access                       | Copilot               |
| https://github.com/github/github-mcp-server/issues/427 | 2025-05-23T11:35:21Z | Add an opt-in way to limit issue, comment and PR input from users without push access | SamMorrowDrums        |
| https://github.com/github/github-mcp-server/pull/426   | 2025-05-23T10:45:44Z | [WIP] Invisible character filtering                                                   | Copilot               |
| https://github.com/github/github-mcp-server/pull/424   | 2025-05-22T08:24:28Z | Add ability to manage and list starred repositories                                   | LukasPoque            |
| https://github.com/github/github-mcp-server/pull/423   | 2025-05-22T06:25:56Z | feat: Add mark_pr_ready_for_review tool                                               | efouts                |
| https://github.com/github/github-mcp-server/issues/422 | 2025-05-21T23:24:05Z | Support completions for GH resources                                                  | connor4312            |
| https://github.com/github/github-mcp-server/issues/420 | 2025-05-21T11:36:50Z | Failed API calls should not be failures, not errors                                   | SamMorrowDrums        |
+--------------------------------------------------------+----------------------+---------------------------------------------------------------------------------------+-----------------------+
```

# How to use

## Run examples

```shell
git clone git@github.com:mrasu/dataharpoon.git
cd dataharpoon
cargo build
cd example
./../target/debug/dataharpoon
```

## Run yours
1. Write `data_harpoon.toml`
2. Run dataharpoon
