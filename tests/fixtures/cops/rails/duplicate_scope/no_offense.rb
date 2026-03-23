class Post < ApplicationRecord
  scope :published, -> { where(published: true) }
  scope :draft, -> { where(published: false) }
  scope :recent, -> { order(created_at: :desc) }
  scope :featured, -> { where(featured: true) }
end

# Same name, different body — NOT a duplicate expression
class Stance < ApplicationRecord
  scope :guests, -> { where(guest: true) }
  scope :guests, -> { where("inviter_id is not null") }
end

# lambda { } and -> { } are different AST node types; RuboCop does not
# normalise them, so they are NOT considered duplicates.
class Widget < ApplicationRecord
  scope :scope_with_lambda, lambda { all }
  scope :anonymous_extension, -> { all } do
    def one
      1
    end
  end
end

# A single bodyless scope is not a duplicate
class Singleton < ApplicationRecord
  scope :default
end
