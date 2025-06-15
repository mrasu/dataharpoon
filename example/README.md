# Examples

## Natural Language via MCP

### 1. Query local files using natural language

> With DataHarpoon, please tell me the number of users (written in user.csv) with the role 'member' in each organization (written in org.json).

MCP will generate and execute the following SQL:

```sql
SELECT o.name AS organization_name, COUNT(u.id) AS member_count
FROM 'user.csv' u
JOIN 'org.json' o ON u.organization_id = o.id
WHERE u.role = 'member'
GROUP BY o.id, o.name
ORDER BY o.name;
```

and shows:

```
Member Count by Organization
| Organization Name | Number of Members |
|-------------------|-------------------|
| EduCore | 1 |
| GreenFuture | 1 |
| HealthPlus | 2 |
| Tech Innovators | 2 |
```

https://github.com/user-attachments/assets/46089765-b3d5-4c58-83e2-71a0e21a8db9


### 2. Ask to call MCP

> With DataHarpoon, can you provide three issues from the github/github-mcp-server repository on GitHub?

MCP will generate and execute the following SQL:
```sql
SELECT * FROM call_mcp('github', 'list_issues', {'owner': 'github', 'repo': 'github-mcp-server', 'perPage': 3}) LIMIT 3
```

## SQL via CLI

### 1. Call MCP

```sql
-- Call mcp/time (https://github.com/modelcontextprotocol/servers/tree/main/src/time)
SELECT * FROM call_mcp('time', 'get_current_time', {'timezone': 'UTC'});

#=>
+---------------------------+--------+----------+
| datetime                  | is_dst | timezone |
+---------------------------+--------+----------+
| 2025-05-25T16:31:38+00:00 | false  | UTC      |
+---------------------------+--------+----------+
```

### 2. Retrieve GitHub issues from `github/github-mcp-server` and categorize them using Claude

The following SQL query retrieves issues from the `github/github-mcp-server` repository and asks Claude to classify them as either bugs or feature requests based on their titles and bodies.

```sql
SELECT
  html_url,
  title,
  exec_mcp(
    'claude',
    'chat-with-claude', 
    {'content': 'Classify the following GitHub issue as "bug", "feature request" or "other". Reply with the classification only.<title> ' || title || '</title><body>' || body || '</body>'}
  ) AS category,
  "user"['login'] AS user
FROM
  call_mcp('github', 'list_issues', {'owner': 'github', 'repo': 'github-mcp-server'})
WHERE pull_request IS NULL
LIMIT 5;

#=>
+--------------------------------------------------------+------------------------------------------------------------------------------------------------------------+-----------------+----------------+
| html_url                                               | title                                                                                                      | category        | user           |
+--------------------------------------------------------+------------------------------------------------------------------------------------------------------------+-----------------+----------------+
| https://github.com/github/github-mcp-server/issues/520 | 64-Character Limit on Tool Names Conflicts with MCP Spec — Should Be Removed or Configurable               | feature request | jlwainwright   |
| https://github.com/github/github-mcp-server/issues/519 | [Visual Studio] cannot connect to remote MCP server `"Invalid content type: must be 'application/json'\n"` | bug             | xperiandri     |
| https://github.com/github/github-mcp-server/issues/517 | Add cursor install info to README.md                                                                       | other           | maxs10-creator |
| https://github.com/github/github-mcp-server/issues/507 | Regression in `get_file_contents` making it return `nil` in latest image `3e32f75`                         | bug             | monotykamary   |
| https://github.com/github/github-mcp-server/issues/504 | git blame tool (to get the latest contributors of the class)                                               | feature request | ismurygin      |
+--------------------------------------------------------+------------------------------------------------------------------------------------------------------------+-----------------+----------------+
```

### 3. Query local files

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
