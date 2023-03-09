```sql
DROP EXTENSION IF EXISTS idi CASCADE;
CREATE EXTENSION idi;
```

```sql
SELECT is_valid_fiscal_code('RSSMRA85T10A562S');

CREATE TABLE users (id INT, cf TEXT CHECK(is_valid_fiscal_code(cf)));
INSERT INTO users (id, cf) VALUES (1, 'asd');
```

```sql
SELECT emojify('rocket');
```