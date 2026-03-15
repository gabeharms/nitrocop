class User < ApplicationRecord
  has_many :posts
  has_many :posts, dependent: :destroy
  ^^^^^^^^ Rails/DuplicateAssociation: Duplicate association `posts` detected.
end

class Post < ApplicationRecord
  belongs_to :author
  belongs_to :author, optional: true
  ^^^^^^^^^^ Rails/DuplicateAssociation: Duplicate association `author` detected.
end

class Company < ApplicationRecord
  has_one :address
  has_one :address, dependent: :destroy
  ^^^^^^^ Rails/DuplicateAssociation: Duplicate association `address` detected.
end

# has_and_belongs_to_many duplicates
class Tag < ApplicationRecord
  has_and_belongs_to_many :articles
  has_and_belongs_to_many :articles
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/DuplicateAssociation: Duplicate association `articles` detected.
end

# String argument instead of symbol
class Invoice < ApplicationRecord
  has_many 'items'
  has_many 'items', dependent: :destroy
  ^^^^^^^^ Rails/DuplicateAssociation: Duplicate association `items` detected.
end

# class_name duplicate detection (has_many, not belongs_to)
class Account < ApplicationRecord
  has_many :foos, class_name: 'Foo'
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/DuplicateAssociation: Association `class_name: 'Foo'` is defined multiple times. Don't repeat associations.
  has_many :bars, class_name: 'Foo'
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/DuplicateAssociation: Association `class_name: 'Foo'` is defined multiple times. Don't repeat associations.
end

# class_name duplicate detection (has_one)
class Profile < ApplicationRecord
  has_one :baz, class_name: 'Bar'
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/DuplicateAssociation: Association `class_name: 'Bar'` is defined multiple times. Don't repeat associations.
  has_one :qux, class_name: 'Bar'
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/DuplicateAssociation: Association `class_name: 'Bar'` is defined multiple times. Don't repeat associations.
end
