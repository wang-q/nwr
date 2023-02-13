# division

## Description

<details>
<summary><strong>Table Definition</strong></summary>

```sql
CREATE TABLE division (
    id       INTEGER      NOT NULL
                          PRIMARY KEY,
    division VARCHAR (50) NOT NULL
)
```

</details>

## Columns

| Name | Type | Default | Nullable | Children | Parents | Comment |
| ---- | ---- | ------- | -------- | -------- | ------- | ------- |
| id | INTEGER |  | false | [node](node.md) |  |  |
| division | VARCHAR (50) |  | false |  |  |  |

## Constraints

| Name | Type | Definition |
| ---- | ---- | ---------- |
| id | PRIMARY KEY | PRIMARY KEY (id) |

## Relations

![er](division.svg)

---

> Generated by [tbls](https://github.com/k1LoW/tbls)