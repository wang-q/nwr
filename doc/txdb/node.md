# node

## Description

<details>
<summary><strong>Table Definition</strong></summary>

```sql
CREATE TABLE node (
    tax_id        INTEGER      NOT NULL
                               PRIMARY KEY,
    parent_tax_id INTEGER,
    rank          VARCHAR (25) NOT NULL,
    division_id   INTEGER      NOT NULL,
    comment       TEXT,
    FOREIGN KEY (
        division_id
    )
    REFERENCES division (id)
)
```

</details>

## Columns

| Name | Type | Default | Nullable | Children | Parents | Comment |
| ---- | ---- | ------- | -------- | -------- | ------- | ------- |
| tax_id | INTEGER |  | false |  |  |  |
| parent_tax_id | INTEGER |  | true |  |  |  |
| rank | VARCHAR (25) |  | false |  |  |  |
| division_id | INTEGER |  | false |  | [division](division.md) |  |
| comment | TEXT |  | true |  |  |  |

## Constraints

| Name | Type | Definition |
| ---- | ---- | ---------- |
| tax_id | PRIMARY KEY | PRIMARY KEY (tax_id) |
| - (Foreign key ID: 0) | FOREIGN KEY | FOREIGN KEY (division_id) REFERENCES division (id) ON UPDATE NO ACTION ON DELETE NO ACTION MATCH NONE |

## Indexes

| Name | Definition |
| ---- | ---------- |
| idx_node_parent_id | CREATE INDEX idx_node_parent_id ON node(parent_tax_id) |

## Relations

![er](node.svg)

---

> Generated by [tbls](https://github.com/k1LoW/tbls)
