User.all.each { |u| u.save }
         ^^^^ Rails/FindEach: Use `find_each` instead of `each` for batch processing.
User.where(active: true).each { |u| u.save }
                         ^^^^ Rails/FindEach: Use `find_each` instead of `each` for batch processing.
User.includes(:posts).each { |u| u.save }
                      ^^^^ Rails/FindEach: Use `find_each` instead of `each` for batch processing.
User.joins(:posts).each { |u| u.save }
                   ^^^^ Rails/FindEach: Use `find_each` instead of `each` for batch processing.
User.all.each(&:save)
         ^^^^ Rails/FindEach: Use `find_each` instead of `each` for batch processing.
User.where(active: true).each(&:activate!)
                         ^^^^ Rails/FindEach: Use `find_each` instead of `each` for batch processing.
class Model < ApplicationRecord
  where(record: [record1, record2]).each(&:touch)
                                    ^^^^ Rails/FindEach: Use `find_each` instead of `each` for batch processing.
end
class Model < ::ApplicationRecord
  all.each { |u| u.x }
      ^^^^ Rails/FindEach: Use `find_each` instead of `each` for batch processing.
end
class Model < ActiveRecord::Base
  all.each { |u| u.x }
      ^^^^ Rails/FindEach: Use `find_each` instead of `each` for batch processing.
end
class Model < ::ActiveRecord::Base
  all.each { |u| u.x }
      ^^^^ Rails/FindEach: Use `find_each` instead of `each` for batch processing.
end
