```sql
CREATE EXTENSION ebs_fdw;

CREATE FOREIGN DATA WRAPPER ebs HANDLER ebs_fdw_handler VALIDATOR ebs_fdw_validator;
CREATE SERVER opfa FOREIGN DATA WRAPPER ebs OPTIONS (region 'eu-west-3');
CREATE FOREIGN TABLE ebs_volumes (id TEXT, name TEXT, type TEXT, size INT, encrypted BOOL) SERVER opfa;

SELECT * FROM ebs_volumes;
```