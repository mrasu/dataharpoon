```sql
SELECT 
  *
FROM
  'org.json' AS orgs 
  INNER JOIN 'user.csv' AS users ON orgs.id = users.organization_id
WHERE role = 'member';
```
