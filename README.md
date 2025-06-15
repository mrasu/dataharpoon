# DataHarpoon

An MCP-ready query engine that connects to your data — wherever it lives

DataHarpoon lets you query both raw data files and MCP-generated results with:
* Natural-language via MCP
* Raw SQL for precise control

# Examples

## Natural Language via MCP

You can write in plain English, and MCP automatically generates and runs the SQL.

When you ask:

> With DataHarpoon, please tell me the number of users (written in user.csv) with the role 'member' in each organization (written in org.json).

MCP will query:

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

### Call MCP

You can also use SQL directly.

The following SQL query retrieves issues from GitHub and asks Claude to classify them as either bugs or feature requests based on their titles and bodies.

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

Refer [example/README.md](./example/README.md) for more examples.

# How to use

After building, you can use DataHarpoon as an MCP server or run it via the CLI.

## Build

```shell
git clone git@github.com:mrasu/dataharpoon.git
cd dataharpoon
cargo build

# file will be in ./target/debug/dataharpoon
```

## Run via CLI

```shell
cd example
./../target/debug/dataharpoon
```

## Run with your own configuration

1. Create a `data_harpoon.toml`
2. Run the DataHarpoon binary with your config.

## Run as an MCP Server

Configure settings for your Agent.

```json
{
  "mcpServers": {
    "dataharpoon": {
      "command": "/path/to/dataharpoon",
      "args": [
        "serve",
        "mcp",
        "-c",
        "/path/to/data_harpoon.toml"
      ]
    }
  }
}
```
