```sql
SELECT 
  *
FROM
  'org.json' AS orgs 
  INNER JOIN 'user.csv' AS users ON orgs.id = users.organization_id
WHERE role = 'member';
```

```sql
SELECT * FROM call_mcp('time', 'get_current_time', {'timezone': 'UTC'});
```

```sql
SELECT html_url, created_at, title, "user"['login'] FROM call_mcp('github', 'list_issues', {'owner': 'github', 'repo': 'github-mcp-server', 'perPage': 10});
```
