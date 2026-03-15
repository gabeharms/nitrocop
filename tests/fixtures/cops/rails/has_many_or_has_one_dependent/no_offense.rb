class User < ApplicationRecord
  has_many :posts, dependent: :destroy
  has_one :profile, dependent: :nullify
  has_many :comments, through: :posts
  has_many :likes, dependent: :delete_all
  has_one :address, dependent: :destroy
  has_many :articles, dependent: nil
end

# ActiveResource::Base should not be flagged
class Resource < ActiveResource::Base
  has_many :items, class_name: 'API::Item'
end

class AnotherResource < ::ActiveResource::Base
  has_many :widgets, class_name: 'API::Widget'
end

# readonly? returning true suppresses offense
class Snapshot < ActiveRecord::Base
  has_one :data
  has_many :entries

  def readonly?
    true
  end
end

# with_options providing dependent
class Order < ApplicationRecord
  with_options dependent: :destroy do
    has_many :line_items
    has_one :invoice
  end
end

# with_options with dependent and explicit receiver
class Catalog < ApplicationRecord
  with_options dependent: :destroy do |model|
    model.has_many :products
  end
end

# with_options providing through (non-nil)
class Project < ApplicationRecord
  with_options through: :memberships do
    has_many :users
  end
end

# Nested with_options
class Blog < ApplicationRecord
  with_options dependent: :destroy do
    has_many :posts
    with_options class_name: 'Post' do
      has_many :drafts, foreign_key: :draft_id
    end
  end
end

# has_many with dependent and lambda scope
class Forum < ApplicationRecord
  has_many :topics, -> { order(created_at: :desc) }, dependent: :destroy
end

# has_many with through and lambda scope
class Activity < ApplicationRecord
  has_many :events, -> { order(created_at: :desc) }, through: :logs, source: :events
end

# has_one with double splat hash literal containing dependent
class Setting < ApplicationRecord
  has_one :preference, **{dependent: :destroy}
end

# with_options and association extension block
class Library < ApplicationRecord
  with_options dependent: :destroy do
    has_many :books do
      def available
      end
    end
  end
end
