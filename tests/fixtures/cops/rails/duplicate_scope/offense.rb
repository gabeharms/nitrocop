class Post < ApplicationRecord
  scope :visible, -> { where(visible: true) }
  ^^^^^ Rails/DuplicateScope: Multiple scopes share this same expression.
  scope :shown, -> { where(visible: true) }
  ^^^^^ Rails/DuplicateScope: Multiple scopes share this same expression.
end

class User < ApplicationRecord
  scope :active, -> { where(active: true) }
  ^^^^^ Rails/DuplicateScope: Multiple scopes share this same expression.
  scope :enabled, -> { where(active: true) }
  ^^^^^ Rails/DuplicateScope: Multiple scopes share this same expression.
end

class Order < ApplicationRecord
  scope :recent, -> { order(created_at: :desc) }
  ^^^^^ Rails/DuplicateScope: Multiple scopes share this same expression.
  scope :pending, -> { where(status: "pending") }
  scope :newest, -> { order(created_at: :desc) }
  ^^^^^ Rails/DuplicateScope: Multiple scopes share this same expression.
end

class Item < ApplicationRecord
  scope :base, -> { all }
  ^^^^^ Rails/DuplicateScope: Multiple scopes share this same expression.
  scope :default_scope, -> { all }
  ^^^^^ Rails/DuplicateScope: Multiple scopes share this same expression.
end

# Bodyless scopes share the same (nil) expression
class Config < ApplicationRecord
  scope :all
  ^^^^^ Rails/DuplicateScope: Multiple scopes share this same expression.
  scope :unpublished
  ^^^^^ Rails/DuplicateScope: Multiple scopes share this same expression.
  scope :published
  ^^^^^ Rails/DuplicateScope: Multiple scopes share this same expression.
  scope :no_batch_actions
  ^^^^^ Rails/DuplicateScope: Multiple scopes share this same expression.
end
