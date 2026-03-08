User.where("status != ?", "active")
     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/WhereNot: Use `where.not(...)` instead of manually constructing negated SQL.

User.where("name IS NOT NULL")
     ^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/WhereNot: Use `where.not(...)` instead of manually constructing negated SQL.

Product.where("category NOT IN (?)", excluded)
        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/WhereNot: Use `where.not(...)` instead of manually constructing negated SQL.

User.where("name != :name", name: "Gabe")
     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/WhereNot: Use `where.not(...)` instead of manually constructing negated SQL.

User.where(["name != ?", "Gabe"])
     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/WhereNot: Use `where.not(...)` instead of manually constructing negated SQL.

User.where(["name IS NOT NULL"])
     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/WhereNot: Use `where.not(...)` instead of manually constructing negated SQL.

User.where(["name NOT IN (?)", ["john", "jane"]])
     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/WhereNot: Use `where.not(...)` instead of manually constructing negated SQL.

User.where(["name <> :name", { name: "Gabe" }])
     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/WhereNot: Use `where.not(...)` instead of manually constructing negated SQL.
