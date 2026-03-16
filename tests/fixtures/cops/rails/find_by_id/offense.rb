User.find_by_id!(id)
     ^^^^^^^^^^^^ Rails/FindById: Use `find` instead of `find_by_id!`.
User.find_by!(id: id)
     ^^^^^^^^ Rails/FindById: Use `find` instead of `find_by!`.
User.where(id: id).take!
     ^^^^^ Rails/FindById: Use `find` instead of `where(id: ...).take!`.
User&.find_by_id!(1)
      ^^^^^^^^^^^^ Rails/FindById: Use `find` instead of `find_by_id!`.
User&.find_by!(id: 1)
      ^^^^^^^^ Rails/FindById: Use `find` instead of `find_by!`.
User.where(id: 1)&.take!
     ^^^^^ Rails/FindById: Use `find` instead of `where(id: ...).take!`.
User&.where(id: 1)&.take!
      ^^^^^ Rails/FindById: Use `find` instead of `where(id: ...).take!`.
find_by!(id: from_external_id(external_id))
^^^^^^^^ Rails/FindById: Use `find` instead of `find_by!`.
find_by_id!(record_id)
^^^^^^^^^^^ Rails/FindById: Use `find` instead of `find_by_id!`.
